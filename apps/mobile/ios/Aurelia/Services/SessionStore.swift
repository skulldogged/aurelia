import AureliaCore
import Foundation
import os

private struct StoredSessionProfile: Codable {
    let id: String
    let provider: String
    let serverUrl: String
    let username: String
    let token: String
    let userId: String
    let updatedAt: TimeInterval
}

struct SessionProfile: Identifiable, Hashable {
    let id: String
    let label: String
    let provider: BackendProvider
    let serverUrl: String
    let userId: String
    let username: String
}

/// Manages user session credentials, backed by the Rust redb database via UniFFI.
@Observable
final class SessionStore: @unchecked Sendable {
    static let shared = SessionStore()

    private let logger = Logger(subsystem: "com.aurelia.app", category: "SessionStore")
    private let libraryRefreshKey = "lastLibraryRefresh"
    private let lyricsServerUrlKey = "lyricsServerUrl"
    private let activeProfileIdKey = "active_profile_id"
    private let profilesKey = "saved_profiles_json"
    private let ioQueue = DispatchQueue(label: "com.aurelia.sessionstore.io", qos: .userInitiated)
    private var cachedCredentials: Credentials?

    private init() {
        // AppDataDir is now computed dynamically to handle iOS container path changes
    }

    // MARK: - App Data Directory

    func getBaseAppDataDir() -> String? {
        guard let supportDir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        let aureliaDir = supportDir.appendingPathComponent("aurelia", isDirectory: true)

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

    func getAppDataDir() -> String? {
        guard let baseAppDataDir = getBaseAppDataDir() else { return nil }
        guard let activeProfileId = getActiveProfileId(), !activeProfileId.isEmpty else { return baseAppDataDir }
        return resolveProfileAppDataDir(baseAppDataDir: baseAppDataDir, profileId: activeProfileId)
    }

    // MARK: - Credentials (via Rust redb)

    func save(
        serverUrl: String,
        userId: String,
        token: String,
        username: String = "",
        provider: BackendProvider = .jellyfin
    ) {
        guard let baseAppDataDir = getBaseAppDataDir(), !baseAppDataDir.isEmpty else {
            logger.warning("Cannot save credentials: appDataDir not set")
            return
        }

        do {
            let credentials = Credentials(
                provider: provider,
                serverUrl: serverUrl,
                username: username,
                token: token,
                userId: userId
            )
            let profileId = upsertProfile(credentials)
            setActiveProfileId(profileId)
            let profileAppDataDir = resolveProfileAppDataDir(baseAppDataDir: baseAppDataDir, profileId: profileId)
            try saveCredentials(appDataDir: profileAppDataDir, credentials: credentials)
            cachedCredentials = credentials
        } catch {
            logger.error("Failed to save credentials: \(error)")
        }
    }

    func getCredentials() -> Credentials? {
        if let cachedCredentials { return cachedCredentials }
        guard let baseAppDataDir = getBaseAppDataDir(), !baseAppDataDir.isEmpty else { return nil }

        bootstrapActiveProfileFromLegacyCredentials(baseAppDataDir: baseAppDataDir)

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
        guard let baseAppDataDir = getBaseAppDataDir(), !baseAppDataDir.isEmpty else { return nil }

        bootstrapActiveProfileFromLegacyCredentials(baseAppDataDir: baseAppDataDir)

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

    func getProfiles() -> [SessionProfile] {
        guard let baseAppDataDir = getBaseAppDataDir(), !baseAppDataDir.isEmpty else { return [] }

        bootstrapActiveProfileFromLegacyCredentials(baseAppDataDir: baseAppDataDir)

        return loadStoredProfiles()
            .sorted { $0.updatedAt > $1.updatedAt }
            .compactMap { storedProfile in
                guard let provider = storedProfile.provider.backendProvider else { return nil }
                let username = storedProfile.username.isEmpty ? storedProfile.userId : storedProfile.username
                return SessionProfile(
                    id: storedProfile.id,
                    label: "\(username) @ \(storedProfile.serverUrl) (\(provider.storageValue))",
                    provider: provider,
                    serverUrl: storedProfile.serverUrl,
                    userId: storedProfile.userId,
                    username: username
                )
            }
    }

    func getActiveProfileId() -> String? {
        UserDefaults.standard.string(forKey: activeProfileIdKey)
    }

    @discardableResult
    func switchProfile(_ profileId: String) -> Bool {
        guard let baseAppDataDir = getBaseAppDataDir(), !baseAppDataDir.isEmpty else { return false }
        guard let storedProfile = loadStoredProfiles().first(where: { $0.id == profileId }),
              let credentials = storedProfile.toCredentials()
        else {
            return false
        }

        do {
            setActiveProfileId(profileId)
            let profileAppDataDir = resolveProfileAppDataDir(baseAppDataDir: baseAppDataDir, profileId: profileId)
            try saveCredentials(appDataDir: profileAppDataDir, credentials: credentials)
            cachedCredentials = credentials
            return true
        } catch {
            logger.error("Failed to switch profile: \(error)")
            return false
        }
    }

    @discardableResult
    func removeProfile(_ profileId: String) -> Bool {
        var profiles = loadStoredProfiles()
        let originalCount = profiles.count
        profiles.removeAll { $0.id == profileId }

        guard profiles.count != originalCount else { return false }

        saveStoredProfiles(profiles)

        if getActiveProfileId() == profileId {
            if let replacement = profiles.max(by: { $0.updatedAt < $1.updatedAt }) {
                _ = switchProfile(replacement.id)
            } else {
                setActiveProfileId(nil)
                cachedCredentials = nil
            }
        }

        return true
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

    var provider: BackendProvider? {
        getCredentials()?.provider
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
            } catch {
                logger.error("Failed to clear credentials: \(error)")
            }
        }
        setActiveProfileId(nil)
        cachedCredentials = nil
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

    private func setActiveProfileId(_ profileId: String?) {
        if let profileId, !profileId.isEmpty {
            UserDefaults.standard.set(profileId, forKey: activeProfileIdKey)
        } else {
            UserDefaults.standard.removeObject(forKey: activeProfileIdKey)
        }
    }

    private func bootstrapActiveProfileFromLegacyCredentials(baseAppDataDir: String) {
        if let activeProfileId = getActiveProfileId(), !activeProfileId.isEmpty {
            return
        }

        guard let legacyCredentials = try? loadCredentials(appDataDir: baseAppDataDir) else {
            return
        }

        let profileId = upsertProfile(legacyCredentials)
        setActiveProfileId(profileId)

        do {
            let profileAppDataDir = resolveProfileAppDataDir(baseAppDataDir: baseAppDataDir, profileId: profileId)
            let existingProfileCredentials = try loadCredentials(appDataDir: profileAppDataDir)
            if existingProfileCredentials == nil {
                try saveCredentials(appDataDir: profileAppDataDir, credentials: legacyCredentials)
            }
        } catch {
            logger.error("Failed to bootstrap legacy profile credentials: \(error)")
        }
    }

    private func buildProfileId(_ credentials: Credentials) -> String {
        let provider = credentials.provider.storageValue
        let username = credentials.username.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        let serverUrl = credentials.serverUrl.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return "\(provider)|\(username)|\(serverUrl)"
    }

    private func profileDirectoryName(profileId: String) -> String {
        let rawSlug = profileId
            .lowercased()
            .replacingOccurrences(of: "[^a-z0-9]+", with: "-", options: .regularExpression)
            .trimmingCharacters(in: CharacterSet(charactersIn: "-"))
        let slug = rawSlug.isEmpty ? "profile" : rawSlug
        let checksum = stableChecksum(profileId)
        return "\(slug)-\(checksum)"
    }

    private func stableChecksum(_ value: String) -> String {
        var hash: UInt64 = 1469598103934665603
        for byte in value.utf8 {
            hash ^= UInt64(byte)
            hash = hash &* 1099511628211
        }
        return String(hash, radix: 16)
    }

    private func resolveProfileAppDataDir(baseAppDataDir: String, profileId: String) -> String {
        let profileDirName = profileDirectoryName(profileId: profileId)
        let profileURL = URL(fileURLWithPath: baseAppDataDir)
            .appendingPathComponent("profiles", isDirectory: true)
            .appendingPathComponent(profileDirName, isDirectory: true)

        if !FileManager.default.fileExists(atPath: profileURL.path) {
            do {
                try FileManager.default.createDirectory(at: profileURL, withIntermediateDirectories: true, attributes: nil)
            } catch {
                logger.error("Failed to create profile directory: \(error)")
            }
        }

        return profileURL.path
    }

    private func upsertProfile(_ credentials: Credentials) -> String {
        let profileId = buildProfileId(credentials)
        var profiles = loadStoredProfiles()

        let updatedProfile = StoredSessionProfile(
            id: profileId,
            provider: credentials.provider.storageValue,
            serverUrl: credentials.serverUrl,
            username: credentials.username,
            token: credentials.token,
            userId: credentials.userId,
            updatedAt: Date().timeIntervalSince1970
        )

        profiles.removeAll { $0.id == profileId }
        profiles.append(updatedProfile)
        saveStoredProfiles(profiles)

        return profileId
    }

    private func loadStoredProfiles() -> [StoredSessionProfile] {
        guard let raw = UserDefaults.standard.string(forKey: profilesKey), !raw.isEmpty else { return [] }
        guard let data = raw.data(using: .utf8) else { return [] }

        do {
            return try JSONDecoder().decode([StoredSessionProfile].self, from: data)
        } catch {
            logger.error("Failed to decode stored profiles: \(error)")
            return []
        }
    }

    private func saveStoredProfiles(_ profiles: [StoredSessionProfile]) {
        do {
            let data = try JSONEncoder().encode(profiles)
            let encoded = String(data: data, encoding: .utf8)
            UserDefaults.standard.set(encoded, forKey: profilesKey)
        } catch {
            logger.error("Failed to encode stored profiles: \(error)")
        }
    }
}

private extension BackendProvider {
    var storageValue: String {
        switch self {
        case .jellyfin:
            return "jellyfin"
        case .navidrome:
            return "navidrome"
        }
    }
}

private extension String {
    var backendProvider: BackendProvider? {
        switch lowercased() {
        case "jellyfin":
            return .jellyfin
        case "navidrome":
            return .navidrome
        default:
            return nil
        }
    }
}

private extension StoredSessionProfile {
    func toCredentials() -> Credentials? {
        guard let provider = provider.backendProvider else { return nil }
        return Credentials(
            provider: provider,
            serverUrl: serverUrl,
            username: username,
            token: token,
            userId: userId
        )
    }
}
