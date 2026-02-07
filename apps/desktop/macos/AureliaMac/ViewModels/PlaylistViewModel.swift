import AureliaCore
import Foundation

@MainActor
final class PlaylistViewModel: ObservableObject {
    @Published var isLoading = true
    @Published var playlists: [Playlist] = []
    @Published var error: String?
    @Published var isCreating = false
    @Published var isDeleting = false

    @Published var detailIsLoading = true
    @Published var detailPlaylist: Playlist?
    @Published var detailSongs: [Song] = []
    @Published var detailError: String?

    private let sessionStore = SessionStore.shared

    func loadPlaylists() {
        guard let creds = sessionStore.getCredentials(),
              !creds.serverUrl.isEmpty, !creds.token.isEmpty, !creds.userId.isEmpty else {
            error = "Missing session data"
            isLoading = false
            return
        }

        isLoading = true
        error = nil

        Task {
            do {
                playlists = try await getPlaylists(serverUrl: creds.serverUrl, token: creds.token, userId: creds.userId)
                isLoading = false
            } catch {
                if !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    isLoading = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    func createPlaylist(name: String, songIds: [String]? = nil) {
        guard let creds = sessionStore.getCredentials() else { return }
        isCreating = true

        Task {
            do {
                let data = PlaylistCreateData(name: name, ids: songIds, userId: creds.userId, isPublic: false)
                let newPlaylist = try await AureliaCore.createPlaylist(serverUrl: creds.serverUrl, token: creds.token, data: data)
                playlists.append(newPlaylist)
                isCreating = false
            } catch {
                isCreating = false
                self.error = "Failed to create playlist"
            }
        }
    }

    func deletePlaylist(_ playlistId: String) {
        guard let creds = sessionStore.getCredentials() else { return }
        isDeleting = true

        Task {
            do {
                try await AureliaCore.deletePlaylist(serverUrl: creds.serverUrl, token: creds.token, playlistId: playlistId)
                playlists.removeAll { $0.id == playlistId }
                isDeleting = false
            } catch {
                isDeleting = false
                self.error = "Failed to delete playlist"
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

        Task {
            do {
                detailSongs = try await getPlaylistItems(serverUrl: creds.serverUrl, token: creds.token, playlistId: playlistId)
                detailPlaylist = playlists.first { $0.id == playlistId }
                detailIsLoading = false
            } catch {
                if !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    detailIsLoading = false
                    detailError = error.localizedDescription
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
}
