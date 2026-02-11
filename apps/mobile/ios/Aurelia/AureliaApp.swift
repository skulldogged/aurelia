import AVFoundation
import SwiftUI

@main
struct AureliaApp: App {
    @State private var appViewModel = AppViewModel()
    @State private var playerController: AudioPlayerController?
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.scenePhase) private var scenePhase

    var body: some Scene {
        WindowGroup {
            Group {
                if appViewModel.isLoading {
                    ZStack {
                        AureliaBackground()
                        ProgressView()
                    }
                } else if appViewModel.isLoggedIn {
                    if let playerController {
                        MainView()
                            .environment(playerController)
                    } else {
                        ZStack {
                            AureliaBackground()
                            ProgressView()
                        }
                        .task {
                            let controller = AudioPlayerController()
                            playerController = controller
                        }
                    }
                } else {
                    LoginView()
                }
            }
            .environment(appViewModel)
            .tint(AureliaPalette.tint(for: colorScheme))
            .onAppear {
                AuthInterceptor.shared.setLogoutCallback { [appViewModel] in
                    Task { @MainActor in
                        playerController = nil
                        appViewModel.logout()
                    }
                }
            }
            .onChange(of: scenePhase) { _, newPhase in
                switch newPhase {
                case .background:
                    print("App entered background - ensuring audio session stays active")
                    do {
                        try AVAudioSession.sharedInstance().setActive(true)
                    } catch {
                        print("Failed to activate audio session in background: \(error)")
                    }
                case .inactive:
                    print("App became inactive")
                case .active:
                    print("App became active")
                    do {
                        try AVAudioSession.sharedInstance().setActive(true)
                    } catch {
                        print("Failed to activate audio session: \(error)")
                    }
                @unknown default:
                    break
                }
            }
        }
        .commands {
            AureliaAppCommands(isEnabled: appViewModel.isLoggedIn)
        }
    }
}

enum AureliaMenuCommand {
    case goHome
    case goSongs
    case goAlbums
    case goArtists
    case goSearch
    case goSettings
    case openNowPlaying
    case togglePlayPause
    case nextTrack
    case previousTrack
    case toggleShuffle
    case cycleRepeatMode
}

extension Notification.Name {
    static let aureliaMenuCommand = Notification.Name("AureliaMenuCommand")
}

func postAureliaMenuCommand(_ command: AureliaMenuCommand) {
    NotificationCenter.default.post(name: .aureliaMenuCommand, object: command)
}

private struct AureliaAppCommands: Commands {
    let isEnabled: Bool

    var body: some Commands {
        SidebarCommands()

        CommandMenu("Navigate") {
            Button("Home") {
                postAureliaMenuCommand(.goHome)
            }
            .keyboardShortcut("1", modifiers: .command)
            .disabled(!isEnabled)

            Button("Songs") {
                postAureliaMenuCommand(.goSongs)
            }
            .keyboardShortcut("2", modifiers: .command)
            .disabled(!isEnabled)

            Button("Albums") {
                postAureliaMenuCommand(.goAlbums)
            }
            .keyboardShortcut("3", modifiers: .command)
            .disabled(!isEnabled)

            Button("Artists") {
                postAureliaMenuCommand(.goArtists)
            }
            .keyboardShortcut("4", modifiers: .command)
            .disabled(!isEnabled)

            Divider()

            Button("Search") {
                postAureliaMenuCommand(.goSearch)
            }
            .keyboardShortcut("f", modifiers: .command)
            .disabled(!isEnabled)

            Button("Settings") {
                postAureliaMenuCommand(.goSettings)
            }
            .keyboardShortcut(",", modifiers: .command)
            .disabled(!isEnabled)
        }

        CommandMenu("Playback") {
            Button("Now Playing") {
                postAureliaMenuCommand(.openNowPlaying)
            }
            .keyboardShortcut("0", modifiers: .command)
            .disabled(!isEnabled)

            Divider()

            Button("Play/Pause") {
                postAureliaMenuCommand(.togglePlayPause)
            }
            .keyboardShortcut(.space, modifiers: [])
            .disabled(!isEnabled)

            Button("Previous Track") {
                postAureliaMenuCommand(.previousTrack)
            }
            .keyboardShortcut(.leftArrow, modifiers: [.command, .option])
            .disabled(!isEnabled)

            Button("Next Track") {
                postAureliaMenuCommand(.nextTrack)
            }
            .keyboardShortcut(.rightArrow, modifiers: [.command, .option])
            .disabled(!isEnabled)

            Divider()

            Button("Toggle Shuffle") {
                postAureliaMenuCommand(.toggleShuffle)
            }
            .keyboardShortcut("s", modifiers: [.command, .shift])
            .disabled(!isEnabled)

            Button("Cycle Repeat Mode") {
                postAureliaMenuCommand(.cycleRepeatMode)
            }
            .keyboardShortcut("r", modifiers: [.command, .shift])
            .disabled(!isEnabled)
        }
    }
}
