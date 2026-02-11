import SwiftUI

struct MainTabView: View {
    @Binding var selectedTab: MainDestination
    @Binding var playerPresentationProgress: CGFloat
    var onMiniPlayerTap: () -> Void
    var onMiniPlayerLyricsTap: () -> Void
    var onMiniPlayerQueueTap: () -> Void
    @Environment(AudioPlayerController.self) private var playerController

    var body: some View {
        if useSidebarAdaptable {
            tabs
                .tabViewStyle(.sidebarAdaptable)
        } else {
            tabs
                .tabViewStyle(.automatic)
                .tabViewBottomAccessoryIfAvailable(isEnabled: playerController.snapshot.currentSongId != nil) {
                    TabBarMiniPlayer(
                        playerPresentationProgress: $playerPresentationProgress,
                        onTap: onMiniPlayerTap,
                        onLyricsTap: onMiniPlayerLyricsTap,
                        onQueueTap: onMiniPlayerQueueTap
                    )
                }
                .background(Color.clear)
        }
    }

    private var tabs: some View {
        TabView(selection: $selectedTab) {
            SwiftUI.Tab(value: MainDestination.home) {
                tabContent {
                    HomeView()
                }
            } label: {
                tabLabel(for: .home)
            }

            SwiftUI.Tab(value: MainDestination.songs) {
                tabContent {
                    LibraryView()
                }
            } label: {
                tabLabel(for: .songs)
            }

            SwiftUI.Tab(value: MainDestination.albums) {
                tabContent {
                    AlbumsView()
                }
            } label: {
                tabLabel(for: .albums)
            }

            SwiftUI.Tab(value: MainDestination.artists) {
                tabContent {
                    ArtistsView()
                }
            } label: {
                tabLabel(for: .artists)
            }

            SwiftUI.Tab(value: MainDestination.search) {
                tabContent {
                    SearchView()
                }
            } label: {
                tabLabel(for: .search)
            }

            SwiftUI.Tab(value: MainDestination.settings) {
                tabContent {
                    SettingsView()
                }
            } label: {
                tabLabel(for: .settings)
            }
        }
    }

    private func tabLabel(for destination: MainDestination) -> some View {
        Label(destination.title, systemImage: destination.systemImage)
    }

    private var useSidebarAdaptable: Bool {
        return UIDevice.current.userInterfaceIdiom == .pad || ProcessInfo.processInfo.isMacCatalystApp
    }

    private struct TabBarMiniPlayer: View {
        @Binding var playerPresentationProgress: CGFloat
        var onTap: () -> Void
        var onLyricsTap: () -> Void
        var onQueueTap: () -> Void

        var body: some View {
            MiniPlayerView(onTap: onTap, onLyricsTap: onLyricsTap, onQueueTap: onQueueTap)
                .onTapGesture(perform: onTap)
                .opacity(Double(max(CGFloat(0), CGFloat(1) - playerPresentationProgress * CGFloat(1.4))))
                .offset(y: playerPresentationProgress * 42)
                .allowsHitTesting(playerPresentationProgress < 0.95)
        }
    }

    @ViewBuilder
    private func tabContent<Content: View>(@ViewBuilder _ content: () -> Content) -> some View {
        content()
            .modifier(SidebarSlideModifier())
    }
}

private struct SidebarSlideModifier: ViewModifier {
    @Environment(\.tabBarPlacement) private var tabBarPlacement
    @State private var animatedWidth: CGFloat = 0
    @State private var hasInitialWidth = false
    @State private var isTransitioning = false
    @State private var pendingWidth: CGFloat? = nil

    func body(content: Content) -> some View {
        Color.clear
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(
                GeometryReader { proxy in
                    Color.clear
                        .preference(key: ContainerWidthKey.self, value: proxy.size.width)
                }
            )
            .onPreferenceChange(ContainerWidthKey.self) { newWidth in
                guard newWidth > 0 else { return }

                guard hasInitialWidth else {
                    animatedWidth = newWidth
                    hasInitialWidth = true
                    return
                }

                if isTransitioning {
                    pendingWidth = newWidth
                    DispatchQueue.main.async {
                        guard let target = pendingWidth else { return }
                        pendingWidth = nil
                        withAnimation(.easeInOut(duration: 0.28)) {
                            animatedWidth = target
                        }
                        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                            isTransitioning = false
                        }
                    }
                } else {
                    animatedWidth = newWidth
                }
            }
            .overlay {
                content
                    .frame(width: max(animatedWidth, 1))
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                    .clipped()
            }
            .onChange(of: tabBarPlacement) { _, _ in
                isTransitioning = true
            }
    }
}

private struct ContainerWidthKey: PreferenceKey {
    static var defaultValue: CGFloat = 0
    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}

private extension View {
    @ViewBuilder
    func tabViewBottomAccessoryIfAvailable<Content: View>(isEnabled: Bool, @ViewBuilder content: () -> Content) -> some View {
        if #available(iOS 26.1, *) {
            self.tabViewBottomAccessory(isEnabled: isEnabled, content: content)
        } else {
            self.overlay(alignment: .bottom) {
                if isEnabled {
                    content()
                        .padding(.bottom, 49) // Approximate standard Tab Bar height
                }
            }
        }
    }
}
