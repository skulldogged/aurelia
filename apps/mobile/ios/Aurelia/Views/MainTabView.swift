import SwiftUI

struct MainTabView: View {
    @Binding var selectedTab: MainDestination
#if !targetEnvironment(macCatalyst)
    @Binding var animatedContentWidth: CGFloat
    @Binding var hasAnimatedContentWidth: Bool
#endif

    var body: some View {
#if targetEnvironment(macCatalyst)
        catalystLayout
#endif
#if !targetEnvironment(macCatalyst)
        mainTabs
#endif
    }

    @ViewBuilder
    private var mainTabs: some View {
        if useSidebarAdaptable {
            tabs
                .tabViewStyle(.sidebarAdaptable)
        } else {
            tabs
                .tabViewStyle(.automatic)
        }
    }

    private var tabs: some View {
        TabView(selection: $selectedTab) {
            SwiftUI.Tab("Home", systemImage: MainDestination.home.systemImage, value: MainDestination.home) {
                tabContent {
                    HomeView()
                }
            }

            SwiftUI.Tab("Songs", systemImage: MainDestination.songs.systemImage, value: MainDestination.songs) {
                tabContent {
                    LibraryView()
                }
            }

            SwiftUI.Tab("Albums", systemImage: MainDestination.albums.systemImage, value: MainDestination.albums) {
                tabContent {
                    AlbumsView()
                }
            }

            SwiftUI.Tab("Artists", systemImage: MainDestination.artists.systemImage, value: MainDestination.artists) {
                tabContent {
                    ArtistsView()
                }
            }

            SwiftUI.Tab("Search", systemImage: MainDestination.search.systemImage, value: MainDestination.search) {
                tabContent {
                    SearchView()
                }
            }

            SwiftUI.Tab("Settings", systemImage: MainDestination.settings.systemImage, value: MainDestination.settings) {
                tabContent {
                    SettingsView()
                }
            }
        }
    }

    private var useSidebarAdaptable: Bool {
        UIDevice.current.userInterfaceIdiom == .pad
    }

#if !targetEnvironment(macCatalyst)
    @ViewBuilder
    private func tabContent<Content: View>(@ViewBuilder _ content: () -> Content) -> some View {
        if useSidebarAdaptable {
            AnimatedTabContentWidthHost(
                animatedWidth: $animatedContentWidth,
                hasInitialWidth: $hasAnimatedContentWidth,
                content: content
            )
        } else {
            content()
        }
    }
#endif

#if targetEnvironment(macCatalyst)
    private var catalystLayout: some View {
        VStack(spacing: 0) {
            catalystTopBar
                .padding(.horizontal, AureliaSpacing.m)
                .padding(.vertical, AureliaSpacing.s)

            selectedTab.destinationView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(AureliaBackground())
    }

    private var catalystTopBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 0) {
                ForEach(Array(MainDestination.allCases.enumerated()), id: \.element.id) { index, destination in
                    tabButton(for: destination)
                    if index < MainDestination.allCases.count - 1 {
                        Divider()
                            .frame(height: 18)
                            .padding(.horizontal, 8)
                    }
                }
            }
            .padding(4)
            .background(.ultraThinMaterial, in: Capsule())
            .overlay(
                Capsule()
                    .stroke(Color.primary.opacity(0.12), lineWidth: 1)
            )
        }
    }

    private func tabButton(for destination: MainDestination) -> some View {
        Button {
            selectedTab = destination
        } label: {
            Text(destination.title)
                .font(.subheadline.weight(.semibold))
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
                .foregroundStyle(selectedTab == destination ? Color.primary : .secondary)
                .background(
                    selectedTab == destination ? Color.primary.opacity(0.16) : Color.clear,
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
    }
#endif
}

#if !targetEnvironment(macCatalyst)
private struct AnimatedTabContentWidthHost<Content: View>: View {
    @Binding var animatedWidth: CGFloat
    @Binding var hasInitialWidth: Bool
    private let content: Content

    init(
        animatedWidth: Binding<CGFloat>,
        hasInitialWidth: Binding<Bool>,
        @ViewBuilder content: () -> Content
    ) {
        _animatedWidth = animatedWidth
        _hasInitialWidth = hasInitialWidth
        self.content = content()
    }

    var body: some View {
        GeometryReader { proxy in
            let targetWidth = max(proxy.size.width, 1)
            let displayedWidth = hasInitialWidth ? animatedWidth : targetWidth
            let trailingOffset = targetWidth - displayedWidth

            content
                .frame(width: displayedWidth, alignment: .topLeading)
                .offset(x: trailingOffset)
                .frame(width: targetWidth, alignment: .topLeading)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .onAppear {
                    if !hasInitialWidth {
                        animatedWidth = targetWidth
                        hasInitialWidth = true
                    } else if abs(targetWidth - animatedWidth) > 0.5 {
                        withAnimation(.easeInOut(duration: 0.28)) {
                            animatedWidth = targetWidth
                        }
                    }
                }
                .onChange(of: targetWidth) { _, newWidth in
                    guard hasInitialWidth else {
                        animatedWidth = newWidth
                        hasInitialWidth = true
                        return
                    }
                    guard abs(newWidth - animatedWidth) > 0.5 else { return }
                    withAnimation(.easeInOut(duration: 0.28)) {
                        animatedWidth = newWidth
                    }
                }
        }
    }
}
#endif
