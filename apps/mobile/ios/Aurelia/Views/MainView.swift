import SwiftUI

struct MainView: View {
    @Environment(\.colorScheme) private var colorScheme
    @Environment(AudioPlayerController.self) private var playerController
    @State private var selection: MainDestination = .home
    @State private var playerPresentationProgress: CGFloat = 0
#if !targetEnvironment(macCatalyst)
    @State private var animatedContentWidth: CGFloat = 0
    @State private var hasAnimatedContentWidth = false
#endif

    var body: some View {
        GeometryReader { geometry in
            let containerHeight = max(geometry.size.height + geometry.safeAreaInsets.top + geometry.safeAreaInsets.bottom, 1)

            #if !targetEnvironment(macCatalyst)
            let tabView = MainTabView(
                selectedTab: $selection,
                animatedContentWidth: $animatedContentWidth,
                hasAnimatedContentWidth: $hasAnimatedContentWidth
            )
            #else
            let tabView = MainTabView(selectedTab: $selection)
            #endif

            tabView
                .miniPlayerInset(
                    playerPresentationProgress: $playerPresentationProgress,
                    onTap: { openPlayer(animated: true) }
                )
            .overlay(alignment: .top) {
                if playerPresentationProgress > 0.0001 {
                    PlayerView(onClose: { closePlayer(animated: true) })
                        .frame(width: geometry.size.width, height: containerHeight)
                        .frame(maxWidth: .infinity, alignment: .top)
                        .ignoresSafeArea()
                        .offset(y: (1 - playerPresentationProgress) * containerHeight)
                        .simultaneousGesture(fullPlayerDismissGesture(containerHeight: containerHeight))
                        .allowsHitTesting(playerPresentationProgress > 0.01)
                        .zIndex(30)
                }
            }
            .onChange(of: playerController.snapshot.currentSongId) { _, newSongId in
                if newSongId == nil {
                    closePlayer(animated: false)
                }
            }
        }
        .tint(AureliaPalette.tint(for: colorScheme))
        .animation(.spring(response: 0.4, dampingFraction: 0.85), value: playerController.snapshot.currentSongId)
        .onReceive(NotificationCenter.default.publisher(for: .aureliaMenuCommand)) { notification in
            guard let command = notification.object as? AureliaMenuCommand else { return }
            handleMenuCommand(command)
        }
    }

    private func openPlayer(animated: Bool) {
        if playerPresentationProgress <= 0 {
            playerPresentationProgress = 0.0001
        }
        let action = { playerPresentationProgress = 1 }
        if animated {
            withAnimation(.interactiveSpring(response: 0.34, dampingFraction: 0.88)) {
                action()
            }
        } else {
            action()
        }
    }

    private func closePlayer(animated: Bool) {
        let action = { playerPresentationProgress = 0 }
        if animated {
            withAnimation(.interactiveSpring(response: 0.34, dampingFraction: 0.9)) {
                action()
            }
        } else {
            action()
        }
    }

    private func fullPlayerDismissGesture(containerHeight: CGFloat) -> some Gesture {
        DragGesture(minimumDistance: 6)
            .onChanged { value in
                guard shouldTrackFullPlayerDismissDrag(value) else { return }
                let closeDistance = max(containerHeight * 0.8, 420)
                let progress = 1 - min(max(value.translation.height / closeDistance, 0), 1)
                playerPresentationProgress = progress
            }
            .onEnded { value in
                guard shouldTrackFullPlayerDismissDrag(value) else { return }
                let projectedDownDistance = max(value.translation.height, value.predictedEndTranslation.height)
                let shouldClose = projectedDownDistance > containerHeight * 0.18 || playerPresentationProgress < 0.6
                if shouldClose {
                    closePlayer(animated: true)
                } else {
                    openPlayer(animated: true)
                }
            }
    }

    private func shouldTrackFullPlayerDismissDrag(_ value: DragGesture.Value) -> Bool {
        let height = value.translation.height
        let width = value.translation.width
        let startsNearTop = value.startLocation.y <= 180
        return startsNearTop && height > 0 && abs(height) > abs(width) * 0.8
    }

    private func handleMenuCommand(_ command: AureliaMenuCommand) {
        switch command {
        case .goHome:
            selection = .home
        case .goSongs:
            selection = .songs
        case .goAlbums:
            selection = .albums
        case .goArtists:
            selection = .artists
        case .goSearch:
            selection = .search
        case .goSettings:
            selection = .settings
        case .openNowPlaying:
            guard playerController.snapshot.currentSongId != nil else { return }
            openPlayer(animated: true)
        case .togglePlayPause:
            if playerController.snapshot.isPlaying {
                playerController.pause()
            } else {
                playerController.resume()
            }
        case .nextTrack:
            playerController.skipNext()
        case .previousTrack:
            playerController.skipPrevious()
        case .toggleShuffle:
            playerController.toggleShuffle()
        case .cycleRepeatMode:
            playerController.cycleRepeatMode()
        }
    }
}

private struct MiniPlayerInsetModifier: ViewModifier {
    @Environment(AudioPlayerController.self) private var playerController
    @Binding var playerPresentationProgress: CGFloat
    var onTap: () -> Void

    func body(content: Content) -> some View {
        content.safeAreaInset(edge: .bottom) {
            if playerController.snapshot.currentSongId != nil && playerPresentationProgress < 0.999 {
                MiniPlayerView(onTap: onTap)
                    .padding(.horizontal, AureliaSpacing.m)
                    .padding(.top, AureliaSpacing.s)
                    .opacity(Double(max(CGFloat(0), CGFloat(1) - playerPresentationProgress * CGFloat(1.4))))
                    .offset(y: playerPresentationProgress * 42)
                    .allowsHitTesting(playerPresentationProgress < 0.95)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
    }
}

private extension View {
    func miniPlayerInset(
        playerPresentationProgress: Binding<CGFloat>,
        onTap: @escaping () -> Void
    ) -> some View {
        modifier(
            MiniPlayerInsetModifier(
                playerPresentationProgress: playerPresentationProgress,
                onTap: onTap
            )
        )
    }
}
