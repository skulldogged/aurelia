import SwiftUI

struct MainTabView: View {
    @Binding var selectedTab: MainDestination
    @Binding var playerPresentationProgress: CGFloat
    var onMiniPlayerTap: () -> Void
    @Environment(AudioPlayerController.self) private var playerController

    var body: some View {
#if targetEnvironment(macCatalyst)
        CatalystSplitView(selectedTab: $selectedTab)
#else
        if useSidebarAdaptable {
            tabs
                .tabViewStyle(.sidebarAdaptable)
        } else {
            tabs
                .tabViewStyle(.automatic)
                .tabViewBottomAccessory(isEnabled: playerController.snapshot.currentSongId != nil) {
                    TabBarMiniPlayer(
                        playerPresentationProgress: $playerPresentationProgress,
                        onTap: onMiniPlayerTap
                    )
                }
                .background(Color.clear)
        }
#endif
    }

#if !targetEnvironment(macCatalyst)
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
        return UIDevice.current.userInterfaceIdiom == .pad
    }

    /// A dedicated view for the tab bar bottom accessory miniplayer.
    /// Isolating this into its own struct ensures that `@State` in child views
    /// (like `AlbumArtView`) persists across `AudioPlayerController` snapshot updates,
    /// preventing album art flickering.
    private struct TabBarMiniPlayer: View {
        @Binding var playerPresentationProgress: CGFloat
        var onTap: () -> Void

        var body: some View {
            MiniPlayerView(onTap: onTap)
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
#endif
}

#if !targetEnvironment(macCatalyst)
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
                    // Don't update animatedWidth yet - keep it at old value.
                    // Schedule the animation for the next frame so SwiftUI
                    // has committed the layout pass and our withAnimation
                    // actually takes effect.
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
#endif

#if targetEnvironment(macCatalyst)
private struct CatalystSplitView: View {
    @Binding var selectedTab: MainDestination
    @State private var columnVisibility: NavigationSplitViewVisibility = .automatic
    @State private var contentWidth: CGFloat = 0

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            sidebarContent
        } detail: {
            detailContent
        }
        .navigationSplitViewStyle(.automatic)
    }

    private var sidebarContent: some View {
        List {
            ForEach(MainDestination.allCases) { destination in
                sidebarRow(for: destination)
            }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(ideal: 200, max: 240)
    }

    private func sidebarRow(for destination: MainDestination) -> some View {
        let isSelected = selectedTab == destination
        return Button {
            selectedTab = destination
        } label: {
            Label {
                Text(destination.title)
                    .font(.body)
            } icon: {
                Image(systemName: destination.systemImage)
                    .font(.body)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.vertical, 4)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .listRowBackground(
            isSelected
                ? RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .fill(Color.accentColor.opacity(0.15))
                : nil
        )
        .foregroundStyle(isSelected ? Color.accentColor : .primary)
    }

    @ViewBuilder
    private var detailContent: some View {
        switch selectedTab {
        case .home: HomeView().navigationTitle(selectedTab.title)
        case .songs: LibraryView().navigationTitle(selectedTab.title)
        case .albums: AlbumsView().navigationTitle(selectedTab.title)
        case .artists: ArtistsView().navigationTitle(selectedTab.title)
        case .search: SearchView().navigationTitle(selectedTab.title)
        case .settings: SettingsView().navigationTitle(selectedTab.title)
        }
    }
}
#endif
