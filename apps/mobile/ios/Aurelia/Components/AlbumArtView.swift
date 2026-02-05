import SwiftUI
import UIKit

struct AlbumArtView: View {
    let url: String?
    var size: ArtSize = .medium
    var customDimension: CGFloat? = nil
    @State private var loadedImage: UIImage? = nil
    @State private var isLoading = false

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
        Group {
            if let loadedImage {
                Image(uiImage: loadedImage)
                    .resizable()
                    .scaledToFill()
            } else if isLoading {
                placeholder
                    .overlay { ProgressView() }
            } else {
                placeholder
            }
        }
        .frame(width: dimension, height: dimension)
        .clipShape(RoundedRectangle(cornerRadius: radius, style: .continuous))
        .background(
            RoundedRectangle(cornerRadius: radius, style: .continuous)
                .fill(.quaternary)
        )
        .overlay(
            RoundedRectangle(cornerRadius: radius, style: .continuous)
                .stroke(Color.white.opacity(0.18), lineWidth: 1)
        )
        .task(id: url) {
            await loadImage()
        }
    }

    private var placeholder: some View {
        ZStack {
            Color(.systemGray5)
            Image(systemName: "music.note")
                .font(.system(size: (customDimension ?? size.dimension) * 0.3))
                .foregroundStyle(.secondary)
        }
    }

    @MainActor
    private func loadImage() async {
        loadedImage = nil
        guard let urlString = url, let imageUrl = URL(string: urlString) else { return }

        isLoading = true
        if let cached = await ImageCache.shared.cachedImage(for: imageUrl) {
            loadedImage = cached
            isLoading = false
            return
        }

        do {
            let (data, _) = try await URLSession.shared.data(from: imageUrl)
            if let image = UIImage(data: data) {
                ImageCache.shared.store(image, for: imageUrl)
                loadedImage = image
            }
        } catch {
            // Ignore errors, keep placeholder
        }

        isLoading = false
    }
}
