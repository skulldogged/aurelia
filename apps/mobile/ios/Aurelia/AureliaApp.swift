import SwiftUI

@main
struct AureliaApp: App {
    @State private var appViewModel = AppViewModel()
    @State private var playerController = AudioPlayerController()
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
                    MainView()
                } else {
                    LoginView()
                }
            }
            .environment(appViewModel)
            .environment(playerController)
            .tint(AureliaPalette.tint(for: colorScheme))
            .onAppear {
                // Enable remote control events for Lock Screen / Control Center
                UIApplication.shared.beginReceivingRemoteControlEvents()
                
                AuthInterceptor.shared.setLogoutCallback { [appViewModel] in
                    Task { @MainActor in
                        appViewModel.logout()
                    }
                }
            }
        }
    }
}
