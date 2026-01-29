import Foundation
import os
import AureliaCore

/// Manages user session credentials, backed by the Rust redb database via UniFFI.
@Observable
final class SessionStore: @unchecked Sendable {
    static let shared = SessionStore()

    private let logger = Logger(subsystem: "com.aurelia.app", category: "SessionStore")

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
        } catch {
            logger.error("Failed to save credentials: \(error)")
        }
    }

    func getCredentials() -> Credentials? {
        guard let appDataDir = getAppDataDir(), !appDataDir.isEmpty else { return nil }
        do {
            return try loadCredentials(appDataDir: appDataDir)
        } catch {
            logger.error("Failed to load credentials: \(error)")
            return nil
        }
    }

    var serverUrl: String? { getCredentials()?.serverUrl }
    var userId: String? { getCredentials()?.userId }
    var token: String? { getCredentials()?.token }

    func clear() {
        if let appDataDir = getAppDataDir(), !appDataDir.isEmpty {
            do {
                try clearCredentials(appDataDir: appDataDir)
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
}