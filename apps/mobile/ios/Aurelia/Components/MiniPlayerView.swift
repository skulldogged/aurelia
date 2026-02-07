import SwiftUI
import os
import AureliaCore

struct MiniPlayerView: View {
    @Environment(\.horizontalSizeClass) private var sizeClass
    @Environment(\.colorScheme) private var colorScheme
    @Environment(AudioPlayerController.self) private var playerController
    @State private var favoriteCache: [String: Bool] = [:]
    @State private var isFavoriteLoading = false
    private let logger = Logger(subsystem: "com.aurelia.app", category: "MiniPlayerView")
    var onTap: () -> Void

    private var snapshot: PlayerSnapshot {
        playerController.snapshot
    }

    private var currentSong: Song? {
        let queue = playerController.getQueue()
        let currentIndex = playerController.getCurrentQueueIndex()
        guard currentIndex >= 0, currentIndex < queue.count else { return nil }
        return queue[currentIndex]
    }

    private var isFavorite: Bool {
        guard let songId = snapshot.currentSongId else { return false }
        if let cached = favoriteCache[songId] {
            return cached
        }
        return currentSong?.isFavorite ?? false
    }

    var body: some View {
        Group {
            if sizeClass == .regular {
                iPadMiniPlayerBar
            } else {
                compactMiniPlayerBar
            }
        }
        .onAppear {
            cacheFavoriteForCurrentSongIfNeeded()
        }
        .onChange(of: snapshot.currentSongId) { _, _ in
            cacheFavoriteForCurrentSongIfNeeded()
        }
    }

    private var iPadMiniPlayerBar: some View {
        HStack(spacing: AureliaSpacing.m) {
            leftControls
                .frame(maxWidth: .infinity, alignment: .leading)

            centerNowPlaying
                .frame(maxWidth: .infinity)

            rightActions
                .frame(maxWidth: .infinity, alignment: .trailing)
        }
        .padding(.horizontal, AureliaSpacing.m)
        .padding(.vertical, AureliaSpacing.s)
        .tint(.primary)
        .frame(minHeight: 64)
        .glassEffect()
        .clipShape(Capsule())
        .overlay {
            Capsule()
                .fill(Color.black.opacity(colorScheme == .dark ? 0.22 : 0.16))
                .allowsHitTesting(false)
        }
        .overlay {
            Capsule()
                .stroke(Color.white.opacity(colorScheme == .dark ? 0.16 : 0.24), lineWidth: 0.8)
                .allowsHitTesting(false)
        }
        .shadow(
            color: Color.black.opacity(colorScheme == .dark ? 0.38 : 0.12),
            radius: 16,
            x: 0,
            y: 8
        )
    }

    private var compactMiniPlayerBar: some View {
        Button(action: onTap) {
            GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.s) {
                HStack(spacing: 12) {
                    AlbumArtView(url: snapshot.albumArtUrl, size: .small)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(snapshot.title)
                            .font(.subheadline.weight(.medium))
                            .lineLimit(1)
                        Text(snapshot.artist)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }

                    Spacer()

                    HStack(spacing: 16) {
                        Button {
                            playerController.skipPrevious()
                        } label: {
                            Image(systemName: "backward.fill")
                        }
                        .disabled(!snapshot.hasPrevious)

                        Button {
                            togglePlayPause()
                        } label: {
                            Image(systemName: snapshot.isPlaying ? "pause.fill" : "play.fill")
                                .font(.system(size: 24, weight: .bold))
                        }

                        Button {
                            playerController.skipNext()
                        } label: {
                            Image(systemName: "forward.fill")
                        }
                        .disabled(!snapshot.hasNext)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .buttonStyle(.plain)
    }

    private var leftControls: some View {
        HStack(spacing: 16) {
            miniControlButton(
                systemName: "shuffle",
                isActive: snapshot.isShuffled,
                action: { playerController.toggleShuffle() }
            )

            miniControlButton(
                systemName: "backward.fill",
                isEnabled: snapshot.hasPrevious,
                action: { playerController.skipPrevious() }
            )

            playPauseButton

            miniControlButton(
                systemName: "forward.fill",
                isEnabled: snapshot.hasNext,
                action: { playerController.skipNext() }
            )

            miniControlButton(
                systemName: snapshot.repeatMode == .one ? "repeat.1" : "repeat",
                isActive: snapshot.repeatMode != .none,
                action: { playerController.cycleRepeatMode() }
            )
        }
    }

    private var centerNowPlaying: some View {
        Button(action: onTap) {
            HStack(spacing: AureliaSpacing.s) {
                AlbumArtView(url: snapshot.albumArtUrl, size: .small)
                    .shadow(color: Color.black.opacity(0.18), radius: 6, x: 0, y: 3)

                VStack(alignment: .leading, spacing: 2) {
                    Text(snapshot.title)
                        .font(.subheadline.weight(.semibold))
                        .lineLimit(1)
                        .foregroundStyle(.primary)

                    Text(snapshot.artist)
                        .font(.caption)
                        .lineLimit(1)
                        .foregroundStyle(.secondary)
                }
            }
            .padding(.horizontal, AureliaSpacing.s)
            .padding(.vertical, AureliaSpacing.xs)
            .frame(maxWidth: .infinity, alignment: .center)
        }
        .buttonStyle(.plain)
    }

    private var rightActions: some View {
        HStack(spacing: 16) {
            miniControlButton(
                systemName: "quote.bubble.fill",
                action: onTap
            )

            miniControlButton(
                systemName: "list.bullet",
                action: onTap
            )

            miniControlButton(
                systemName: isFavorite ? "heart.fill" : "heart",
                tint: isFavorite ? .red : .secondary,
                isEnabled: !isFavoriteLoading,
                action: toggleFavorite
            )
        }
    }

    private var playPauseButton: some View {
        Button {
            togglePlayPause()
        } label: {
            Image(systemName: snapshot.isPlaying ? "pause.fill" : "play.fill")
                .font(.system(size: 22, weight: .bold))
                .frame(width: 44, height: 44)
                .foregroundStyle(.primary)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    private func miniControlButton(
        systemName: String,
        tint: Color? = nil,
        isEnabled: Bool = true,
        isActive: Bool = false,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 17, weight: .semibold))
                .frame(width: 44, height: 44)
                .contentShape(Rectangle())
                .foregroundStyle(tint ?? (isActive ? .primary : .secondary))
                .opacity(isEnabled ? 1 : 0.4)
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }

    private func togglePlayPause() {
        if snapshot.isPlaying {
            playerController.pause()
        } else {
            playerController.resume()
        }
    }

    private func cacheFavoriteForCurrentSongIfNeeded() {
        guard let song = currentSong else { return }
        if favoriteCache[song.id] == nil {
            favoriteCache[song.id] = song.isFavorite ?? false
        }
    }

    private func toggleFavorite() {
        guard !isFavoriteLoading else { return }
        guard let songId = snapshot.currentSongId else { return }

        let sessionStore = SessionStore.shared
        guard let serverUrl = sessionStore.serverUrl,
              let token = sessionStore.token,
              let userId = sessionStore.userId else { return }

        let isCurrentlyFavorite = isFavorite
        let targetFavoriteState = !isCurrentlyFavorite
        isFavoriteLoading = true

        Task.detached {
            do {
                let newState = try await AureliaCore.toggleFavorite(
                    serverUrl: serverUrl,
                    token: token,
                    userId: userId,
                    itemId: songId,
                    isFavorite: targetFavoriteState
                )
                await MainActor.run {
                    favoriteCache[songId] = newState
                    isFavoriteLoading = false
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    logger.error("Failed to toggle favorite from mini player: \(error)")
                }
                await MainActor.run {
                    isFavoriteLoading = false
                }
            }
        }
    }

}
