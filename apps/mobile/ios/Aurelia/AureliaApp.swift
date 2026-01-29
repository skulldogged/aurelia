import SwiftUI

@main
struct AureliaApp: App {
    @State private var appViewModel = AppViewModel()
    @State private var playerController = AudioPlayerController()

    var body: some Scene {
        WindowGroup {
            Group {
                if appViewModel.isLoading {
                    ProgressView()
                } else if appViewModel.isLoggedIn {
                    MainTabView()
                } else {
                    LoginView()
                }
            }
            .environment(appViewModel)
            .environment(playerController)
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
