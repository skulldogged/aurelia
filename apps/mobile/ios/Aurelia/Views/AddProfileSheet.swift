import AureliaCore
import SwiftUI
import UIKit

private enum AddProfileProviderSelection: String, CaseIterable, Identifiable {
    case auto
    case jellyfin
    case navidrome

    var id: String { rawValue }

    var title: String {
        switch self {
        case .auto:
            return "Auto"
        case .jellyfin:
            return "Jellyfin"
        case .navidrome:
            return "Navidrome"
        }
    }
}

struct AddProfileSheet: View {
    @Environment(\.dismiss) private var dismiss

    @State private var serverUrl = ""
    @State private var username = ""
    @State private var password = ""
    @State private var providerSelection: AddProfileProviderSelection = .auto
    @State private var detectedProvider: BackendProvider?
    @State private var isDetectingProvider = false
    @State private var isSubmitting = false
    @State private var error: String?

    let onProfileAdded: () -> Void

    var body: some View {
        NavigationStack {
            Form {
                Section("Server") {
                    TextField("Server URL", text: $serverUrl)
                        .textContentType(.URL)
                        .keyboardType(.URL)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)

                    Picker("Provider", selection: $providerSelection) {
                        ForEach(AddProfileProviderSelection.allCases) { option in
                            Text(option.title).tag(option)
                        }
                    }
                    .pickerStyle(.segmented)

                    if providerSelection == .auto {
                        Button {
                            detectProviderNow()
                        } label: {
                            if isDetectingProvider {
                                ProgressView()
                            } else {
                                Text("Detect Provider")
                            }
                        }
                        .disabled(isSubmitting || serverUrl.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                        Text(detectedProviderText)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Credentials") {
                    TextField("Username", text: $username)
                        .textContentType(.username)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)

                    SecureField("Password", text: $password)
                        .textContentType(.password)
                }

                if let error {
                    Section {
                        Text(error)
                            .font(.callout)
                            .foregroundStyle(.red)
                    }
                }
            }
            .navigationTitle("Add Profile")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        addProfile()
                    } label: {
                        if isSubmitting {
                            ProgressView()
                        } else {
                            Text("Add")
                        }
                    }
                    .disabled(isSubmitting)
                }
            }
        }
    }

    private var detectedProviderText: String {
        switch detectedProvider {
        case .some(.jellyfin):
            return "Detected provider: Jellyfin"
        case .some(.navidrome):
            return "Detected provider: Navidrome"
        case nil:
            return "Detected provider: not detected"
        }
    }

    private func detectProviderNow() {
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
                await MainActor.run {
                    isDetectingProvider = false
                    detectedProvider = provider
                }
            } catch {
                await MainActor.run {
                    isDetectingProvider = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    private func addProfile() {
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
        let username = self.username
        let password = self.password

        let deviceId = UIDevice.current.identifierForVendor?.uuidString ?? "aurelia-ios-\(UUID().uuidString)"

        Task.detached {
            [serverUrl = normalizedServerUrl, username, password, deviceId, providerSelection, detectedProvider] in
            do {
                let resolvedProvider: BackendProvider = switch providerSelection {
                case .jellyfin:
                    .jellyfin
                case .navidrome:
                    .navidrome
                case .auto:
                    if let detectedProvider {
                        detectedProvider
                    } else {
                        try await detectProvider(serverUrl: serverUrl)
                    }
                }

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
                    let previousProfileId = sessionStore.getActiveProfileId()
                    sessionStore.save(
                        serverUrl: serverUrl,
                        userId: response.userId,
                        token: response.token,
                        username: username,
                        provider: resolvedProvider
                    )
                    if let previousProfileId, !previousProfileId.isEmpty {
                        _ = sessionStore.switchProfile(previousProfileId)
                    }

                    onProfileAdded()
                    isSubmitting = false
                    dismiss()
                }
            } catch {
                await MainActor.run {
                    isSubmitting = false
                    self.error = error.localizedDescription
                }
            }
        }
    }
}
