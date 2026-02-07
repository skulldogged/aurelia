import SwiftUI

struct CachedImageView: View {
    let url: URL?
    var contentMode: ContentMode = .fill
    var placeholderColor: Color = Color(nsColor: .windowBackgroundColor)
    var targetSize: CGSize? = nil

    @Environment(\.displayScale) private var displayScale

    @State private var loadedImage: NSImage?
    @State private var loadedKey: String?

    var body: some View {
        Group {
            if let loadedImage {
                Image(nsImage: loadedImage)
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

        loadedImage = await ImageCache.shared.fetchImage(
            for: url,
            targetSize: targetSize,
            scale: displayScale
        )
    }

    private func requestKey() -> String? {
        guard let url else { return nil }
        let width = Int((targetSize?.width ?? 0).rounded(.up))
        let height = Int((targetSize?.height ?? 0).rounded(.up))
        return "\(url.absoluteString)#\(width)x\(height)"
    }
}
