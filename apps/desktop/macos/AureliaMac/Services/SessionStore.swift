import AureliaCore
import Foundation

@MainActor
final class SessionStore {
    static let shared = SessionStore()

    private let directoryName = "aurelia"
    private let libraryRefreshKey = "lastLibraryRefresh"
    private var cachedCredentials: Credentials?

    private init() {}

    func getAppDataDir() -> String? {
        guard let supportDirectory = FileManager.default.urls(
            for: .applicationSupportDirectory,
            in: .userDomainMask
        ).first else {
            return nil
        }

        let appDirectory = supportDirectory.appendingPathComponent(directoryName, isDirectory: true)
        if !FileManager.default.fileExists(atPath: appDirectory.path) {
            do {
                try FileManager.default.createDirectory(at: appDirectory, withIntermediateDirectories: true)
            } catch {
                return nil
            }
        }

        return appDirectory.path
    }

    func save(serverUrl: String, userId: String, token: String, username: String = "") {
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return }
        do {
            let credentials = Credentials(serverUrl: serverUrl, username: username, token: token, userId: userId)
            try saveCredentials(appDataDir: appDataDir, credentials: credentials)
            cachedCredentials = credentials
        } catch {}
    }

    func getCredentials() -> Credentials? {
        if let cachedCredentials { return cachedCredentials }
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return nil }
        do {
            let credentials = try loadCredentials(appDataDir: appDataDir)
            cachedCredentials = credentials
            return credentials
        } catch {
            return nil
        }
    }

    func clear() {
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return }
        do {
            try clearCredentials(appDataDir: appDataDir)
            cachedCredentials = nil
        } catch {}
    }

    var serverUrl: String? { getCredentials()?.serverUrl }
    var userId: String? { getCredentials()?.userId }
    var token: String? { getCredentials()?.token }

    var hasValidSession: Bool {
        guard let creds = getCredentials() else { return false }
        return !creds.serverUrl.isEmpty && !creds.userId.isEmpty && !creds.token.isEmpty
    }

    func markLibraryRefreshed() {
        UserDefaults.standard.set(Date(), forKey: libraryRefreshKey)
    }

    func lastLibraryRefreshDate() -> Date? {
        UserDefaults.standard.object(forKey: libraryRefreshKey) as? Date
    }

    func shouldRefreshLibrary(maxAge: TimeInterval = 6 * 60 * 60) -> Bool {
        guard let refresh = lastLibraryRefreshDate() else { return true }
        return Date().timeIntervalSince(refresh) > maxAge
    }
}
