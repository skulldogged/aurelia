import Foundation

@MainActor
final class AppViewModel: ObservableObject {
    @Published var isLoading = true
    @Published var isLoggedIn = false

    private let sessionStore = SessionStore.shared

    init() {
        checkSession()
    }

    func checkSession() {
        isLoading = true
        isLoggedIn = sessionStore.hasValidSession
        isLoading = false
    }

    func logout() {
        sessionStore.clear()
        isLoggedIn = false
    }
}
