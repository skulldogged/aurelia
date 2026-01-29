import Foundation
import Observation

@Observable
final class AppViewModel: @unchecked Sendable {
    var isLoading = true
    var isLoggedIn = false

    private let sessionStore = SessionStore.shared

    init() {
        checkSession()
    }

    func checkSession() {
        isLoggedIn = sessionStore.hasValidSession
        isLoading = false
    }

    func logout() {
        sessionStore.clear()
        isLoggedIn = false
    }
}
