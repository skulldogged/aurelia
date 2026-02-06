import SwiftUI

@main
struct AureliaApp: App {
    @State private var appViewModel = AppViewModel()
    @State private var playerController: AudioPlayerController?
    @Environment(\.colorScheme) private var colorScheme

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
                            UIApplication.shared.beginReceivingRemoteControlEvents()
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
                        self.playerController = nil
                        appViewModel.logout()
                    }
                }
            }
        }
    }
}
