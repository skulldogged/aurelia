import SwiftUI

struct AlbumArtView: View {
    let url: String?
    var size: ArtSize = .medium
    var customDimension: CGFloat? = nil

    enum ArtSize {
        case small, medium, large

        var dimension: CGFloat {
            switch self {
            case .small: 44
            case .medium: 156
            case .large: 236
            }
        }

        var cornerRadius: CGFloat {
            switch self {
            case .small: 6
            case .medium: 10
            case .large: 14
            }
        }

        func cornerRadius(for dimension: CGFloat?) -> CGFloat {
            let baseRadius = cornerRadius
            let baseDimension = self.dimension
            guard let dimension else { return baseRadius }
            let ratio = baseRadius / baseDimension
            return max(6, dimension * ratio)
        }
    }

    var body: some View {
        let dimension = customDimension ?? size.dimension
        let radius = size.cornerRadius(for: dimension)

        CachedImageView(
            url: url.flatMap { URL(string: $0) },
            contentMode: ContentMode.fill,
            placeholderColor: Color.secondary.opacity(0.14),
            targetSize: CGSize(width: dimension, height: dimension)
        )
        .overlay {
            if url == nil {
                Image(systemName: "music.note")
                    .font(.system(size: dimension * 0.28))
                    .foregroundStyle(.secondary)
            }
        }
        .frame(width: dimension, height: dimension)
        .clipShape(RoundedRectangle(cornerRadius: radius, style: .continuous))
        .background(
            RoundedRectangle(cornerRadius: radius, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
        .overlay(
            RoundedRectangle(cornerRadius: radius, style: .continuous)
                .stroke(Color.white.opacity(0.16), lineWidth: 1)
        )
    }
}
