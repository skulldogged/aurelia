import Foundation
import Observation
import UIKit
import AureliaCore

@Observable
final class LoginViewModel: @unchecked Sendable {
    var serverUrl = ""
    var username = ""
    var password = ""
    var isSubmitting = false
    var error: String?

    // Set after successful login
    var token: String?
    var userId: String?

    private let sessionStore = SessionStore.shared

    func submit() {
        guard !serverUrl.trimmingCharacters(in: .whitespaces).isEmpty,
              !username.trimmingCharacters(in: .whitespaces).isEmpty,
              !password.isEmpty else {
            error = "All fields are required"
            return
        }

        isSubmitting = true
        error = nil

        let deviceId = UIDevice.current.identifierForVendor?.uuidString ?? UUID().uuidString

        Task.detached { [serverUrl = self.serverUrl, username = self.username, password = self.password, deviceId = deviceId] in
            do {
                let response = try await authenticate(
                    serverUrl: serverUrl,
                    username: username,
                    password: password,
                    deviceId: deviceId
                )
                let sessionStore = await SessionStore.shared
                await sessionStore
                    .save(serverUrl: serverUrl, userId: response.userId, token: response.token, username: username)

                await MainActor.run { [self] in
                    self.isSubmitting = false
                    self.token = response.token
                    self.userId = response.userId
                }
            } catch {
                await MainActor.run { [self] in
                    self.isSubmitting = false
                    self.error = error.localizedDescription
                }
            }
        }
    }
}
