import AureliaCore
import Foundation
import Observation
import UIKit

enum LoginProviderSelection: String, CaseIterable, Identifiable {
    case auto
    case jellyfin

    var id: String { rawValue }

    var title: String {
        switch self {
        case .auto:
            return "Auto"
        case .jellyfin:
            return "Jellyfin"
        }
    }
}

@Observable
final class LoginViewModel: @unchecked Sendable {
    var serverUrl = ""
    var username = ""
    var password = ""
    var providerSelection: LoginProviderSelection = .auto
    var detectedProvider: BackendProvider?
    var isDetectingProvider = false
    var isSubmitting = false
    var error: String?

    // Set after successful login
    var token: String?
    var userId: String?

    private let sessionStore = SessionStore.shared

    func detectProviderNow() {
        guard let normalizedServerUrl = ServerURLNormalizer.normalizeForServer(raw: serverUrl),
              ServerURLNormalizer.isValidServerURL(normalizedServerUrl)
        else {
            error = "Enter a valid server URL"
            return
        }

        isDetectingProvider = true
        error = nil
        serverUrl = normalizedServerUrl

        Task.detached { [serverUrl = normalizedServerUrl] in
            do {
                let provider = try await detectProvider(serverUrl: serverUrl)
                await MainActor.run { [self] in
                    isDetectingProvider = false
                    detectedProvider = provider
                }
            } catch {
                await MainActor.run { [self] in
                    isDetectingProvider = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    func submit() {
        guard !serverUrl.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
              !username.trimmingCharacters(in: .whitespaces).isEmpty,
              !password.isEmpty
        else {
            error = "All fields are required"
            return
        }

        guard let normalizedServerUrl = ServerURLNormalizer.normalizeForServer(raw: serverUrl),
              ServerURLNormalizer.isValidServerURL(normalizedServerUrl)
        else {
            error = "Enter a valid server URL"
            return
        }

        isSubmitting = true
        error = nil
        serverUrl = normalizedServerUrl
        let providerSelection = self.providerSelection
        let detectedProvider = self.detectedProvider

        let deviceId = UIDevice.current.identifierForVendor?.uuidString ?? "aurelia-ios-\(UUID().uuidString)"

        Task.detached {
            [serverUrl = normalizedServerUrl, username = self.username, password = self.password, deviceId, providerSelection, detectedProvider] in
            do {
                let resolvedProvider: BackendProvider = .jellyfin

                let response = try await authenticate(
                    request: AuthRequest(
                        provider: resolvedProvider,
                        serverUrl: serverUrl,
                        username: username,
                        password: password,
                        deviceId: deviceId
                    )
                )
                await MainActor.run {
                    let sessionStore = SessionStore.shared
                    sessionStore.save(
                        serverUrl: serverUrl,
                        userId: response.userId,
                        token: response.token,
                        username: username,
                        provider: resolvedProvider
                    )
                }

                await MainActor.run { [self] in
                    isSubmitting = false
                    token = response.token
                    userId = response.userId
                    self.detectedProvider = resolvedProvider
                }
            } catch {
                await MainActor.run { [self] in
                    isSubmitting = false
                    self.error = error.localizedDescription
                }
            }
        }
    }
}
