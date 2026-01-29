import SwiftUI

struct LoginView: View {
    @Environment(AppViewModel.self) private var appViewModel
    @State private var viewModel = LoginViewModel()

    var body: some View {
        NavigationStack {
            VStack(spacing: 32) {
                Spacer()

                // App branding
                VStack(spacing: 8) {
                    Image(systemName: "music.note.house.fill")
                        .font(.system(size: 64))
                        .foregroundStyle(.tint)
                    Text("Aurelia")
                        .font(.largeTitle.bold())
                    Text("A Jellyfin Music Client")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                // Login form
                VStack(spacing: 16) {
                    TextField("Server URL", text: $viewModel.serverUrl)
                        .textContentType(.URL)
                        .keyboardType(.URL)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)

                    TextField("Username", text: $viewModel.username)
                        .textContentType(.username)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)

                    SecureField("Password", text: $viewModel.password)
                        .textContentType(.password)
                }
                .textFieldStyle(.roundedBorder)
                .padding(.horizontal)

                // Error message
                if let error = viewModel.error {
                    Text(error)
                        .font(.callout)
                        .foregroundStyle(.red)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal)
                }

                // Submit button
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
                .padding(.horizontal)

                Spacer()
            }
            .navigationTitle("Sign In")
            .navigationBarTitleDisplayMode(.inline)
        }
        .onChange(of: viewModel.token) {
            if viewModel.token != nil {
                appViewModel.checkSession()
            }
        }
    }
}
