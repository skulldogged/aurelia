import SwiftUI

@main
struct AureliaMacApp: App {
    @StateObject private var appViewModel = AppViewModel()
    @StateObject private var playerController = AudioPlayerController()

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
                        .environmentObject(playerController)
                } else {
                    LoginView()
                }
            }
            .environmentObject(appViewModel)
            .onAppear {
                AuthInterceptor.shared.setLogoutCallback { [appViewModel] in
                    Task { @MainActor in
                        appViewModel.logout()
                    }
                }
            }
        }
        .windowStyle(.titleBar)
    }
}
