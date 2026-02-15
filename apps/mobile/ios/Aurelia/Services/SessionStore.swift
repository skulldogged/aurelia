import AureliaCore
import Foundation
import os

/// Manages user session credentials, backed by the Rust redb database via UniFFI.
@Observable
final class SessionStore: @unchecked Sendable {
    static let shared = SessionStore()

    private let logger = Logger(subsystem: "com.aurelia.app", category: "SessionStore")
    private let libraryRefreshKey = "lastLibraryRefresh"
    private let lyricsServerUrlKey = "lyricsServerUrl"
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
            guard let credentials = try loadCredentials(appDataDir: appDataDir) else { return nil }
            let normalized = normalizeCredentialsIfNeeded(credentials, appDataDir: appDataDir)
            cachedCredentials = normalized
            return normalized
        } catch {
            logger.error("Failed to load credentials: \(error)")
            return nil
        }
    }

    func getCredentialsAsync() async -> Credentials? {
        if let cachedCredentials { return cachedCredentials }
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return nil }

        let loadedCredentials: Credentials? = await withCheckedContinuation { (continuation: CheckedContinuation<Credentials?, Never>) in
            ioQueue.async {
                do {
                    continuation.resume(returning: try loadCredentials(appDataDir: appDataDir))
                } catch {
                    continuation.resume(returning: nil)
                }
            }
        }

        guard let loadedCredentials else { return nil }
        let normalized = normalizeCredentialsIfNeeded(loadedCredentials, appDataDir: appDataDir)
        cachedCredentials = normalized
        return normalized
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
        let referenceDate = [lastSync, lastRefresh].compactMap(\.self).max()
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

        let referenceDate = [lastSync, lastRefresh].compactMap(\.self).max()
        guard let referenceDate else { return true }
        return Date().timeIntervalSince(referenceDate) > maxAge
    }

    var serverUrl: String? {
        getCredentials()?.serverUrl
    }

    var userId: String? {
        getCredentials()?.userId
    }

    var token: String? {
        getCredentials()?.token
    }

    // MARK: - Lyrics Server URL (for sidecar lyrics from daemon)

    var lyricsServerUrl: String? {
        get { UserDefaults.standard.string(forKey: lyricsServerUrlKey) }
        set { UserDefaults.standard.set(newValue, forKey: lyricsServerUrlKey) }
    }

    // MARK: - EQ Settings

    var eqEnabled: Bool {
        get { UserDefaults.standard.bool(forKey: "eq_enabled") }
        set { UserDefaults.standard.set(newValue, forKey: "eq_enabled") }
    }

    var eqBands: [Float] {
        get {
            if let stored = UserDefaults.standard.string(forKey: "eq_bands") {
                let values = stored.split(separator: ",").compactMap { Float($0) }
                return values.count == 5 ? values : [0, 0, 0, 0, 0]
            }
            return [0, 0, 0, 0, 0]
        }
        set {
            let stored = newValue.map { String($0) }.joined(separator: ",")
            UserDefaults.standard.set(stored, forKey: "eq_bands")
        }
    }

    var eqPreset: String? {
        get { UserDefaults.standard.string(forKey: "eq_preset") }
        set { UserDefaults.standard.set(newValue, forKey: "eq_preset") }
    }

    // MARK: - Visualizer Settings

    var visualizerEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "visualizer_enabled") as? Bool ?? true }
        set { UserDefaults.standard.set(newValue, forKey: "visualizer_enabled") }
    }

    var visualizerStyle: String {
        get { UserDefaults.standard.string(forKey: "visualizer_style") ?? "bars" }
        set { UserDefaults.standard.set(newValue, forKey: "visualizer_style") }
    }

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

    private func normalizeCredentialsIfNeeded(_ credentials: Credentials, appDataDir: String) -> Credentials {
        guard let normalizedServerUrl = ServerURLNormalizer.normalizeForServer(raw: credentials.serverUrl),
              ServerURLNormalizer.isValidServerURL(normalizedServerUrl),
              normalizedServerUrl != credentials.serverUrl
        else {
            return credentials
        }

        var normalized = credentials
        normalized.serverUrl = normalizedServerUrl

        do {
            try saveCredentials(appDataDir: appDataDir, credentials: normalized)
        } catch {
            logger.error("Failed to save normalized credentials: \(error)")
        }

        return normalized
    }
}
