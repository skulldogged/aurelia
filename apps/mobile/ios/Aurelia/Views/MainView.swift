import SwiftUI

struct MainView: View {
    @Environment(\.horizontalSizeClass) private var sizeClass
    @Environment(\.colorScheme) private var colorScheme
    @Environment(AudioPlayerController.self) private var playerController
    @State private var selection: MainDestination = .home
    @State private var playerPresentationProgress: CGFloat = 0

    var body: some View {
        GeometryReader { geometry in
            let containerHeight = max(geometry.size.height + geometry.safeAreaInsets.top + geometry.safeAreaInsets.bottom, 1)

            Group {
                if sizeClass == .regular {
                    MainSplitView(
                        selection: $selection,
                        playerPresentationProgress: $playerPresentationProgress,
                        onOpenPlayer: { openPlayer(animated: true) }
                    )
                } else {
                    MainTabView(selectedTab: $selection)
                        .miniPlayerInset(
                            playerPresentationProgress: $playerPresentationProgress,
                            onTap: { openPlayer(animated: true) }
                        )
                }
            }
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
}

struct MainSplitView: View {
    @Binding var selection: MainDestination
    @Binding var playerPresentationProgress: CGFloat
    var onOpenPlayer: () -> Void

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
                .miniPlayerInset(
                    playerPresentationProgress: $playerPresentationProgress,
                    onTap: onOpenPlayer
                )
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
