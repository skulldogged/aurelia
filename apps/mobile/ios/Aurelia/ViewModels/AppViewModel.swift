import Foundation
import Observation

@Observable
final class AppViewModel: @unchecked Sendable {
    var isLoading = true
    var isLoggedIn = false
    var sessionVersion = 0

    private let sessionStore = SessionStore.shared

    init() {
        checkSession()
    }

    func checkSession() {
        isLoading = true
        Task.detached { [sessionStore] in
            let isValid = await sessionStore.hasValidSessionAsync()
            await MainActor.run {
                self.isLoggedIn = isValid
                self.isLoading = false
            }
        }
    }

    func logout() {
        sessionStore.clear()
        sessionVersion += 1
        isLoggedIn = false
    }

    @discardableResult
    func switchProfile(_ profileId: String) -> Bool {
        guard sessionStore.switchProfile(profileId) else { return false }
        refreshAfterSessionChange()
        return true
    }

    func refreshAfterSessionChange() {
        sessionVersion += 1
        checkSession()
    }
}
