import AppKit
import CryptoKit
import Foundation
import ImageIO
import os

final class ImageCache: @unchecked Sendable {
    static let shared = ImageCache()

    private let memoryCache = NSCache<NSString, NSImage>()
    private let fileManager = FileManager.default
    private let ioQueue = DispatchQueue(label: "com.aurelia.macos.imagecache", qos: .utility)
    private let logger = Logger(subsystem: "com.aurelia.macos", category: "ImageCache")
    private let defaultMaxPixelSize = 1024
    private let inFlightLock = NSLock()
    private var inFlight: [String: InFlightTask] = [:]

    private init() {
        memoryCache.countLimit = 300
        memoryCache.totalCostLimit = 140 * 1024 * 1024
    }

    func cachedImage(for url: URL, targetSize: CGSize? = nil, scale: CGFloat = 2.0) async -> NSImage? {
        let variant = cacheVariant(for: url, targetSize: targetSize, scale: scale)
        if let image = memoryCache.object(forKey: variant.memoryKey) {
            return image
        }

        guard let diskURL = diskURL(for: url),
              let data = await readData(from: diskURL),
              let image = await decodeImageAsync(from: data, maxPixelSize: variant.maxPixelSize) else {
            return nil
        }

        memoryCache.setObject(image, forKey: variant.memoryKey, cost: image.memoryCost)
        return image
    }

    func fetchImage(for url: URL, targetSize: CGSize? = nil, scale: CGFloat = 2.0) async -> NSImage? {
        let variant = cacheVariant(for: url, targetSize: targetSize, scale: scale)
        if let cached = await cachedImage(for: url, targetSize: targetSize, scale: scale) {
            return cached
        }

        let key = variant.memoryKey as String
        if let existing = inFlightTask(for: key) {
            return await existing.task.value
        }

        let created = InFlightTask(id: UUID(), task: Task { [weak self] in
            guard let self else { return nil }

            do {
                let (data, _) = try await URLSession.shared.data(from: url)
                guard !data.isEmpty else { return nil }

                guard let image = await self.decodeImageAsync(from: data, maxPixelSize: variant.maxPixelSize) else {
                    return nil
                }

                self.memoryCache.setObject(image, forKey: variant.memoryKey, cost: image.memoryCost)
                self.storeDataAsync(data, for: url)
                return image
            } catch {
                self.logger.debug("Image fetch failed: \(error)")
                return nil
            }
        })

        let taskToAwait = upsertInFlightTask(created, for: key)
        let image = await taskToAwait.task.value
        clearInFlightTaskIfMatching(id: taskToAwait.id, for: key)
        return image
    }

    func clear() {
        memoryCache.removeAllObjects()
        ioQueue.async {
            guard let dir = self.cacheDirectoryURL() else { return }
            try? self.fileManager.removeItem(at: dir)
        }
    }

    private func inFlightTask(for key: String) -> InFlightTask? {
        inFlightLock.lock()
        defer { inFlightLock.unlock() }
        return inFlight[key]
    }

    private func upsertInFlightTask(_ candidate: InFlightTask, for key: String) -> InFlightTask {
        inFlightLock.lock()
        defer { inFlightLock.unlock() }
        if let existing = inFlight[key] {
            return existing
        }
        inFlight[key] = candidate
        return candidate
    }

    private func clearInFlightTaskIfMatching(id: UUID, for key: String) {
        inFlightLock.lock()
        defer { inFlightLock.unlock() }
        guard inFlight[key]?.id == id else { return }
        inFlight.removeValue(forKey: key)
    }

    private func readData(from url: URL) async -> Data? {
        await withCheckedContinuation { continuation in
            ioQueue.async {
                continuation.resume(returning: try? Data(contentsOf: url, options: .mappedIfSafe))
            }
        }
    }

    private func decodeImageAsync(from data: Data, maxPixelSize: Int) async -> NSImage? {
        await withCheckedContinuation { continuation in
            ioQueue.async {
                continuation.resume(returning: self.decodeImage(from: data, maxPixelSize: maxPixelSize))
            }
        }
    }

    private func storeDataAsync(_ data: Data, for url: URL) {
        ioQueue.async { [weak self] in
            guard let self else { return }
            guard let diskURL = self.diskURL(for: url) else { return }

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

    private func decodeImage(from data: Data, maxPixelSize: Int) -> NSImage? {
        let sourceOptions = [kCGImageSourceShouldCache: false] as CFDictionary
        guard let source = CGImageSourceCreateWithData(data as CFData, sourceOptions) else {
            return nil
        }

        let thumbnailOptions: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceThumbnailMaxPixelSize: maxPixelSize,
            kCGImageSourceShouldCacheImmediately: true,
            kCGImageSourceShouldCache: true,
        ]

        guard let cgImage = CGImageSourceCreateThumbnailAtIndex(source, 0, thumbnailOptions as CFDictionary) else {
            return nil
        }

        return NSImage(cgImage: cgImage, size: NSSize(width: cgImage.width, height: cgImage.height))
    }

    private func cacheVariant(for url: URL, targetSize: CGSize?, scale: CGFloat) -> (memoryKey: NSString, maxPixelSize: Int) {
        let maxPixelSize = maxPixelSize(for: targetSize, scale: scale)
        let baseKey = cacheKey(for: url) as String
        return (NSString(string: "\(baseKey)#\(maxPixelSize)"), maxPixelSize)
    }

    private func maxPixelSize(for targetSize: CGSize?, scale: CGFloat) -> Int {
        guard let targetSize, targetSize.width > 0, targetSize.height > 0 else {
            return defaultMaxPixelSize
        }

        let longestSide = max(targetSize.width, targetSize.height)
        let pixels = Int((longestSide * max(scale, 1)).rounded(.up))
        return min(max(pixels, 64), 4096)
    }

    private func cacheKey(for url: URL) -> NSString {
        NSString(string: url.absoluteString)
    }

    private func cacheDirectoryURL() -> URL? {
        guard let caches = fileManager.urls(for: .cachesDirectory, in: .userDomainMask).first else { return nil }
        return caches.appendingPathComponent("com.aurelia.macos/image-cache", isDirectory: true)
    }

    private func diskURL(for url: URL) -> URL? {
        guard let dir = cacheDirectoryURL() else { return nil }
        let hash = sha256(url.absoluteString)
        return dir.appendingPathComponent("\(hash).bin")
    }

    private func sha256(_ input: String) -> String {
        let data = Data(input.utf8)
        let digest = SHA256.hash(data: data)
        return digest.map { String(format: "%02x", $0) }.joined()
    }
}

private extension NSImage {
    var memoryCost: Int {
        guard let cgImage = cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            return 1
        }
        return cgImage.bytesPerRow * cgImage.height
    }
}

private struct InFlightTask {
    let id: UUID
    let task: Task<NSImage?, Never>
}
