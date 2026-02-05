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
                MainSplitView(selection: $selection)
            } else {
                MainTabView(selectedTab: $selection)
            }
        }
        .tint(AureliaPalette.tint(for: colorScheme))
        .safeAreaInset(edge: .bottom) {
            if playerController.snapshot.currentSongId != nil {
                MiniPlayerView(onTap: { showPlayer = true })
                    .padding(.horizontal, AureliaSpacing.m)
                    .padding(.top, AureliaSpacing.s)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.85), value: playerController.snapshot.currentSongId)
        .sheet(isPresented: $showPlayer) {
            PlayerView()
        }
    }
}

struct MainSplitView: View {
    @Binding var selection: MainDestination

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
