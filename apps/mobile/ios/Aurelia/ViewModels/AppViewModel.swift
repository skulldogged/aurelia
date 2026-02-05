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
        isLoading = true
        Task.detached { [sessionStore] in
            let isValid = await sessionStore.hasValidSession
            await MainActor.run {
                self.isLoggedIn = isValid
                self.isLoading = false
            }
        }
    }

    func logout() {
        sessionStore.clear()
        isLoggedIn = false
    }
}
