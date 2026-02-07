import SwiftUI

struct LoginView: View {
    @EnvironmentObject private var appViewModel: AppViewModel
    @StateObject private var viewModel = LoginViewModel()

    var body: some View {
        VStack(spacing: 20) {
            VStack(spacing: 8) {
                Text("Aurelia")
                    .font(.system(size: 32, weight: .bold))
                Text("Sign in to your Jellyfin server")
                    .foregroundStyle(.secondary)
            }

            GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m) {
                VStack(spacing: 12) {
                    TextField("Server URL", text: $viewModel.serverUrl)
                        .textFieldStyle(.roundedBorder)
                    TextField("Username", text: $viewModel.username)
                        .textFieldStyle(.roundedBorder)
                    SecureField("Password", text: $viewModel.password)
                        .textFieldStyle(.roundedBorder)
                }
            }

            Button {
                viewModel.submit {
                    appViewModel.checkSession()
                }
            } label: {
                if viewModel.isSubmitting {
                    ProgressView().controlSize(.small)
                } else {
                    Text("Sign In")
                        .frame(maxWidth: .infinity)
                }
            }
            .buttonStyle(.glass)
            .disabled(viewModel.isSubmitting)

            if let error = viewModel.error {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .multilineTextAlignment(.center)
            }
        }
        .padding(32)
        .frame(width: 420)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .aureliaScreen()
    }
}
