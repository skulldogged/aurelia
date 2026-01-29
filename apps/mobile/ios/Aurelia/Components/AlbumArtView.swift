import SwiftUI

struct AlbumArtView: View {
    let url: String?
    var size: ArtSize = .medium

    enum ArtSize {
        case small, medium, large, extraLarge

        var dimension: CGFloat {
            switch self {
            case .small: 48
            case .medium: 140
            case .large: 220
            case .extraLarge: 300
            }
        }

        var cornerRadius: CGFloat {
            switch self {
            case .small: 6
            case .medium: 10
            case .large: 14
            case .extraLarge: 16
            }
        }
    }

    var body: some View {
        AsyncImage(url: url.flatMap { URL(string: $0) }) { phase in
            switch phase {
            case .success(let image):
                image
                    .resizable()
                    .scaledToFill()
            case .failure:
                placeholder
            case .empty:
                placeholder
                    .overlay { ProgressView() }
            @unknown default:
                placeholder
            }
        }
        .frame(width: size.dimension, height: size.dimension)
        .clipShape(RoundedRectangle(cornerRadius: size.cornerRadius))
        .background(
            RoundedRectangle(cornerRadius: size.cornerRadius)
                .fill(.quaternary)
        )
    }

    private var placeholder: some View {
        ZStack {
            Color(.systemGray5)
            Image(systemName: "music.note")
                .font(.system(size: size.dimension * 0.3))
                .foregroundStyle(.secondary)
        }
    }
}
