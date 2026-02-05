import Foundation
import Observation
import os
import AureliaCore

@Observable
final class LibraryViewModel: @unchecked Sendable {
    var isLoading = false
    var songs: [Song] = []
    var error: String?

    private let sessionStore = SessionStore.shared
    private let logger = Logger(subsystem: "com.aurelia.app", category: "LibraryViewModel")

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

        Task.detached { [self] in
            // Load cached first
            var loadedCache = false
            if !appDataDir.isEmpty {
                do {
                    let cached = try loadCachedSongs(appDataDir: appDataDir)
                    if !cached.isEmpty {
                        loadedCache = true
                        await MainActor.run {
                            self.songs = cached
                            self.isLoading = false
                        }
                    }
                } catch {
                    self.logger.warning("Failed to load cached songs: \(error)")
                }
            }

            if loadedCache && !shouldRefresh {
                return
            }

            // Fetch fresh
            do {
                let freshSongs = try fetchSongs(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.songs = freshSongs
                    self.isLoading = false
                    self.sessionStore.markLibraryRefreshed()
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

    func playFromList(_ songId: String, playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        guard let startIndex = songs.firstIndex(where: { $0.id == songId }) else { return }
        playerController.setQueue(songs, serverUrl: serverUrl, token: token, startIndex: startIndex)
    }
}
