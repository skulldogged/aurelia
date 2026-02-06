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
    func updateFrom(_ snapshot: PlayerSnapshot, playerController: AudioPlayerController) {
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
        setIfChanged(\.positionMs, snapshot.positionMs)
        setIfChanged(\.durationMs, snapshot.durationMs)
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

        Task.detached { [self, songId] in
                let serverUrl = await self.sessionStore.serverUrl ?? ""
                let token = await self.sessionStore.token ?? ""
                let itemId = songId ?? ""

                let parsedLyrics = await getParsedLyrics(
                    serverUrl: serverUrl,
                    token: token,
                    itemId: itemId,
                    artist: artist,
                    title: title
                )
            let parsed = await LyricsParser.fromParsed(parsedLyrics)

                await MainActor.run {
                    if songId == self.currentSongId {
                        if parsed.isValid {
                            self.lyrics = parsed
                        } else {
                            self.showLyrics = false
                        }
                    }
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
