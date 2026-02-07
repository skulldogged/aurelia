import AureliaCore
import Foundation

@MainActor
final class LibraryViewModel: ObservableObject {
    @Published var isLoading = false
    @Published var songs: [Song] = []
    @Published var error: String?

    private let sessionStore = SessionStore.shared

    func loadLibrary() {
        guard let creds = sessionStore.getCredentials(),
              !creds.serverUrl.isEmpty, !creds.token.isEmpty, !creds.userId.isEmpty else {
            error = "Missing session data"
            return
        }

        isLoading = true
        error = nil

        let appDataDir = sessionStore.getAppDataDir() ?? ""
        let shouldRefresh = sessionStore.shouldRefreshLibrary()

        Task {
            var loadedCache = false
            if !appDataDir.isEmpty {
                do {
                    let cached = try loadCachedSongs(appDataDir: appDataDir)
                    if !cached.isEmpty {
                        loadedCache = true
                        songs = cached
                        isLoading = false
                    }
                } catch {}
            }

            if loadedCache && !shouldRefresh {
                return
            }

            do {
                let freshSongs = try await fetchSongs(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    appDataDir: appDataDir
                )
                songs = freshSongs
                isLoading = false
                sessionStore.markLibraryRefreshed()
            } catch {
                if !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    isLoading = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    func playFromList(_ songId: String, playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        guard let startIndex = songs.firstIndex(where: { $0.id == songId }) else { return }
        playerController.setQueue(songs, serverUrl: serverUrl, token: token, startIndex: startIndex)
    }
}
