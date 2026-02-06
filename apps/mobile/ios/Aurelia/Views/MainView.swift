import SwiftUI

struct MainView: View {
    @Environment(\.horizontalSizeClass) private var sizeClass
    @Environment(\.colorScheme) private var colorScheme
    @Environment(AudioPlayerController.self) private var playerController
    @State private var selection: MainDestination = .home
    @State private var showPlayer = false

    var body: some View {
        Group {
            if sizeClass == .regular {
                MainSplitView(selection: $selection, showPlayer: $showPlayer)
            } else {
                MainTabView(selectedTab: $selection)
                    .miniPlayerInset(showPlayer: $showPlayer)
            }
        }
        .tint(AureliaPalette.tint(for: colorScheme))
        .animation(.spring(response: 0.4, dampingFraction: 0.85), value: playerController.snapshot.currentSongId)
        .fullScreenCover(isPresented: $showPlayer) {
            PlayerView()
        }
    }
}

struct MainSplitView: View {
    @Binding var selection: MainDestination
    @Binding var showPlayer: Bool

    var body: some View {
        NavigationSplitView {
            List(selection: selectionBinding) {
                ForEach(MainDestination.allCases) { destination in
                    NavigationLink(value: destination) {
                        Label(destination.title, systemImage: destination.systemImage)
                    }
                }
            }
            .listStyle(.sidebar)
            .navigationTitle("Aurelia")
        } detail: {
            selection.destinationView()
                .miniPlayerInset(showPlayer: $showPlayer)
        }
        .navigationSplitViewStyle(.balanced)
    }

    private var selectionBinding: Binding<MainDestination?> {
        Binding(
            get: { selection },
            set: { newValue in
                if let newValue {
                    selection = newValue
                }
            }
        )
    }
}

private struct MiniPlayerInsetModifier: ViewModifier {
    @Environment(AudioPlayerController.self) private var playerController
    @Binding var showPlayer: Bool

    func body(content: Content) -> some View {
        content.safeAreaInset(edge: .bottom) {
            if playerController.snapshot.currentSongId != nil {
                MiniPlayerView(onTap: { showPlayer = true })
                    .padding(.horizontal, AureliaSpacing.m)
                    .padding(.top, AureliaSpacing.s)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
    }
}

private extension View {
    func miniPlayerInset(showPlayer: Binding<Bool>) -> some View {
        modifier(MiniPlayerInsetModifier(showPlayer: showPlayer))
    }
}
