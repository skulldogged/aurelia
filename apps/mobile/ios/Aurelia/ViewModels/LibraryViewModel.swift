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

        Task.detached { [self] in
            // Load cached first
            if !appDataDir.isEmpty {
                do {
                    let cached = try loadCachedSongs(appDataDir: appDataDir)
                    if !cached.isEmpty {
                        await MainActor.run {
                            self.songs = cached
                            self.isLoading = false
                        }
                    }
                } catch {
                    self.logger.warning("Failed to load cached songs: \(error)")
                }
            }

            // Fetch fresh
            do {
                let freshSongs = try await fetchSongs(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.songs = freshSongs
                    self.isLoading = false
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
