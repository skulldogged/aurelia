import AVFoundation
import MediaPlayer
import Observation
import os
import AureliaCore

/// iOS audio player using AVQueuePlayer with Now Playing integration.
/// Equivalent to Android's PlayerController + PlaybackService.
@Observable
final class AudioPlayerController: @unchecked Sendable {
    // MARK: - Published State

    private(set) var snapshot = PlayerSnapshot()
    private(set) var isReady = false

    // MARK: - Private State

    private var player: AVQueuePlayer
    private var songQueue: [Song] = []
    private var songByMediaId: [String: Song] = [:]
    private var currentIndex: Int = -1
    private var lastServerUrl: String = ""
    private var lastToken: String = ""
    private var seekOffsetMs: Int64 = 0
    private var timeObserver: Any?
    private var audioSessionConfigured = false
    private let logger = Logger(subsystem: "com.aurelia.app", category: "AudioPlayer")

    private static let seekableContainers: Set<String> = ["flac", "mp3", "aac", "ogg"]

    init() {
        player = AVQueuePlayer()
        player.allowsExternalPlayback = false
        setupRemoteTransportControls()
        setupNotifications()
        setupTimeObserver()
    }

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
        } catch {
            logger.error("Failed to configure audio session: \(error)")
        }
    }

    private func ensureAudioSession() {
        guard !audioSessionConfigured else { return }
        audioSessionConfigured = true
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
            if self.snapshot.isPlaying {
                self.pause()
            } else {
                self.resume()
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
            guard let self,
                  let item = notification.object as? AVPlayerItem,
                  item == self.player.currentItem else { return }
            self.handleTrackEnded()
        }
    }

    // MARK: - Time Observer

    private func setupTimeObserver() {
        let interval = CMTime(seconds: 0.5, preferredTimescale: 600)
        timeObserver = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) { [weak self] time in
            self?.updateSnapshot()
        }
    }

    // MARK: - Queue Management

    func setQueue(_ songs: [Song], serverUrl: String, token: String, startIndex: Int = 0, autoPlay: Bool = true) {
        ensureAudioSession()
        songQueue = songs
        songByMediaId.removeAll()
        songs.forEach { songByMediaId[$0.id] = $0 }
        lastServerUrl = serverUrl
        lastToken = token
        seekOffsetMs = 0
        currentIndex = startIndex

        player.removeAllItems()

        // Build and insert items starting from startIndex
        for i in startIndex..<songs.count {
            let url = buildStreamUrl(for: songs[i])
            if let playerItem = makePlayerItem(url: url) {
                player.insert(playerItem, after: nil)
            }
        }

        if autoPlay {
            player.play()
        }

        updateSnapshot()
        updateNowPlayingInfo()
    }

    func getQueue() -> [Song] { songQueue }
    func getCurrentQueueIndex() -> Int { currentIndex }

    func addToQueue(_ song: Song, serverUrl: String, token: String) {
        songByMediaId[song.id] = song
        songQueue.append(song)
        let url = buildStreamUrl(for: song, serverUrl: serverUrl, token: token)
        if let item = makePlayerItem(url: url) {
            player.insert(item, after: nil)
        }
    }

    func playNext(_ song: Song, serverUrl: String, token: String) {
        songByMediaId[song.id] = song
        let insertIndex = currentIndex + 1
        songQueue.insert(song, at: min(insertIndex, songQueue.count))
        let url = buildStreamUrl(for: song, serverUrl: serverUrl, token: token)
        if let item = makePlayerItem(url: url) {
            // Insert after the currently playing item
            player.insert(item, after: player.currentItem)
        }
    }

    func playQueueItem(_ index: Int) {
        guard index >= 0, index < songQueue.count else { return }
        ensureAudioSession()
        seekOffsetMs = 0

        // Rebuild queue from the target index
        player.removeAllItems()
        currentIndex = index
        for i in index..<songQueue.count {
            let url = buildStreamUrl(for: songQueue[i])
            if let item = makePlayerItem(url: url) {
                player.insert(item, after: nil)
            }
        }
        player.play()
        updateSnapshot()
        updateNowPlayingInfo()
    }

    // MARK: - Playback Controls

    func play(_ song: Song, serverUrl: String, token: String) {
        setQueue([song], serverUrl: serverUrl, token: token)
    }

    func pause() {
        player.pause()
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

        if !Self.isContainerSeekable(song.container) && !lastServerUrl.isEmpty {
            // Non-seekable container: rebuild URL with startTimeTicks
            let ticks = targetPosition * 10_000
            let baseUrl = buildMobileStreamUrl(
                serverUrl: lastServerUrl,
                token: lastToken,
                itemId: song.id,
                container: song.container
            )
            let seekUrl = "\(baseUrl)&startTimeTicks=\(ticks)"
            let wasPlaying = player.rate > 0

            seekOffsetMs = targetPosition

            // Rebuild from current index with the new URL for the current item
            player.removeAllItems()
            if let seekItem = makePlayerItem(url: seekUrl) {
                player.insert(seekItem, after: nil)
            }
            // Re-add subsequent items
            for i in (currentIndex + 1)..<songQueue.count {
                let url = buildStreamUrl(for: songQueue[i])
                if let item = makePlayerItem(url: url) {
                    player.insert(item, after: nil)
                }
            }
            if wasPlaying { player.play() }
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
        currentIndex += 1
        player.advanceToNextItem()
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
        // When shuffle is toggled, we keep the current song but randomize the rest
        if snapshot.isShuffled, currentIndex >= 0, currentIndex < songQueue.count {
            let current = songQueue[currentIndex]
            var remaining = songQueue
            remaining.remove(at: currentIndex)
            remaining.shuffle()
            songQueue = [current] + remaining
            currentIndex = 0
        }
        updateSnapshot()
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
            updateSnapshot()
            updateNowPlayingInfo()
        } else if snapshot.repeatMode == .all && !songQueue.isEmpty {
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

        snapshot = PlayerSnapshot(
            title: song?.name ?? "",
            artist: song?.artists?.joined(separator: ", ") ?? "",
            albumArtUrl: song?.albumArtUrl,
            isPlaying: player.rate > 0,
            isBuffering: player.currentItem?.status == .unknown,
            positionMs: positionMs,
            durationMs: durationMs,
            hasPrevious: currentIndex > 0,
            hasNext: currentIndex + 1 < songQueue.count,
            isShuffled: snapshot.isShuffled,
            repeatMode: snapshot.repeatMode,
            currentSongId: song?.id,
            currentAlbumId: song?.albumId,
            currentArtistId: song?.artistIds?.first,
            currentAlbumName: song?.album,
            playbackSpeed: player.rate != 0 ? player.rate : snapshot.playbackSpeed,
            updateTimeMs: Int64(ProcessInfo.processInfo.systemUptime * 1000),
            codec: song?.codec,
            bitRate: song?.bitRate.flatMap { Int32($0) },
            sampleRate: song?.sampleRate.flatMap { Int32($0) }
        )
    }

    // MARK: - Now Playing Info (Lock Screen / Control Center)

    private func updateNowPlayingInfo() {
        guard currentIndex >= 0, currentIndex < songQueue.count else { return }
        let song = songQueue[currentIndex]

        let info: [String: Any] = [
            MPMediaItemPropertyTitle: song.name,
            MPMediaItemPropertyArtist: song.artists?.joined(separator: ", ") ?? "",
            MPMediaItemPropertyAlbumTitle: song.album ?? "",
            MPNowPlayingInfoPropertyElapsedPlaybackTime: Double(snapshot.positionMs) / 1000.0,
            MPMediaItemPropertyPlaybackDuration: Double(snapshot.durationMs) / 1000.0,
            MPNowPlayingInfoPropertyPlaybackRate: player.rate,
        ]

        // Load artwork asynchronously
        if let artUrlString = song.albumArtUrl, let artUrl = URL(string: artUrlString) {
            let targetSongId = song.id
            Task.detached { [targetSongId] in
                let cached = await ImageCache.shared.cachedImage(for: artUrl)
                let image: UIImage?
                if let cached {
                    image = cached
                } else {
                    guard let (data, _) = try? await URLSession.shared.data(from: artUrl),
                          let fetched = UIImage(data: data) else { return }
                    await ImageCache.shared.store(fetched, for: artUrl)
                    image = fetched
                }
                guard let image else { return }
                let artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
                await MainActor.run { [weak self] in
                    guard let self, self.snapshot.currentSongId == targetSongId else { return }
                    var currentInfo = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
                    currentInfo[MPMediaItemPropertyArtwork] = artwork
                    MPNowPlayingInfoCenter.default().nowPlayingInfo = currentInfo
                }
            }
        }

        MPNowPlayingInfoCenter.default().nowPlayingInfo = info
    }

    // MARK: - URL Building

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

    private static func isContainerSeekable(_ container: String?) -> Bool {
        guard let container = container?.lowercased() else { return false }
        return seekableContainers.contains(container)
    }
}
