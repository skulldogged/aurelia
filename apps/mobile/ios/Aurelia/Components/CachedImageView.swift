import SwiftUI
import UIKit

struct CachedImageView: View {
    let url: URL?
    var contentMode: ContentMode = .fill
    var placeholderColor: Color = Color(.systemGray5)
    var targetSize: CGSize? = nil

    @State private var loadedImage: UIImage? = nil
    @State private var loadedKey: String? = nil

    var body: some View {
        Group {
            if let loadedImage {
                Image(uiImage: loadedImage)
                    .resizable()
                    .aspectRatio(contentMode: contentMode)
            } else {
                placeholderColor
            }
        }
        .task(id: requestKey()) {
            await loadImage()
        }
    }

    @MainActor
    private func loadImage() async {
        guard let requestKey = requestKey(), let url else {
            loadedImage = nil
            loadedKey = nil
            return
        }

        if loadedKey != requestKey {
            loadedImage = nil
            loadedKey = requestKey
        } else if loadedImage != nil {
            return
        }

        loadedImage = await ImageCache.shared.fetchImage(for: url, targetSize: targetSize)
    }

    private func requestKey() -> String? {
        guard let url else { return nil }
        let width = Int((targetSize?.width ?? 0).rounded(.up))
        let height = Int((targetSize?.height ?? 0).rounded(.up))
        return "\(url.absoluteString)#\(width)x\(height)"
    }
}
