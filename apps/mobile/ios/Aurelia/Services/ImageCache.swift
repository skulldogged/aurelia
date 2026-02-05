import Foundation
import UIKit
import CryptoKit
import os

final class ImageCache {
    static let shared = ImageCache()

    private let memoryCache = NSCache<NSString, UIImage>()
    private let fileManager = FileManager.default
    private let ioQueue = DispatchQueue(label: "com.aurelia.imagecache")
    private let logger = Logger(subsystem: "com.aurelia.app", category: "ImageCache")

    private init() {
        memoryCache.countLimit = 200
    }

    func cachedImage(for url: URL) async -> UIImage? {
        let key = cacheKey(for: url)
        if let image = memoryCache.object(forKey: key) {
            return image
        }

        return await withCheckedContinuation { continuation in
            ioQueue.async {
                guard let diskURL = self.diskURL(for: url),
                      let data = try? Data(contentsOf: diskURL),
                      let image = UIImage(data: data) else {
                    continuation.resume(returning: nil)
                    return
                }

                self.memoryCache.setObject(image, forKey: key)
                continuation.resume(returning: image)
            }
        }
    }

    func store(_ image: UIImage, for url: URL) {
        let key = cacheKey(for: url)
        memoryCache.setObject(image, forKey: key)

        ioQueue.async {
            guard let diskURL = self.diskURL(for: url) else { return }
            guard let data = image.pngData() else { return }
            do {
                let directory = diskURL.deletingLastPathComponent()
                if !self.fileManager.fileExists(atPath: directory.path) {
                    try self.fileManager.createDirectory(at: directory, withIntermediateDirectories: true)
                }
                try data.write(to: diskURL, options: .atomic)
            } catch {
                self.logger.error("Failed to write image cache: \(error)")
            }
        }
    }

    func clear() {
        memoryCache.removeAllObjects()
        ioQueue.async {
            guard let appDataDir = SessionStore.shared.getAppDataDir(), !appDataDir.isEmpty else { return }
            let dir = URL(fileURLWithPath: appDataDir).appendingPathComponent("image-cache", isDirectory: true)
            try? self.fileManager.removeItem(at: dir)
        }
    }

    private func cacheKey(for url: URL) -> NSString {
        NSString(string: url.absoluteString)
    }

    private func diskURL(for url: URL) -> URL? {
        guard let appDataDir = SessionStore.shared.getAppDataDir(), !appDataDir.isEmpty else { return nil }
        let hash = sha256(url.absoluteString)
        let dir = URL(fileURLWithPath: appDataDir).appendingPathComponent("image-cache", isDirectory: true)
        return dir.appendingPathComponent("\(hash).png")
    }

    private func sha256(_ input: String) -> String {
        let data = Data(input.utf8)
        let digest = SHA256.hash(data: data)
        return digest.map { String(format: "%02x", $0) }.joined()
    }
}
