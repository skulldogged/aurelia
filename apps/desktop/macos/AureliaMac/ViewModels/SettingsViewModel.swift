import AureliaCore
import Foundation

@MainActor
final class SettingsViewModel: ObservableObject {
    @Published var isSyncing = false
    @Published var isClearing = false
    @Published var syncSuccess: Bool?
    @Published var clearSuccess: Bool?
    @Published var error: String?
    @Published var lastSyncTime: String?

    private let sessionStore = SessionStore.shared

    init() {
        loadSyncState()
    }

    private func loadSyncState() {
        guard let appDataDir = sessionStore.getAppDataDir(), !appDataDir.isEmpty else { return }

        Task {
            do {
                let state = try getSyncState(appDataDir: appDataDir)
                lastSyncTime = state.lastSyncTime
            } catch {}
        }
    }

    func syncLibrary() {
        guard let serverUrl = sessionStore.serverUrl,
              let userId = sessionStore.userId,
              let token = sessionStore.token else {
            error = "Missing session data"
            return
        }

        let appDataDir = sessionStore.getAppDataDir() ?? ""

        isSyncing = true
        error = nil
        syncSuccess = nil

        Task {
            do {
                _ = try await syncSongsOnly(
                    serverUrl: serverUrl,
                    token: token,
                    userId: userId,
                    appDataDir: appDataDir
                )
                loadSyncState()
                isSyncing = false
                syncSuccess = true
                sessionStore.markLibraryRefreshed()
            } catch {
                if !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    isSyncing = false
                    syncSuccess = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    func clearLocalCache() {
        guard let appDataDir = sessionStore.getAppDataDir(), !appDataDir.isEmpty else {
            error = "No cache directory"
            return
        }

        isClearing = true
        error = nil
        clearSuccess = nil

        Task {
            do {
                try clearCache(appDataDir: appDataDir)
                isClearing = false
                clearSuccess = true
                lastSyncTime = nil
            } catch {
                isClearing = false
                clearSuccess = false
                self.error = error.localizedDescription
            }
        }
    }
}
