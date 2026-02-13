import SwiftUI

struct SettingsView: View {
    @Environment(AppViewModel.self) private var appViewModel
    @Environment(AudioPlayerController.self) private var playerController
    @State private var viewModel = SettingsViewModel()
    @State private var showLogoutConfirmation = false
    @State private var lyricsServerUrl: String = SessionStore.shared.lyricsServerUrl ?? ""
    @State private var visualizerEnabled: Bool = SessionStore.shared.visualizerEnabled
    @State private var visualizerStyle: VisualizerStyle = .init(rawValue: SessionStore.shared.visualizerStyle) ?? .bars

    var body: some View {
        NavigationStack {
            settingsContent
                .aureliaRootTabHeader("Settings")
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
        .onAppear {
            visualizerEnabled = SessionStore.shared.visualizerEnabled
            visualizerStyle = VisualizerStyle(rawValue: SessionStore.shared.visualizerStyle) ?? .bars
            playerController.refreshVisualizerSettings()
        }
    }

    private var settingsSections: some View {
        Group {
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
                .disabled(viewModel.isClearing)
            }

            Section("Visualizer") {
                Toggle("Enable Visualizer", isOn: $visualizerEnabled)
                    .onChange(of: visualizerEnabled) { _, newValue in
                        playerController.setVisualizerEnabled(newValue)
                    }

                Picker("Style", selection: $visualizerStyle) {
                    ForEach(VisualizerStyle.allCases, id: \.self) { style in
                        Text(style.title).tag(style)
                    }
                }
                .disabled(!visualizerEnabled)
                .onChange(of: visualizerStyle) { _, newValue in
                    playerController.setVisualizerStyle(newValue)
                }
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
            }

            Section {
                TextField("http://localhost:3030", text: $lyricsServerUrl)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .keyboardType(.URL)
                    .onChange(of: lyricsServerUrl) { _, newValue in
                        SessionStore.shared.lyricsServerUrl = newValue.isEmpty ? nil : newValue
                    }
            } header: {
                Text("Lyrics Server")
            } footer: {
                Text("URL of the lyrics daemon for synced lyrics from sidecar files. Leave empty to use Jellyfin lyrics only.")
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
    }

    @ViewBuilder
    private var settingsContent: some View {
        #if targetEnvironment(macCatalyst)
            Form {
                settingsSections
            }
            .formStyle(.grouped)
            .frame(maxWidth: 600)
            .frame(maxWidth: .infinity)
        #else
            List {
                settingsSections
            }
            .listStyle(.insetGrouped)
            .scrollContentBackground(.hidden)
            .listRowBackground(Rectangle().fill(.ultraThinMaterial))
        #endif
    }
}
