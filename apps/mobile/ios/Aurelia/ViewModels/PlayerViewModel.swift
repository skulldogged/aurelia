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

        // Update favorite cache from queue
        for song in playerController.getQueue() {
            if favoriteCache[song.id] == nil {
                favoriteCache[song.id] = song.isFavorite ?? false
            }
        }

        title = snapshot.title
        artist = snapshot.artist
        albumArtUrl = snapshot.albumArtUrl
        isPlaying = snapshot.isPlaying
        isBuffering = snapshot.isBuffering
        positionMs = snapshot.positionMs
        durationMs = snapshot.durationMs
        queue = playerController.getQueue()
        currentQueueIndex = playerController.getCurrentQueueIndex()
        isShuffled = snapshot.isShuffled
        repeatMode = snapshot.repeatMode
        currentSongId = newSongId
        currentAlbumId = snapshot.currentAlbumId
        currentArtistId = snapshot.currentArtistId
        currentAlbumName = snapshot.currentAlbumName
        isFavorite = newSongId.flatMap { favoriteCache[$0] } ?? false
        playbackSpeed = snapshot.playbackSpeed
        codec = snapshot.codec
        bitRate = snapshot.bitRate
        sampleRate = snapshot.sampleRate

        // Mark previous song as played on track change
        if let previousSongId, let newSongId, newSongId != previousSongId {
            markSongAsPlayed(previousSongId)
        }

        // Fetch lyrics for new song
        if let newSongId, !newSongId.isEmpty, newSongId != lastFetchedSongId {
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

                let lrcContent = await getLyrics(
                    serverUrl: serverUrl,
                    token: token,
                    itemId: itemId,
                    artist: artist,
                    title: title
                )
                let parsed = await LyricsParser.parse(lrcContent)

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

        isFavoriteLoading = true

        Task.detached { [self, isFav = self.isFavorite] in
            do {
                let newState = try await AureliaCore.toggleFavorite(
                    serverUrl: serverUrl,
                    token: token,
                    userId: userId,
                    itemId: songId,
                    isFavorite: isFav
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
