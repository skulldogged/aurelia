import Foundation
import Observation
import os
import AureliaCore

@Observable
final class PlaylistViewModel: @unchecked Sendable {
    // Playlists list state
    var isLoading = true
    var playlists: [Playlist] = []
    var error: String?
    var isCreating = false
    var isDeleting = false

    // Playlist detail state
    var detailIsLoading = true
    var detailPlaylist: Playlist?
    var detailSongs: [Song] = []
    var detailError: String?

    private let sessionStore = SessionStore.shared
    private let logger = Logger(subsystem: "com.aurelia.app", category: "PlaylistViewModel")

    func loadPlaylists() {
        guard let creds = sessionStore.getCredentials(),
              !creds.serverUrl.isEmpty, !creds.token.isEmpty, !creds.userId.isEmpty else {
            error = "Missing session data"
            isLoading = false
            return
        }

        isLoading = true
        error = nil

        Task.detached { [self] in
            do {
                let result = try getPlaylists(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId
                )
                await MainActor.run {
                    self.isLoading = false
                    self.playlists = result
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        self.isLoading = false
                        self.error = error.localizedDescription
                    }
                }
            }
        }
    }

    func createPlaylist(name: String, songIds: [String]? = nil) {
        guard let creds = sessionStore.getCredentials() else { return }

        isCreating = true

        Task.detached { [self] in
            do {
                let data = PlaylistCreateData(
                    name: name,
                    ids: songIds,
                    userId: creds.userId,
                    isPublic: false
                )
                let newPlaylist = try AureliaCore.createPlaylist(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    data: data
                )
                await MainActor.run {
                    self.isCreating = false
                    self.playlists.append(newPlaylist)
                }
            } catch {
                await MainActor.run {
                    self.isCreating = false
                    self.error = "Failed to create playlist"
                }
            }
        }
    }

    func deletePlaylist(_ playlistId: String) {
        guard let creds = sessionStore.getCredentials() else { return }

        isDeleting = true

        Task.detached { [self] in
            do {
                try AureliaCore.deletePlaylist(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    playlistId: playlistId
                )
                await MainActor.run {
                    self.isDeleting = false
                    self.playlists.removeAll { $0.id == playlistId }
                }
            } catch {
                await MainActor.run {
                    self.isDeleting = false
                    self.error = "Failed to delete playlist"
                }
            }
        }
    }

    func loadPlaylistDetail(playlistId: String) {
        guard let creds = sessionStore.getCredentials() else {
            detailError = "Missing session data"
            detailIsLoading = false
            return
        }

        detailIsLoading = true
        detailError = nil

        Task.detached { [self] in
            do {
                let songs = try getPlaylistItems(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    playlistId: playlistId
                )
                let playlist = await self.playlists.first { $0.id == playlistId }
                await MainActor.run {
                    self.detailIsLoading = false
                    self.detailPlaylist = playlist
                    self.detailSongs = songs
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        self.detailIsLoading = false
                        self.detailError = error.localizedDescription
                    }
                }
            }
        }
    }

    func addSongsToPlaylist(playlistId: String, songIds: [String]) {
        guard let creds = sessionStore.getCredentials() else { return }

        Task.detached { [self] in
            do {
                try addPlaylistItems(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    playlistId: playlistId,
                    itemIds: songIds
                )
                await MainActor.run {
                    self.loadPlaylistDetail(playlistId: playlistId)
                    self.loadPlaylists()
                }
            } catch {
                self.logger.error("Failed to add songs to playlist: \(error)")
                await MainActor.run {
                    self.error = "Failed to add songs to playlist"
                }
            }
        }
    }

    func playPlaylist(startIndex: Int = 0, playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        guard !detailSongs.isEmpty else { return }
        playerController.setQueue(detailSongs, serverUrl: serverUrl, token: token, startIndex: startIndex)
    }

    func shufflePlaylist(playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        let shuffled = detailSongs.shuffled()
        guard !shuffled.isEmpty else { return }
        playerController.setQueue(shuffled, serverUrl: serverUrl, token: token)
    }

    func clearError() {
        error = nil
        detailError = nil
    }
}
