import SwiftUI

struct FeaturedHeroView: View {
    let albums: [FeaturedAlbum]
    let isWide: Bool
    let availableWidth: CGFloat
    let onSelect: (FeaturedAlbum) -> Void

    @Environment(\.colorScheme) private var colorScheme
    @State private var scrollIndex: Int? = 0
    @State private var scrollOffset: CGFloat = 0
    @State private var lastSettledIndex: Int = 0
    @State private var lastScrollTime: Date = .distantPast
    @State private var isScrolling = false
    @State private var scrollDebounceTask: Task<Void, Never>? = nil

    private let autoAdvanceSeconds: UInt64 = 12

    var body: some View {
        let cardHeight: CGFloat = isWide ? 260 : 220
        let horizontalPadding = AureliaSpacing.m
        let padding = AureliaSpacing.l
        let imageSize: CGFloat = cardHeight - (padding * 2)
        let cardWidth = max(1, availableWidth - (horizontalPadding * 2))
        let count = albums.count
        let maxOffset = max(0, CGFloat(count - 1) * cardWidth)
        let clampedOffset = min(max(scrollOffset, 0), maxOffset)
        let fractionalIndex = count > 0 ? clampedOffset / cardWidth : 0
        let baseIndex = min(max(Int(floor(fractionalIndex)), 0), max(count - 1, 0))
        let progress = min(max(fractionalIndex - CGFloat(baseIndex), 0), 1)
        let nextIndex = min(baseIndex + 1, max(count - 1, 0))
        let baseAlbum = albums[safe: baseIndex]
        let nextAlbum = albums[safe: nextIndex]
        let selectedAlbum = progress > 0.5 ? (nextAlbum ?? baseAlbum) : baseAlbum

        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            AureliaSectionHeader(title: "Featured", subtitle: "Fresh picks for you")

            if let baseAlbum {
                ZStack {
                    if let nextAlbum, nextAlbum.id != baseAlbum.id {
                        featuredCard(for: nextAlbum, imageSize: imageSize, cardHeight: cardHeight)
                            .opacity(progress)
                            .allowsHitTesting(false)
                    }

                    featuredCard(for: baseAlbum, imageSize: imageSize, cardHeight: cardHeight)
                        .opacity(1 - progress)
                        .allowsHitTesting(false)
                }
                .frame(width: cardWidth, height: cardHeight)
                .padding(.horizontal, horizontalPadding)
                .overlay {
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: 0) {
                            ForEach(0..<max(count, 1), id: \.self) { index in
                                Color.clear
                                    .frame(width: cardWidth, height: cardHeight)
                                    .id(index)
                            }
                        }
                        .scrollTargetLayout()
                        .background(
                            GeometryReader { proxy in
                                Color.clear
                                    .preference(
                                        key: FeaturedScrollOffsetKey.self,
                                        value: -proxy.frame(in: .named("featuredScroll")).minX
                                    )
                            }
                        )
                    }
                    .scrollTargetBehavior(.paging)
                    .scrollPosition(id: $scrollIndex)
                    .coordinateSpace(name: "featuredScroll")
                    .frame(width: cardWidth, height: cardHeight)
                    .contentShape(Rectangle())
                    .onPreferenceChange(FeaturedScrollOffsetKey.self) { value in
                        handleScrollOffset(value, cardWidth: cardWidth, count: count)
                    }
                    .onChange(of: scrollIndex) { _, newIndex in
                        guard let newIndex else { return }
                        let lowerBound = max(lastSettledIndex - 1, 0)
                        let upperBound = min(lastSettledIndex + 1, max(count - 1, 0))
                        let target = clamp(newIndex, lower: lowerBound, upper: upperBound)
                        if target != newIndex {
                            withAnimation(.easeInOut(duration: 0.35)) {
                                scrollIndex = target
                            }
                        }
                        lastSettledIndex = target
                    }
                    .onTapGesture {
                        guard !isScrolling else { return }
                        guard Date().timeIntervalSince(lastScrollTime) > 0.25 else { return }
                        if let selectedAlbum {
                            onSelect(selectedAlbum)
                        }
                    }
                }
                .task(id: albums.map(\.id)) {
                    await startAutoAdvance(count: count)
                }
                .onAppear {
                    let initialIndex = min(scrollIndex ?? 0, max(count - 1, 0))
                    scrollIndex = initialIndex
                    lastSettledIndex = initialIndex
                }
            }
        }
    }

    private func featuredCard(for album: FeaturedAlbum, imageSize: CGFloat, cardHeight: CGFloat) -> some View {
        HStack(spacing: AureliaSpacing.m) {
            AlbumArtView(url: album.albumArtUrl, size: .medium, customDimension: imageSize)
                .shadow(color: .black.opacity(0.25), radius: 10, x: 0, y: 6)

            VStack(alignment: .leading, spacing: 10) {
                Text("Featured Album")
                    .font(.caption)
                    .foregroundColor(Color.white.opacity(0.85))
                    .shadow(color: .black.opacity(0.25), radius: 2, x: 0, y: 1)

                Text(album.name)
                    .font(isWide ? .title2.bold() : .title3.bold())
                    .foregroundColor(.white)
                    .lineLimit(2)
                    .shadow(color: .black.opacity(0.35), radius: 2, x: 0, y: 1)

                Text(album.artist)
                    .font(.subheadline)
                    .foregroundColor(Color.white.opacity(0.9))
                    .lineLimit(1)
                    .shadow(color: .black.opacity(0.3), radius: 2, x: 0, y: 1)

                Spacer()

                Text("Open album")
                    .font(.caption)
                    .foregroundColor(Color.white.opacity(0.8))
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        }
        .padding(AureliaSpacing.l)
        .frame(height: cardHeight)
        .background(featuredBackground(for: album))
        .clipShape(RoundedRectangle(cornerRadius: AureliaRadius.l, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: AureliaRadius.l, style: .continuous)
                .stroke(AureliaPalette.glassBorder(for: colorScheme), lineWidth: 1)
        )
    }

    private func featuredBackground(for album: FeaturedAlbum) -> some View {
        ZStack {
            CachedImageView(url: album.albumArtUrl.flatMap { URL(string: $0) }, contentMode: .fill)
                .blur(radius: 28)
                .scaleEffect(1.2)
                .saturation(0.8)
                .brightness(-0.1)

            Color.black.opacity(colorScheme == .dark ? 0.52 : 0.45)
        }
    }

    private func handleScrollOffset(_ value: CGFloat, cardWidth: CGFloat, count: Int) {
        guard count > 0 else { return }
        let maxOffset = CGFloat(count - 1) * cardWidth
        let clamped = min(max(value, 0), maxOffset)
        guard abs(clamped - scrollOffset) > 0.5 else { return }

        scrollOffset = clamped
        lastScrollTime = Date()
        isScrolling = true

        scrollDebounceTask?.cancel()
        scrollDebounceTask = Task { @MainActor in
            try? await Task.sleep(nanoseconds: 200_000_000)
            isScrolling = false
        }
    }

    private func clamp(_ value: Int, lower: Int, upper: Int) -> Int {
        min(max(value, lower), upper)
    }

    private func startAutoAdvance(count: Int) async {
        guard count > 1 else { return }
        while !Task.isCancelled {
            try? await Task.sleep(nanoseconds: autoAdvanceSeconds * 1_000_000_000)
            let shouldAdvance = await MainActor.run {
                !isScrolling && Date().timeIntervalSince(lastScrollTime) > 1.5
            }
            guard shouldAdvance else { continue }

            await MainActor.run {
                let current = scrollIndex ?? 0
                let next = (current + 1) % count
                withAnimation(.easeInOut(duration: 0.7)) {
                    scrollIndex = next
                }
                lastSettledIndex = next
            }
        }
    }
}

private extension Collection {
    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

private struct FeaturedScrollOffsetKey: PreferenceKey {
    static var defaultValue: CGFloat = 0

    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}
