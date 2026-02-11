import AureliaCore
import AVFoundation
import MediaPlayer
import Observation
import os

/// iOS audio player using AVQueuePlayer with Now Playing integration.
/// Equivalent to Android's PlayerController + PlaybackService.
@Observable
final class AudioPlayerController: @unchecked Sendable {
    // MARK: - Published State

    private(set) var snapshot = PlayerSnapshot()
    private(set) var playbackPosition = PlaybackPosition()
    private(set) var isReady = false

    // MARK: - Private State

    private var player: AVQueuePlayer
    private var songQueue: [Song] = []
    private var queueBeforeShuffle: [Song]?
    private var currentIndex: Int = -1
    private var lastServerUrl: String = ""
    private var lastToken: String = ""
    private var seekOffsetMs: Int64 = 0
    private var loadedQueueRange: ClosedRange<Int>?
    private var timeObserver: Any?
    private var audioSessionConfigured = false
    private let logger = Logger(subsystem: "com.aurelia.app", category: "AudioPlayer")

    private static let seekableContainers: Set<String> = ["flac", "mp3", "aac", "ogg"]
    private static let maxPreloadedItems = 10
    private static let initialPreloadedItems = 3

    init() {
        player = AVQueuePlayer()
        player.allowsExternalPlayback = false
        // Configure for background playback
        player.automaticallyWaitsToMinimizeStalling = true
        configureAudioSession()
        setupRemoteTransportControls()
        setupNotifications()
        setupTimeObserver()
        // Begin receiving remote control events early for Control Center integration
        UIApplication.shared.beginReceivingRemoteControlEvents()
    }

    @MainActor
    deinit {
        if let observer = timeObserver {
            player.removeTimeObserver(observer)
        }
        player.pause()
    }

    // MARK: - Audio Session

    private func configureAudioSession() {
        do {
            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.playback, mode: .default, options: [])
            try session.setActive(true)
            audioSessionConfigured = true
            logger.info("Audio session configured successfully")
        } catch {
            logger.error("Failed to configure audio session: \(error)")
            audioSessionConfigured = false
        }
    }

    private func ensureAudioSession() {
        guard !audioSessionConfigured else { return }
        configureAudioSession()
    }

    // MARK: - Remote Controls (Lock Screen / Control Center)

    private func setupRemoteTransportControls() {
        let commandCenter = MPRemoteCommandCenter.shared()

        commandCenter.playCommand.addTarget { [weak self] _ in
            self?.resume()
            return .success
        }

        commandCenter.pauseCommand.addTarget { [weak self] _ in
            self?.pause()
            return .success
        }

        commandCenter.nextTrackCommand.addTarget { [weak self] _ in
            self?.skipNext()
            return .success
        }

        commandCenter.previousTrackCommand.addTarget { [weak self] _ in
            self?.skipPrevious()
            return .success
        }

        commandCenter.changePlaybackPositionCommand.addTarget { [weak self] event in
            guard let event = event as? MPChangePlaybackPositionCommandEvent else { return .commandFailed }
            self?.seekTo(positionMs: Int64(event.positionTime * 1000))
            return .success
        }

        commandCenter.togglePlayPauseCommand.addTarget { [weak self] _ in
            guard let self else { return .commandFailed }
            if snapshot.isPlaying {
                pause()
            } else {
                resume()
            }
            return .success
        }
    }

    // MARK: - Notifications

    private func setupNotifications() {
        NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let self else { return }
            // Extract Sendable values before entering isolated context
            let notificationObject = notification.object as? AVPlayerItem
            Task { @MainActor in
                guard let item = notificationObject,
                      item == self.player.currentItem else { return }
                self.handleTrackEnded()
            }
        }

        // Handle audio session interruptions
        NotificationCenter.default.addObserver(
            forName: AVAudioSession.interruptionNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let self else { return }
            // Extract Sendable values before entering isolated context
            let typeValue = notification.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
            let optionsValue = notification.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt
            Task { @MainActor in
                self.handleAudioSessionInterruption(typeValue: typeValue, optionsValue: optionsValue)
            }
        }

        // Handle app lifecycle
        NotificationCenter.default.addObserver(
            forName: UIApplication.didEnterBackgroundNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            guard let self else { return }
            Task { @MainActor in
                self.logger.info("App entered background, ensuring audio session stays active")
                self.ensureAudioSessionActive()
            }
        }

        NotificationCenter.default.addObserver(
            forName: UIApplication.willEnterForegroundNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            guard let self else { return }
            Task { @MainActor in
                self.logger.info("App will enter foreground")
                self.ensureAudioSessionActive()
            }
        }
    }

    private func handleAudioSessionInterruption(typeValue: UInt?, optionsValue: UInt?) {
        guard let typeValue,
              let type = AVAudioSession.InterruptionType(rawValue: typeValue)
        else {
            return
        }

        switch type {
        case .began:
            logger.info("Audio session interruption began")
        // Interruption began, audio will pause automatically
        case .ended:
            logger.info("Audio session interruption ended")
            if let optionsValue {
                let options = AVAudioSession.InterruptionOptions(rawValue: optionsValue)
                if options.contains(.shouldResume), snapshot.isPlaying {
                    logger.info("Resuming playback after interruption")
                    resume()
                }
            }
        @unknown default:
            break
        }
    }

    private func ensureAudioSessionActive() {
        do {
            let session = AVAudioSession.sharedInstance()
            if !session.isOtherAudioPlaying {
                try session.setActive(true)
                logger.info("Audio session reactivated")
            }
        } catch {
            logger.error("Failed to reactivate audio session: \(error)")
        }
    }

    // MARK: - Time Observer

    private func setupTimeObserver() {
        let interval = CMTime(seconds: 0.5, preferredTimescale: 600)
        timeObserver = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) { [weak self] _ in
            MainActor.assumeIsolated {
                self?.updateSnapshot()
            }
        }

        // Observe rate changes to detect unexpected pauses
        _ = player.observe(\.rate, options: [.new]) { [weak self] _, change in
            MainActor.assumeIsolated {
                guard let self else { return }
                let newRate = change.newValue ?? 0
                self.logger.info("Player rate changed to: \(newRate)")
            }
        }
    }

    // MARK: - Queue Management

    func setQueue(_ songs: [Song], serverUrl: String, token: String, startIndex: Int = 0, autoPlay: Bool = true) {
        ensureAudioSession()
        songQueue = songs
        queueBeforeShuffle = snapshot.isShuffled ? songs : nil
        lastServerUrl = serverUrl
        lastToken = token
        seekOffsetMs = 0

        guard let safeStart = normalizedStartIndex(startIndex, queueCount: songs.count) else {
            player.removeAllItems()
            loadedQueueRange = nil
            currentIndex = -1
            updateSnapshot()
            MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
            return
        }

        rebuildPlayerQueue(startingAt: safeStart, preloadItemCount: Self.initialPreloadedItems)
        if snapshot.isShuffled {
            shuffleUpcomingQueueInPlace()
        }

        if autoPlay {
            player.play()
        } else {
            player.pause()
        }

        scheduleDeferredPreload()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    func getQueue() -> [Song] {
        songQueue
    }

    func getCurrentQueueIndex() -> Int {
        currentIndex
    }

    func addToQueue(_ song: Song, serverUrl: String, token: String) {
        lastServerUrl = serverUrl
        lastToken = token
        songQueue.append(song)
        queueBeforeShuffle?.append(song)

        if loadedQueueRange != nil {
            preloadUpcomingItems()
        }
    }

    func playNext(_ song: Song, serverUrl: String, token: String) {
        lastServerUrl = serverUrl
        lastToken = token
        let insertIndex = min(currentIndex + 1, songQueue.count)
        songQueue.insert(song, at: min(insertIndex, songQueue.count))
        if var baseQueue = queueBeforeShuffle {
            if currentIndex >= 0,
               currentIndex < songQueue.count,
               let currentSongId = songQueue[safe: currentIndex]?.id,
               let baseCurrentIndex = baseQueue.firstIndex(where: { $0.id == currentSongId })
            {
                let baseInsertIndex = min(baseCurrentIndex + 1, baseQueue.count)
                baseQueue.insert(song, at: baseInsertIndex)
            } else {
                baseQueue.append(song)
            }
            queueBeforeShuffle = baseQueue
        }

        if currentIndex >= 0, currentIndex < songQueue.count {
            let wasPlaying = player.rate > 0
            rebuildPlayerQueue(startingAt: currentIndex)
            if wasPlaying {
                player.play()
            }
            updateSnapshot()
            updateNowPlayingInfo()
        }
    }

    func playQueueItem(_ index: Int) {
        guard index >= 0, index < songQueue.count else { return }
        ensureAudioSession()
        seekOffsetMs = 0

        rebuildPlayerQueue(startingAt: index, preloadItemCount: Self.initialPreloadedItems)
        player.play()
        scheduleDeferredPreload()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    // MARK: - Playback Controls

    func play(_ song: Song, serverUrl: String, token: String) {
        setQueue([song], serverUrl: serverUrl, token: token)
    }

    func pause() {
        logger.info("Pause called")
        player.pause()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    func play() {
        logger.info("Play called")
        ensureAudioSession()
        player.play()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    func resume() {
        ensureAudioSession()
        player.play()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    func stop() {
        player.pause()
        player.removeAllItems()
        songQueue.removeAll()
        queueBeforeShuffle = nil
        loadedQueueRange = nil
        currentIndex = -1
        seekOffsetMs = 0
        updateSnapshot()
        MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
    }

    func seekTo(positionMs: Int64) {
        guard currentIndex >= 0, currentIndex < songQueue.count else { return }
        let song = songQueue[currentIndex]

        let songDurationMs = Int64((song.duration ?? 0) * 1000)
        let targetPosition = max(0, min(positionMs, songDurationMs > 0 ? songDurationMs : positionMs))

        if !Self.isContainerSeekable(song.container), !lastServerUrl.isEmpty {
            // Non-seekable container: rebuild URL with startTimeTicks
            let ticks = targetPosition * 10000
            let baseUrl = buildMobileStreamUrl(
                serverUrl: lastServerUrl,
                token: lastToken,
                itemId: song.id,
                container: song.container
            )
            let seekUrl = "\(baseUrl)&startTimeTicks=\(ticks)"
            let wasPlaying = player.rate > 0

            seekOffsetMs = targetPosition

            rebuildPlayerQueue(startingAt: currentIndex, firstItemOverrideUrl: seekUrl)
            if wasPlaying {
                player.play()
            }
        } else {
            let time = CMTime(seconds: Double(targetPosition) / 1000.0, preferredTimescale: 600)
            player.seek(to: time)
        }

        updateSnapshot()
        updateNowPlayingInfo()
    }

    func skipNext() {
        guard currentIndex + 1 < songQueue.count else { return }
        seekOffsetMs = 0
        if loadedQueueRange?.contains(currentIndex + 1) != true {
            rebuildPlayerQueue(startingAt: currentIndex)
        }
        currentIndex += 1
        player.advanceToNextItem()
        if let range = loadedQueueRange {
            loadedQueueRange = currentIndex ... max(currentIndex, range.upperBound)
        } else {
            loadedQueueRange = currentIndex ... currentIndex
        }
        preloadUpcomingItems()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    func skipPrevious() {
        seekOffsetMs = 0
        if currentIndex > 0 {
            currentIndex -= 1
            playQueueItem(currentIndex)
        } else {
            // Seek to beginning
            player.seek(to: .zero)
        }
        updateSnapshot()
        updateNowPlayingInfo()
    }

    func toggleShuffle() {
        snapshot.isShuffled.toggle()

        if snapshot.isShuffled {
            queueBeforeShuffle = songQueue
            // Match platform-native behavior: keep current playback uninterrupted.
            if currentIndex >= 0, currentIndex < songQueue.count {
                shuffleUpcomingQueueInPlace()
            }
        } else {
            restoreQueueAfterShuffle()
            queueBeforeShuffle = nil
        }

        updateSnapshot()
        updateNowPlayingInfo()
    }

    func cycleRepeatMode() {
        switch snapshot.repeatMode {
        case .none: snapshot.repeatMode = .one
        case .one: snapshot.repeatMode = .all
        case .all: snapshot.repeatMode = .none
        }
        updateSnapshot()
    }

    // MARK: - Track Transition

    private func handleTrackEnded() {
        if snapshot.repeatMode == .one {
            // Replay current track
            player.seek(to: .zero)
            player.play()
            return
        }

        seekOffsetMs = 0

        if currentIndex + 1 < songQueue.count {
            currentIndex += 1
            if let range = loadedQueueRange {
                loadedQueueRange = currentIndex ... max(currentIndex, range.upperBound)
            } else {
                loadedQueueRange = currentIndex ... currentIndex
            }
            preloadUpcomingItems()
            updateSnapshot()
            updateNowPlayingInfo()
        } else if snapshot.repeatMode == .all, !songQueue.isEmpty {
            // Loop back to beginning
            playQueueItem(0)
        } else {
            // End of queue
            updateSnapshot()
            MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
        }
    }

    // MARK: - Snapshot

    private func updateSnapshot() {
        let currentTime = player.currentTime()
        let positionMs = max(0, Int64(CMTimeGetSeconds(currentTime) * 1000)) + seekOffsetMs

        let song = currentIndex >= 0 && currentIndex < songQueue.count ? songQueue[currentIndex] : nil
        let durationMs: Int64
        if let d = song?.duration, d > 0 {
            durationMs = Int64(d * 1000)
        } else if let item = player.currentItem {
            let dur = CMTimeGetSeconds(item.duration)
            durationMs = dur.isFinite ? Int64(dur * 1000) + seekOffsetMs : 0
        } else {
            durationMs = 0
        }

        // Always update position (separate from snapshot so views reading
        // only display state aren't invalidated by the 500ms timer tick).
        let newPosition = PlaybackPosition(
            positionMs: positionMs,
            durationMs: durationMs,
            updateTimeMs: Int64(ProcessInfo.processInfo.systemUptime * 1000)
        )
        if playbackPosition != newPosition {
            playbackPosition = newPosition
        }

        // Update Now Playing elapsed time during playback for smooth progress display
        if player.rate > 0, MPNowPlayingInfoCenter.default().nowPlayingInfo != nil {
            var nowPlayingInfo = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
            nowPlayingInfo[MPNowPlayingInfoPropertyElapsedPlaybackTime] = Double(positionMs) / 1000.0
            MPNowPlayingInfoCenter.default().nowPlayingInfo = nowPlayingInfo
        }

        // Build candidate and only replace snapshot when display fields change.
        // Position/duration/updateTime are tracked separately via `playbackPosition`
        // so that views reading only display state are not invalidated by position ticks.
        let candidate = PlayerSnapshot(
            title: song?.name ?? "",
            artist: song?.artists?.joined(separator: ", ") ?? "",
            albumArtUrl: song?.albumArtUrl,
            isPlaying: player.rate > 0,
            isBuffering: player.currentItem?.status == .unknown,
            hasPrevious: currentIndex > 0,
            hasNext: currentIndex + 1 < songQueue.count,
            isShuffled: snapshot.isShuffled,
            repeatMode: snapshot.repeatMode,
            currentSongId: song?.id,
            currentAlbumId: song?.albumId,
            currentArtistId: song?.artistIds?.first,
            currentAlbumName: song?.album,
            playbackSpeed: player.rate != 0 ? player.rate : snapshot.playbackSpeed,
            codec: song?.codec,
            bitRate: song?.bitRate.flatMap { Int32($0) },
            sampleRate: song?.sampleRate.flatMap { Int32($0) }
        )

        if snapshot != candidate {
            snapshot = candidate
        }
    }

    // MARK: - Now Playing Info (Lock Screen / Control Center)

    private func updateNowPlayingInfo() {
        guard currentIndex >= 0, currentIndex < songQueue.count else { return }
        let song = songQueue[currentIndex]

        let info: [String: Any] = [
            MPMediaItemPropertyTitle: song.name,
            MPMediaItemPropertyArtist: song.artists?.joined(separator: ", ") ?? "",
            MPMediaItemPropertyAlbumTitle: song.album ?? "",
            MPNowPlayingInfoPropertyElapsedPlaybackTime: Double(playbackPosition.positionMs) / 1000.0,
            MPMediaItemPropertyPlaybackDuration: Double(playbackPosition.durationMs) / 1000.0,
            MPNowPlayingInfoPropertyPlaybackRate: player.rate,
        ]

        // Load artwork asynchronously
        if let artUrlString = song.albumArtUrl, let artUrl = URL(string: artUrlString) {
            let targetSongId = song.id
            Task.detached { [targetSongId] in
                let image = await ImageCache.shared.fetchImage(
                    for: artUrl,
                    targetSize: CGSize(width: 512, height: 512)
                )
                guard let image else { return }
                let artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
                await MainActor.run { [weak self] in
                    guard let self, snapshot.currentSongId == targetSongId else { return }
                    var currentInfo = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
                    currentInfo[MPMediaItemPropertyArtwork] = artwork
                    MPNowPlayingInfoCenter.default().nowPlayingInfo = currentInfo
                }
            }
        }

        MPNowPlayingInfoCenter.default().nowPlayingInfo = info
    }

    // MARK: - URL Building

    private func normalizedStartIndex(_ index: Int, queueCount: Int) -> Int? {
        guard queueCount > 0 else { return nil }
        return min(max(index, 0), queueCount - 1)
    }

    private func rebuildPlayerQueue(startingAt index: Int, firstItemOverrideUrl: String? = nil, preloadItemCount: Int = AudioPlayerController.maxPreloadedItems) {
        guard let safeStart = normalizedStartIndex(index, queueCount: songQueue.count) else {
            player.removeAllItems()
            loadedQueueRange = nil
            currentIndex = -1
            return
        }

        player.removeAllItems()
        currentIndex = safeStart

        let preloadCount = max(1, min(preloadItemCount, Self.maxPreloadedItems))
        let endIndex = min(songQueue.count - 1, safeStart + preloadCount - 1)
        for queueIndex in safeStart ... endIndex {
            let url: String = if queueIndex == safeStart, let firstItemOverrideUrl {
                firstItemOverrideUrl
            } else {
                buildStreamUrl(for: songQueue[queueIndex])
            }
            if let playerItem = makePlayerItem(url: url) {
                player.insert(playerItem, after: nil)
            }
        }
        loadedQueueRange = safeStart ... endIndex
    }

    private func preloadUpcomingItems() {
        guard currentIndex >= 0, currentIndex < songQueue.count else {
            loadedQueueRange = nil
            return
        }

        guard var range = loadedQueueRange else {
            rebuildPlayerQueue(startingAt: currentIndex)
            return
        }

        if range.lowerBound != currentIndex {
            range = currentIndex ... max(currentIndex, range.upperBound)
        }

        let desiredEnd = min(songQueue.count - 1, currentIndex + Self.maxPreloadedItems - 1)
        var nextIndex = range.upperBound + 1

        while nextIndex <= desiredEnd {
            let url = buildStreamUrl(for: songQueue[nextIndex])
            if let item = makePlayerItem(url: url) {
                player.insert(item, after: nil)
            }
            range = currentIndex ... nextIndex
            nextIndex += 1
        }

        loadedQueueRange = range
    }

    private func scheduleDeferredPreload() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { [weak self] in
            self?.preloadUpcomingItems()
        }
    }

    private func buildStreamUrl(for song: Song, serverUrl: String? = nil, token: String? = nil) -> String {
        buildMobileStreamUrl(
            serverUrl: serverUrl ?? lastServerUrl,
            token: token ?? lastToken,
            itemId: song.id,
            container: song.container
        )
    }

    private func makePlayerItem(url: String) -> AVPlayerItem? {
        guard let itemUrl = URL(string: url) else {
            logger.error("Invalid stream URL: \(url)")
            return nil
        }
        return AVPlayerItem(url: itemUrl)
    }

    private func shuffleUpcomingQueueInPlace() {
        guard currentIndex >= 0, currentIndex < songQueue.count else { return }
        guard let currentItem = player.currentItem else {
            rebuildPlayerQueue(startingAt: currentIndex)
            return
        }

        let upcomingStart = currentIndex + 1
        guard upcomingStart < songQueue.count else {
            loadedQueueRange = currentIndex ... currentIndex
            return
        }

        let playedAndCurrent = Array(songQueue.prefix(upcomingStart))
        var upcoming = Array(songQueue.suffix(from: upcomingStart))
        upcoming.shuffle()
        songQueue = playedAndCurrent + upcoming

        // Keep the current AVPlayerItem alive, replace only queued upcoming items.
        for item in player.items() where item !== currentItem {
            player.remove(item)
        }

        let endIndex = min(songQueue.count - 1, currentIndex + Self.maxPreloadedItems - 1)
        if upcomingStart <= endIndex {
            for queueIndex in upcomingStart ... endIndex {
                let url = buildStreamUrl(for: songQueue[queueIndex])
                if let item = makePlayerItem(url: url) {
                    player.insert(item, after: nil)
                }
            }
            loadedQueueRange = currentIndex ... endIndex
        } else {
            loadedQueueRange = currentIndex ... currentIndex
        }

        scheduleDeferredPreload()
    }

    private func restoreQueueAfterShuffle() {
        guard currentIndex >= 0, currentIndex < songQueue.count else { return }
        guard let originalQueue = queueBeforeShuffle else { return }
        let currentSong = songQueue[currentIndex]
        let currentOccurrence = occurrenceCount(
            songId: currentSong.id,
            in: songQueue,
            through: currentIndex
        )

        guard let currentItem = player.currentItem else {
            let restoredQueue = mergedQueuePreservingAdditions(base: originalQueue, current: songQueue)
            songQueue = restoredQueue
            if let restoredIndex = indexOfOccurrence(songId: currentSong.id, occurrence: currentOccurrence, in: restoredQueue) {
                currentIndex = restoredIndex
            } else if let fallbackIndex = restoredQueue.firstIndex(where: { $0.id == currentSong.id }) {
                currentIndex = fallbackIndex
            } else {
                currentIndex = restoredQueue.isEmpty ? -1 : min(currentIndex, restoredQueue.count - 1)
            }
            return
        }

        let restoredQueue = mergedQueuePreservingAdditions(base: originalQueue, current: songQueue)
        guard !restoredQueue.isEmpty else { return }

        songQueue = restoredQueue
        if let restoredIndex = indexOfOccurrence(songId: currentSong.id, occurrence: currentOccurrence, in: restoredQueue) {
            currentIndex = restoredIndex
        } else if let fallbackIndex = restoredQueue.firstIndex(where: { $0.id == currentSong.id }) {
            currentIndex = fallbackIndex
        } else {
            currentIndex = min(currentIndex, restoredQueue.count - 1)
        }

        for item in player.items() where item !== currentItem {
            player.remove(item)
        }

        let upcomingStart = currentIndex + 1
        let endIndex = min(songQueue.count - 1, currentIndex + Self.maxPreloadedItems - 1)
        if upcomingStart <= endIndex {
            for queueIndex in upcomingStart ... endIndex {
                let url = buildStreamUrl(for: songQueue[queueIndex])
                if let item = makePlayerItem(url: url) {
                    player.insert(item, after: nil)
                }
            }
            loadedQueueRange = currentIndex ... endIndex
        } else {
            loadedQueueRange = currentIndex ... currentIndex
        }

        scheduleDeferredPreload()
    }

    private func mergedQueuePreservingAdditions(base: [Song], current: [Song]) -> [Song] {
        var merged = base
        var remainingById: [String: Int] = [:]
        for song in base {
            remainingById[song.id, default: 0] += 1
        }

        for song in current {
            let remaining = remainingById[song.id, default: 0]
            if remaining > 0 {
                remainingById[song.id] = remaining - 1
            } else {
                merged.append(song)
            }
        }

        return merged
    }

    private func occurrenceCount(songId: String, in queue: [Song], through index: Int) -> Int {
        guard !queue.isEmpty else { return 1 }
        let upperBound = min(max(index, 0), queue.count - 1)
        return max(1, queue[0 ... upperBound].filter { $0.id == songId }.count)
    }

    private func indexOfOccurrence(songId: String, occurrence: Int, in queue: [Song]) -> Int? {
        var seen = 0
        for (index, song) in queue.enumerated() where song.id == songId {
            seen += 1
            if seen == occurrence {
                return index
            }
        }
        return nil
    }

    private static func isContainerSeekable(_ container: String?) -> Bool {
        guard let container = container?.lowercased() else { return false }
        return seekableContainers.contains(container)
    }
}

private extension Array {
    subscript(safe index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
