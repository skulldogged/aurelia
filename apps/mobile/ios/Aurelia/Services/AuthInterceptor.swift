import Foundation
import os

/// Detects 401/unauthorized errors from Rust API calls and triggers automatic logout.
final class AuthInterceptor: @unchecked Sendable {
    static let shared = AuthInterceptor()

    private let logger = Logger(subsystem: "com.aurelia.app", category: "AuthInterceptor")
    private var logoutCallback: (() -> Void)?

    private init() {}

    func setLogoutCallback(_ callback: @escaping () -> Void) {
        logoutCallback = callback
    }

    func clearLogoutCallback() {
        logoutCallback = nil
    }

    func isUnauthorizedError(_ error: Error) -> Bool {
        isUnauthorizedError(error.localizedDescription)
    }

    func isUnauthorizedError(_ message: String?) -> Bool {
        guard let message = message?.lowercased() else { return false }
        return message.contains("unauthorized")
            || message.contains("401")
            || message.contains("authentication")
            || message.contains("not authenticated")
    }

    /// Returns true if the error was an auth error and logout was triggered.
    @discardableResult
    func handlePotentialAuthError(_ error: Error) -> Bool {
        handlePotentialAuthError(error.localizedDescription)
    }

    @discardableResult
    func handlePotentialAuthError(_ message: String?) -> Bool {
        if isUnauthorizedError(message) {
            logger.warning("Unauthorized error detected, triggering logout")
            logoutCallback?()
            return true
        }
        return false
    }
}
