import Foundation
import Observation
import os
import AureliaCore

@Observable
final class SettingsViewModel: @unchecked Sendable {
    var isSyncing = false
    var isClearing = false
    var syncSuccess: Bool?
    var clearSuccess: Bool?
    var error: String?
    var lastSyncTime: String?

    private let sessionStore = SessionStore.shared
    private let logger = Logger(subsystem: "com.aurelia.app", category: "SettingsViewModel")

    init() {
        loadSyncState()
    }

    private func loadSyncState() {
        guard let appDataDir = sessionStore.getAppDataDir(), !appDataDir.isEmpty else { return }

        Task.detached { [self] in
            do {
                let state = try getSyncState(appDataDir: appDataDir)
                await MainActor.run {
                    self.lastSyncTime = state.lastSyncTime
                }
            } catch {
                // Ignore errors loading sync state
            }
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

        Task.detached { [self] in
            do {
                let _ = try await syncSongsOnly(
                    serverUrl: serverUrl,
                    token: token,
                    userId: userId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.loadSyncState()
                    self.isSyncing = false
                    self.syncSuccess = true
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        self.isSyncing = false
                        self.syncSuccess = false
                        self.error = error.localizedDescription
                    }
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

        Task.detached { [self] in
            do {
                try clearCache(appDataDir: appDataDir)
                await MainActor.run {
                    self.isClearing = false
                    self.clearSuccess = true
                    self.lastSyncTime = nil
                }
            } catch {
                await MainActor.run {
                    self.isClearing = false
                    self.clearSuccess = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    func clearMessages() {
        syncSuccess = nil
        clearSuccess = nil
        error = nil
    }
}
