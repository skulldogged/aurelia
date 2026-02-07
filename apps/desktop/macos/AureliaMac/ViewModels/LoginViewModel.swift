import AureliaCore
import Foundation

@MainActor
final class LoginViewModel: ObservableObject {
    @Published var serverUrl = ""
    @Published var username = ""
    @Published var password = ""
    @Published var isSubmitting = false
    @Published var error: String?

    private let sessionStore = SessionStore.shared

    func submit(onSuccess: @escaping () -> Void) {
        guard !serverUrl.trimmingCharacters(in: .whitespaces).isEmpty,
              !username.trimmingCharacters(in: .whitespaces).isEmpty,
              !password.isEmpty else {
            error = "All fields are required"
            return
        }

        isSubmitting = true
        error = nil

        Task {
            do {
                let response = try await authenticate(
                    serverUrl: serverUrl,
                    username: username,
                    password: password,
                    deviceId: "aurelia-macos-\(UUID().uuidString)"
                )
                sessionStore.save(serverUrl: serverUrl, userId: response.userId, token: response.token, username: username)
                isSubmitting = false
                onSuccess()
            } catch {
                isSubmitting = false
                self.error = error.localizedDescription
            }
        }
    }
}
