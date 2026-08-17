import SwiftUI

struct MainTabView: View {
    @Binding var selectedTab: MainDestination
    @Binding var playerPresentationProgress: CGFloat
    var onMiniPlayerTap: () -> Void
    var onMiniPlayerLyricsTap: () -> Void
    var onMiniPlayerQueueTap: () -> Void
    @Environment(\.tabBarPlacement) private var tabBarPlacement
    @Environment(AppViewModel.self) private var appViewModel
    @Environment(AudioPlayerController.self) private var playerController
    @State private var profiles: [SessionProfile] = []
    @State private var activeProfileId: String?
    @State private var showAddProfileSheet = false

    var body: some View {
        Group {
            if useSidebarAdaptable {
                tabs
                    .tabViewStyle(.sidebarAdaptable)
                    .overlay(alignment: .bottomLeading) {
                        if showsSidebarAccountMenu {
                            SidebarAccountMenu(
                                activeProfileLabel: activeProfileLabel,
                                activeProfileId: activeProfileId,
                                profiles: profiles,
                                onAddProfile: { showAddProfileSheet = true },
                                onLogout: handleLogout,
                                onSwitchProfile: switchToProfile
                            )
                            .padding(.leading, AureliaSpacing.m)
                            .padding(.bottom, sidebarAccountBottomPadding)
                        }
                    }
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
        .sheet(isPresented: $showAddProfileSheet) {
            AddProfileSheet {
                refreshProfiles()
            }
        }
        .onAppear {
            refreshProfiles()
        }
        .onChange(of: appViewModel.sessionVersion) { _, _ in
            refreshProfiles()
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
        UIDevice.current.userInterfaceIdiom == .pad
    }

    private var showsSidebarAccountMenu: Bool {
        tabBarPlacement == .sidebar || tabBarPlacement == .topBar
    }

    private var activeProfileLabel: String {
        if let activeProfileId,
           let profile = profiles.first(where: { $0.id == activeProfileId })
        {
            return profile.username.isEmpty ? profile.userId : profile.username
        }
        return "Account"
    }

    private var sidebarAccountBottomPadding: CGFloat {
        if playerController.snapshot.currentSongId == nil {
            return AureliaSpacing.s
        }
        // Keep the account chip visible when the shared mini-player inset is present.
        return 84
    }

    private func refreshProfiles() {
        profiles = SessionStore.shared.getProfiles()
        activeProfileId = SessionStore.shared.getActiveProfileId()
    }

    private func switchToProfile(_ profile: SessionProfile) {
        guard profile.id != activeProfileId else { return }
        playerController.stop()
        guard appViewModel.switchProfile(profile.id) else { return }
        refreshProfiles()
    }

    private func handleLogout() {
        playerController.stop()
        appViewModel.logout()
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

    private struct SidebarAccountMenu: View {
        let activeProfileLabel: String
        let activeProfileId: String?
        let profiles: [SessionProfile]
        let onAddProfile: () -> Void
        let onLogout: () -> Void
        let onSwitchProfile: (SessionProfile) -> Void

        var body: some View {
            Menu {
                Section("Profiles") {
                    if profiles.isEmpty {
                        Text("No saved profiles")
                    }
                    ForEach(profiles) { profile in
                        Button {
                            onSwitchProfile(profile)
                        } label: {
                            if profile.id == activeProfileId {
                                Label(profile.label, systemImage: "checkmark")
                            } else {
                                Text(profile.label)
                            }
                        }
                        .disabled(profile.id == activeProfileId)
                    }
                }

                Button {
                    onAddProfile()
                } label: {
                    Label("Add Profile", systemImage: "plus")
                }

                Divider()

                Button(role: .destructive) {
                    onLogout()
                } label: {
                    Label("Log Out", systemImage: "rectangle.portrait.and.arrow.right")
                }
            } label: {
                HStack(spacing: 10) {
                    Image(systemName: "person.crop.circle.fill")
                        .font(.title3)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(activeProfileLabel)
                            .font(.subheadline.weight(.semibold))
                            .lineLimit(1)
                        Text("Account")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Image(systemName: "chevron.up.chevron.down")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 10)
                .frame(width: 250, alignment: .leading)
                .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
            }
            .menuStyle(.borderlessButton)
            .buttonStyle(.plain)
        }
    }

    private func tabContent(@ViewBuilder _ content: () -> some View) -> some View {
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
    func tabViewBottomAccessoryIfAvailable(isEnabled: Bool, @ViewBuilder content: () -> some View) -> some View {
        if #available(iOS 26.1, *) {
            tabViewBottomAccessory(isEnabled: isEnabled, content: content)
        } else {
            overlay(alignment: .bottom) {
                if isEnabled {
                    content()
                        .padding(.bottom, 49) // Approximate standard Tab Bar height
                }
            }
        }
    }
}
