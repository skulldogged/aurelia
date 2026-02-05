import SwiftUI
import UIKit

struct CachedImageView: View {
    let url: URL?
    var contentMode: ContentMode = .fill
    var placeholderColor: Color = Color(.systemGray5)

    @State private var loadedImage: UIImage? = nil
    @State private var isLoading = false

    var body: some View {
        Group {
            if let loadedImage {
                Image(uiImage: loadedImage)
                    .resizable()
                    .aspectRatio(contentMode: contentMode)
            } else if isLoading {
                placeholderColor
                    .overlay { ProgressView() }
            } else {
                placeholderColor
            }
        }
        .task(id: url) {
            await loadImage()
        }
    }

    @MainActor
    private func loadImage() async {
        loadedImage = nil
        guard let url else { return }

        isLoading = true
        if let cached = await ImageCache.shared.cachedImage(for: url) {
            loadedImage = cached
            isLoading = false
            return
        }

        do {
            let (data, _) = try await URLSession.shared.data(from: url)
            if let image = UIImage(data: data) {
                ImageCache.shared.store(image, for: url)
                loadedImage = image
            }
        } catch {
            // Ignore errors
        }

        isLoading = false
    }
}
