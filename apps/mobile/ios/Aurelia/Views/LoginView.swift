import AureliaCore
import SwiftUI

struct LoginView: View {
    @Environment(AppViewModel.self) private var appViewModel
    @State private var viewModel = LoginViewModel()

    var body: some View {
        NavigationStack {
            GeometryReader { proxy in
                let isWide = AureliaLayout.isWide(proxy.size.width)
                ScrollView {
                    if isWide {
                        HStack(alignment: .center, spacing: AureliaSpacing.xxl) {
                            branding
                                .frame(maxWidth: 320)

                            formCard
                                .frame(maxWidth: 420)
                        }
                        .frame(maxWidth: 900)
                        .padding(.horizontal, AureliaSpacing.xxl)
                        .padding(.vertical, AureliaSpacing.xl)
                        .frame(maxWidth: .infinity)
                    } else {
                        VStack(spacing: AureliaSpacing.xl) {
                            branding
                            formCard
                        }
                        .padding(.horizontal, AureliaSpacing.l)
                        .padding(.vertical, AureliaSpacing.xl)
                        .frame(maxWidth: .infinity)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .navigationTitle("Sign In")
            .navigationBarTitleDisplayMode(.inline)
        }
        .aureliaScreen()
        .onChange(of: viewModel.token) {
            if viewModel.token != nil {
                appViewModel.checkSession()
            }
        }
    }

    private var branding: some View {
        VStack(spacing: 12) {
            Image(systemName: "music.note.house.fill")
                .font(.system(size: 64))
                .foregroundStyle(.tint)
            Text("Aurelia")
                .font(.largeTitle.bold())
            Text("A Multi-Provider Music Client")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
    }

    private var formCard: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.l) {
            VStack(spacing: AureliaSpacing.m) {
                TextField("Server URL", text: $viewModel.serverUrl)
                    .textContentType(.URL)
                    .keyboardType(.URL)
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)

                Picker("Provider", selection: $viewModel.providerSelection) {
                    ForEach(LoginProviderSelection.allCases) { option in
                        Text(option.title).tag(option)
                    }
                }
                .pickerStyle(.segmented)

                if viewModel.providerSelection == .auto {
                    Button {
                        viewModel.detectProviderNow()
                    } label: {
                        if viewModel.isDetectingProvider {
                            ProgressView()
                                .frame(maxWidth: .infinity)
                        } else {
                            Text("Detect Provider")
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .buttonStyle(.bordered)
                    .disabled(viewModel.isSubmitting || viewModel.serverUrl.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    Text(detectedProviderText)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                TextField("Username", text: $viewModel.username)
                    .textContentType(.username)
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)

                SecureField("Password", text: $viewModel.password)
                    .textContentType(.password)

                if let error = viewModel.error {
                    Text(error)
                        .font(.callout)
                        .foregroundStyle(.red)
                        .multilineTextAlignment(.center)
                }

                Button {
                    viewModel.submit()
                } label: {
                    if viewModel.isSubmitting {
                        ProgressView()
                            .frame(maxWidth: .infinity)
                    } else {
                        Text("Sign In")
                            .frame(maxWidth: .infinity)
                    }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .disabled(viewModel.isSubmitting)
            }
            .textFieldStyle(.roundedBorder)
        }
    }

    private var detectedProviderText: String {
        switch viewModel.detectedProvider {
        case .some(.jellyfin):
            return "Detected provider: Jellyfin"
        case .some(.navidrome):
            return "Detected provider: Navidrome"
        case nil:
            return "Detected provider: not detected"
        }
    }
}
