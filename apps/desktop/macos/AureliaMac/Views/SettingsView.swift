import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var appViewModel: AppViewModel
    @StateObject private var viewModel = SettingsViewModel()
    @State private var showLogoutConfirmation = false

    var body: some View {
        NavigationStack {
            List {
                Section("Library Sync") {
                    HStack {
                        Text("Last Synced")
                        Spacer()
                        Text(TimeFormatter.formatRelativeTime(viewModel.lastSyncTime))
                            .foregroundStyle(.secondary)
                    }

                    Button {
                        viewModel.syncLibrary()
                    } label: {
                        HStack {
                            Text("Sync Now")
                            Spacer()
                            if viewModel.isSyncing {
                                ProgressView()
                            } else if let success = viewModel.syncSuccess {
                                Image(systemName: success ? "checkmark.circle.fill" : "xmark.circle.fill")
                                    .foregroundStyle(success ? .green : .red)
                            }
                        }
                    }
                    .buttonStyle(.glass)
                    .disabled(viewModel.isSyncing)
                }

                Section("Cache") {
                    Button(role: .destructive) {
                        viewModel.clearLocalCache()
                    } label: {
                        HStack {
                            Text("Clear Local Cache")
                            Spacer()
                            if viewModel.isClearing {
                                ProgressView()
                            } else if let success = viewModel.clearSuccess {
                                Image(systemName: success ? "checkmark.circle.fill" : "xmark.circle.fill")
                                    .foregroundStyle(success ? .green : .red)
                            }
                        }
                    }
                    .buttonStyle(.glass)
                    .disabled(viewModel.isClearing)
                }

                Section("Account") {
                    if let serverUrl = SessionStore.shared.serverUrl {
                        HStack {
                            Text("Server")
                            Spacer()
                            Text(serverUrl)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                    }

                    Button("Sign Out", role: .destructive) {
                        showLogoutConfirmation = true
                    }
                    .buttonStyle(.glass)
                }

                Section("About") {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.1.0")
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .listStyle(.inset)
            .navigationTitle("Settings")
            .confirmationDialog("Sign Out", isPresented: $showLogoutConfirmation) {
                Button("Sign Out", role: .destructive) {
                    SessionStore.shared.clear()
                    appViewModel.logout()
                }
            } message: {
                Text("Are you sure you want to sign out?")
            }
        }
        .aureliaScreen()
    }
}
