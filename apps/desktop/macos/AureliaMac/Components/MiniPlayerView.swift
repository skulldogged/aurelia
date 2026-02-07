import SwiftUI

struct MiniPlayerView: View {
    @Environment(\.colorScheme) private var colorScheme
    @EnvironmentObject private var playerController: AudioPlayerController
    var onTap: () -> Void

    private var snapshot: PlayerSnapshot {
        playerController.snapshot
    }

    var body: some View {
        if snapshot.currentSongId != nil {
            HStack {
                HStack(spacing: AureliaSpacing.m) {
                    leftControls
                        .frame(maxWidth: .infinity, alignment: .leading)

                    centerNowPlaying
                        .frame(maxWidth: .infinity)

                    rightControls
                        .frame(maxWidth: .infinity, alignment: .trailing)
                }
                .padding(.horizontal, AureliaSpacing.m)
                .padding(.vertical, AureliaSpacing.s)
                .frame(minHeight: 64)
                .frame(maxWidth: 860)
                .tint(.primary)
                .glassEffect()
                .clipShape(Capsule())
                .overlay {
                    Capsule()
                        .stroke(Color.white.opacity(colorScheme == .dark ? 0.16 : 0.26), lineWidth: 0.8)
                        .allowsHitTesting(false)
                }
                .shadow(
                    color: Color.black.opacity(colorScheme == .dark ? 0.34 : 0.10),
                    radius: 14,
                    x: 0,
                    y: 8
                )
            }
            .frame(maxWidth: .infinity)
            .padding(.horizontal, AureliaSpacing.l)
            .padding(.top, AureliaSpacing.s)
        }
    }

    private var leftControls: some View {
        HStack(spacing: 8) {
            miniControlButton(systemName: "backward.fill", isEnabled: snapshot.hasPrevious) {
                playerController.skipPrevious()
            }

            playPauseButton

            miniControlButton(systemName: "forward.fill", isEnabled: snapshot.hasNext) {
                playerController.skipNext()
            }
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
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(.horizontal, AureliaSpacing.s)
            .padding(.vertical, AureliaSpacing.xs)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    private var rightControls: some View {
        HStack(spacing: 16) {
            if snapshot.durationMs > 0 {
                Text("\(TimeFormatter.formatDuration(snapshot.positionMs)) / \(TimeFormatter.formatDuration(snapshot.durationMs))")
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.secondary)
            }

            miniControlButton(systemName: "music.note.list", action: onTap)
        }
    }

    private var playPauseButton: some View {
        Button {
            playerController.togglePlayPause()
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
        isEnabled: Bool = true,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 17, weight: .semibold))
                .frame(width: 44, height: 44)
                .contentShape(Rectangle())
                .foregroundStyle(isEnabled ? Color.secondary : Color.secondary.opacity(0.45))
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }
}
