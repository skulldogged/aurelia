import Foundation
import os
import AureliaCore

/// Manages user session credentials, backed by the Rust redb database via UniFFI.
@Observable
final class SessionStore: @unchecked Sendable {
    static let shared = SessionStore()

    private let logger = Logger(subsystem: "com.aurelia.app", category: "SessionStore")
    private let libraryRefreshKey = "lastLibraryRefresh"
    private let ioQueue = DispatchQueue(label: "com.aurelia.sessionstore.io", qos: .userInitiated)
    private var cachedCredentials: Credentials?

    private init() {
        // AppDataDir is now computed dynamically to handle iOS container path changes
    }

    // MARK: - App Data Directory

    func getAppDataDir() -> String? {
        guard let supportDir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        let aureliaDir = supportDir.appendingPathComponent("aurelia", isDirectory: true)
        
        // Ensure directory exists
        if !FileManager.default.fileExists(atPath: aureliaDir.path) {
            do {
                try FileManager.default.createDirectory(at: aureliaDir, withIntermediateDirectories: true, attributes: nil)
            } catch {
                logger.error("Failed to create app data directory: \(error)")
                return nil
            }
        }
        
        return aureliaDir.path
    }

    // MARK: - Credentials (via Rust redb)

    func save(serverUrl: String, userId: String, token: String, username: String = "") {
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else {
            logger.warning("Cannot save credentials: appDataDir not set")
            return
        }
        do {
            let credentials = Credentials(
                serverUrl: serverUrl,
                username: username,
                token: token,
                userId: userId
            )
            try saveCredentials(appDataDir: appDataDir, credentials: credentials)
            cachedCredentials = credentials
        } catch {
            logger.error("Failed to save credentials: \(error)")
        }
    }

    func getCredentials() -> Credentials? {
        if let cachedCredentials { return cachedCredentials }
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return nil }
        do {
            let credentials = try loadCredentials(appDataDir: appDataDir)
            cachedCredentials = credentials
            return credentials
        } catch {
            logger.error("Failed to load credentials: \(error)")
            return nil
        }
    }

    func getCredentialsAsync() async -> Credentials? {
        if let cachedCredentials { return cachedCredentials }
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return nil }

        let credentials = await withCheckedContinuation { continuation in
            ioQueue.async {
                do {
                    continuation.resume(returning: try loadCredentials(appDataDir: appDataDir))
                } catch {
                    continuation.resume(returning: nil)
                }
            }
        }
        cachedCredentials = credentials
        return credentials
    }

    func getSyncState() -> SyncState? {
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return nil }
        do {
            return try AureliaCore.getSyncState(appDataDir: appDataDir)
        } catch {
            logger.error("Failed to load sync state: \(error)")
            return nil
        }
    }

    func lastSyncDate() -> Date? {
        guard let syncState = getSyncState(), !syncState.lastSyncTime.isEmpty else { return nil }
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter.date(from: syncState.lastSyncTime) ?? ISO8601DateFormatter().date(from: syncState.lastSyncTime)
    }

    func markLibraryRefreshed() {
        UserDefaults.standard.set(Date(), forKey: libraryRefreshKey)
    }

    func lastLibraryRefreshDate() -> Date? {
        UserDefaults.standard.object(forKey: libraryRefreshKey) as? Date
    }

    func shouldRefreshLibrary(maxAge: TimeInterval = 6 * 60 * 60) -> Bool {
        let lastSync = lastSyncDate()
        let lastRefresh = lastLibraryRefreshDate()
        let referenceDate = [lastSync, lastRefresh].compactMap { $0 }.max()
        guard let referenceDate else { return true }
        return Date().timeIntervalSince(referenceDate) > maxAge
    }

    func shouldRefreshLibraryAsync(maxAge: TimeInterval = 6 * 60 * 60) async -> Bool {
        let lastRefresh = lastLibraryRefreshDate()
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else {
            return true
        }

        let lastSync: Date? = await withCheckedContinuation { continuation in
            ioQueue.async {
                do {
                    let syncState = try AureliaCore.getSyncState(appDataDir: appDataDir)
                    guard !syncState.lastSyncTime.isEmpty else {
                        continuation.resume(returning: nil)
                        return
                    }
                    let formatter = ISO8601DateFormatter()
                    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
                    let parsed = formatter.date(from: syncState.lastSyncTime)
                        ?? ISO8601DateFormatter().date(from: syncState.lastSyncTime)
                    continuation.resume(returning: parsed)
                } catch {
                    continuation.resume(returning: nil)
                }
            }
        }

        let referenceDate = [lastSync, lastRefresh].compactMap { $0 }.max()
        guard let referenceDate else { return true }
        return Date().timeIntervalSince(referenceDate) > maxAge
    }

    var serverUrl: String? { getCredentials()?.serverUrl }
    var userId: String? { getCredentials()?.userId }
    var token: String? { getCredentials()?.token }

    func clear() {
        if let appDataDir = getAppDataDir(), !appDataDir.isEmpty {
            do {
                try clearCredentials(appDataDir: appDataDir)
                cachedCredentials = nil
            } catch {
                logger.error("Failed to clear credentials: \(error)")
            }
        }
    }

    /// Returns true if a valid session exists with all required fields.
    var hasValidSession: Bool {
        guard let creds = getCredentials() else { return false }
        return !creds.serverUrl.isEmpty && !creds.userId.isEmpty && !creds.token.isEmpty
    }

    func hasValidSessionAsync() async -> Bool {
        guard let creds = await getCredentialsAsync() else { return false }
        return !creds.serverUrl.isEmpty && !creds.userId.isEmpty && !creds.token.isEmpty
    }
}
