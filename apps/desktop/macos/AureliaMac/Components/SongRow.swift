import AureliaCore
import SwiftUI

struct SongRow: View {
    let song: Song
    var isPlaying: Bool = false
    var showTrackNumber: Bool = false
    var onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                if showTrackNumber {
                    Text("\(song.trackNumber ?? 0)")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(width: 32)
                } else {
                    AlbumArtView(url: song.albumArtUrl, size: .small)
                }

                VStack(alignment: .leading, spacing: 2) {
                    Text(song.name)
                        .font(.body.weight(.medium))
                        .foregroundStyle(isPlaying ? AnyShapeStyle(.tint) : AnyShapeStyle(.primary))
                        .lineLimit(1)
                    Text(song.artists?.joined(separator: ", ") ?? "")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                if let duration = song.duration {
                    Text(TimeFormatter.formatDuration(Int64(duration * 1000)))
                        .font(.subheadline.monospacedDigit())
                        .foregroundStyle(.secondary)
                }
            }
            .padding(.vertical, 9)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
