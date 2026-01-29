import SwiftUI

struct MiniPlayerView: View {
    @Environment(AudioPlayerController.self) private var playerController
    var onTap: () -> Void

    private var snapshot: PlayerSnapshot {
        playerController.snapshot
    }

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                // Album Art
                AlbumArtView(url: snapshot.albumArtUrl, size: .small)

                // Song Info
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

                // Controls
                HStack(spacing: 16) {
                    Button {
                        playerController.skipPrevious()
                    } label: {
                        Image(systemName: "backward.fill")
                    }
                    .disabled(!snapshot.hasPrevious)

                    Button {
                        if snapshot.isPlaying {
                            playerController.pause()
                        } else {
                            playerController.resume()
                        }
                    } label: {
                        Image(systemName: snapshot.isPlaying ? "pause.fill" : "play.fill")
                            .font(.title3)
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
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 16))
            .shadow(radius: 4, y: 2)
            .padding(.horizontal, 8)
        }
        .buttonStyle(.plain)
    }
}
