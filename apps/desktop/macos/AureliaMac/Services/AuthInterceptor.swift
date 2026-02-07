import Foundation

@MainActor
final class AuthInterceptor {
    static let shared = AuthInterceptor()

    private var logoutCallback: (() -> Void)?

    private init() {}

    func setLogoutCallback(_ callback: @escaping () -> Void) {
        logoutCallback = callback
    }

    @discardableResult
    func handlePotentialAuthError(_ error: Error) -> Bool {
        let message = error.localizedDescription.lowercased()
        if message.contains("unauthorized") || message.contains("401") || message.contains("not authenticated") {
            logoutCallback?()
            return true
        }
        return false
    }
}
