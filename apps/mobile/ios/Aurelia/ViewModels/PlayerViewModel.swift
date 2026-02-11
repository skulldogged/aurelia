import Foundation
import Observation
import os
import AureliaCore

@Observable
final class PlayerViewModel: @unchecked Sendable {
    var title = ""
    var artist = ""
    var albumArtUrl: String?
    var isPlaying = false
    var isBuffering = false
    var positionMs: Int64 = 0
    var durationMs: Int64 = 0
    var positionUpdateTimeMs: Int64 = 0
    var queue: [Song] = []
    var currentQueueIndex: Int = -1
    var lyrics: Lyrics?
    var showLyrics = false
    var isShuffled = false
    var repeatMode: RepeatMode = .none
    var currentSongId: String?
    var currentAlbumId: String?
    var currentArtistId: String?
    var currentAlbumName: String?
    var isFavorite = false
    var isFavoriteLoading = false
    var playbackSpeed: Float = 1.0
    var codec: String?
    var bitRate: Int32?
    var sampleRate: Int32?

    var hasPrevious: Bool { currentQueueIndex > 0 }
    var hasNext: Bool { currentQueueIndex >= 0 && currentQueueIndex < queue.count - 1 }

    var formatInfo: String? {
        var parts: [String] = []
        if let codec { parts.append(codec.uppercased()) }
        if let sampleRate { parts.append("\(Double(sampleRate) / 1000.0) kHz") }
        if let bitRate { parts.append("\(bitRate / 1000) kbps") }
        return parts.isEmpty ? nil : parts.joined(separator: " / ")
    }

    private let sessionStore = SessionStore.shared
    private let logger = Logger(subsystem: "com.aurelia.app", category: "PlayerViewModel")
    private var lastFetchedSongId: String?
    private var favoriteCache: [String: Bool] = [:]

    /// Call this to sync state from the player controller.
    func updateFrom(_ snapshot: PlayerSnapshot, position: PlaybackPosition, playerController: AudioPlayerController) {
        let previousSongId = currentSongId
        let newSongId = snapshot.currentSongId
        let didChangeSong = newSongId != previousSongId
        let shouldRefreshQueue = queue.isEmpty || didChangeSong || snapshot.isShuffled != isShuffled

        if shouldRefreshQueue {
            let latestQueue = playerController.getQueue()
            if queueSignature(for: latestQueue) != queueSignature(for: queue) {
                queue = latestQueue
            }

            // Update favorite cache from queue only when queue changes.
            for song in latestQueue where favoriteCache[song.id] == nil {
                favoriteCache[song.id] = song.isFavorite ?? false
            }
        }

        let latestQueueIndex = playerController.getCurrentQueueIndex()

        setIfChanged(\.title, snapshot.title)
        setIfChanged(\.artist, snapshot.artist)
        setIfChanged(\.albumArtUrl, snapshot.albumArtUrl)
        setIfChanged(\.isPlaying, snapshot.isPlaying)
        setIfChanged(\.isBuffering, snapshot.isBuffering)
        setIfChanged(\.positionMs, position.positionMs)
        setIfChanged(\.durationMs, position.durationMs)
        setIfChanged(\.positionUpdateTimeMs, position.updateTimeMs)
        setIfChanged(\.currentQueueIndex, latestQueueIndex)
        setIfChanged(\.isShuffled, snapshot.isShuffled)
        setIfChanged(\.repeatMode, snapshot.repeatMode)
        setIfChanged(\.currentSongId, newSongId)
        setIfChanged(\.currentAlbumId, snapshot.currentAlbumId)
        setIfChanged(\.currentArtistId, snapshot.currentArtistId)
        setIfChanged(\.currentAlbumName, snapshot.currentAlbumName)
        setIfChanged(\.isFavorite, newSongId.flatMap { favoriteCache[$0] } ?? false)
        setIfChanged(\.playbackSpeed, snapshot.playbackSpeed)
        setIfChanged(\.codec, snapshot.codec)
        setIfChanged(\.bitRate, snapshot.bitRate)
        setIfChanged(\.sampleRate, snapshot.sampleRate)

        // Mark previous song as played on track change
        if let previousSongId, didChangeSong, newSongId != nil {
            markSongAsPlayed(previousSongId)
        }

        // Fetch lyrics for new song
        if let newSongId, !newSongId.isEmpty, didChangeSong, newSongId != lastFetchedSongId {
            lastFetchedSongId = newSongId
            fetchLyrics(songId: newSongId, artist: snapshot.artist, title: snapshot.title)
        }
    }

    // MARK: - Playback Controls

    func togglePlayPause(playerController: AudioPlayerController) {
        if isPlaying {
            playerController.pause()
        } else {
            playerController.resume()
        }
    }

    func seekTo(_ positionMs: Int64, playerController: AudioPlayerController) {
        playerController.seekTo(positionMs: positionMs)
    }

    func skipNext(playerController: AudioPlayerController) {
        playerController.skipNext()
    }

    func skipPrevious(playerController: AudioPlayerController) {
        playerController.skipPrevious()
    }

    func playQueueItem(_ index: Int, playerController: AudioPlayerController) {
        playerController.playQueueItem(index)
    }

    func toggleShuffle(playerController: AudioPlayerController) {
        playerController.toggleShuffle()
    }

    func cycleRepeatMode(playerController: AudioPlayerController) {
        playerController.cycleRepeatMode()
    }

    // MARK: - Lyrics

    func toggleLyrics() {
        if lyrics == nil && !title.isEmpty {
            fetchLyrics(songId: currentSongId, artist: artist, title: title)
        }
        showLyrics.toggle()
    }

    private func fetchLyrics(songId: String?, artist: String, title: String) {
        lyrics = nil
        
        Task { @MainActor [self] in
            let serverUrl = self.sessionStore.serverUrl ?? ""
            let token = self.sessionStore.token ?? ""
            let lyricsServerUrl = self.sessionStore.lyricsServerUrl
            let itemId = songId ?? ""

            self.logger.info("[Lyrics] Fetching lyrics for '\(title)' by '\(artist)' (itemId=\(itemId), serverUrl=\(serverUrl.prefix(30))..., hasToken=\(!token.isEmpty))")

            self.logger.info("[Lyrics] Debug: lyricsServerUrl = '\(lyricsServerUrl ?? "nil")'")
            let parsedLyrics = await getParsedLyrics(
                serverUrl: serverUrl,
                token: token,
                itemId: itemId,
                artist: artist,
                title: title,
                path: nil,
                lyricsServerUrl: lyricsServerUrl
            )

            self.logger.info("[Lyrics] Got ParsedLyrics from core: syncedLines=\(parsedLyrics.synced.count), plainLines=\(parsedLyrics.plain.count), areFromRemote=\(parsedLyrics.areFromRemote), hasSections=\(parsedLyrics.sections != nil), hasAgents=\(parsedLyrics.agents != nil), hasSongwriters=\(parsedLyrics.songwriters != nil), language=\(parsedLyrics.language ?? "nil")")

            // Log first few synced lines for debugging
            for (i, line) in parsedLyrics.synced.prefix(5).enumerated() {
                let wordCount = line.words?.count ?? 0
                self.logger.info("[Lyrics] synced[\(i)]: timeMs=\(line.timeMs), endTimeMs=\(line.endTimeMs.map { String($0) } ?? "nil"), words=\(wordCount), agentId=\(line.agentId ?? "nil"), text='\(line.line.prefix(60))'")
                if let words = line.words {
                    for (j, word) in words.prefix(3).enumerated() {
                        self.logger.info("[Lyrics] word[\(j)]: timeMs=\(word.timeMs), endTimeMs=\(word.endTimeMs.map { String($0) } ?? "nil"), '\(word.word)'")
                    }
                    if words.count > 3 {
                        self.logger.info("[Lyrics] ... and \(words.count - 3) more words")
                    }
                }
            }
            if parsedLyrics.synced.count > 5 {
                self.logger.info("[Lyrics] ... and \(parsedLyrics.synced.count - 5) more synced lines")
            }

            let parsed = LyricsParser.fromParsed(parsedLyrics)
            let isValid = parsed.isValid
            let hasSynced = parsed.synced != nil
            let syncedCount = parsed.synced?.count ?? 0
            let hasPlain = parsed.plain != nil
            let hasWordSync = parsed.synced?.first?.words != nil
            self.logger.info("[Lyrics] After LyricsParser: isValid=\(isValid), hasSynced=\(hasSynced), syncedCount=\(syncedCount), hasPlain=\(hasPlain), hasWordSync=\(hasWordSync)")

            if songId == self.currentSongId {
                if isValid {
                    self.logger.info("[Lyrics] Setting lyrics on view model (valid)")
                    self.lyrics = parsed
                } else {
                    self.logger.warning("[Lyrics] Lyrics invalid, hiding lyrics panel")
                    self.showLyrics = false
                }
        } else {
            self.logger.info("[Lyrics] Song changed during fetch (was=\(songId ?? "nil"), now=\(self.currentSongId ?? "nil")), discarding result")
        }
    }
}

// MARK: - Favorites

    func toggleFavorite() {
        guard let songId = currentSongId else { return }
        guard let serverUrl = sessionStore.serverUrl,
              let token = sessionStore.token,
              let userId = sessionStore.userId else { return }
        let targetFavoriteState = !isFavorite

        isFavoriteLoading = true

        Task.detached { [self, targetFavoriteState] in
            do {
                let newState = try await AureliaCore.toggleFavorite(
                    serverUrl: serverUrl,
                    token: token,
                    userId: userId,
                    itemId: songId,
                    isFavorite: targetFavoriteState
                )
                await MainActor.run {
                    self.favoriteCache[songId] = newState
                    self.isFavorite = newState
                    self.isFavoriteLoading = false
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    self.logger.error("Failed to toggle favorite: \(error)")
                    await MainActor.run { self.isFavoriteLoading = false }
                }
            }
        }
    }

    // MARK: - Scrobble

    private func setIfChanged<T: Equatable>(_ keyPath: ReferenceWritableKeyPath<PlayerViewModel, T>, _ value: T) {
        if self[keyPath: keyPath] != value {
            self[keyPath: keyPath] = value
        }
    }

    private func queueSignature(for queue: [Song]) -> String {
        guard let first = queue.first?.id, let last = queue.last?.id else {
            return "empty:\(queue.count)"
        }
        return "\(queue.count):\(first):\(last)"
    }

    private func markSongAsPlayed(_ songId: String) {
        guard let serverUrl = sessionStore.serverUrl,
              let token = sessionStore.token,
              let userId = sessionStore.userId else { return }

        Task.detached {
            do {
                try await markItemPlayed(
                    serverUrl: serverUrl,
                    token: token,
                    userId: userId,
                    itemId: songId
                )
            } catch {
                self.logger.error("Failed to mark song as played: \(songId) - \(error)")
            }
        }
    }
}
