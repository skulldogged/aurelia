import SwiftUI
import AureliaCore

struct PlayerView: View {
    private enum Panel {
        case none
        case lyrics
        case queue
    }

    @Environment(\.colorScheme) private var colorScheme
    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.dismiss) private var dismiss

    @State private var viewModel = PlayerViewModel()
    @State private var isDragging = false
    @State private var dragPosition: Double = 0
    @State private var activePanel: Panel = .none

    var body: some View {
        GeometryReader { geometry in
            let isWide = UIDevice.current.userInterfaceIdiom == .pad || AureliaLayout.isWide(geometry.size.width)
            let horizontalInset: CGFloat = isWide ? AureliaSpacing.xxl : AureliaSpacing.m
            let panelVisible = activePanel != .none
            let contentWidth = max(geometry.size.width - (horizontalInset * 2), 320)
            let preferredPlayerWidth = min(CGFloat(520), contentWidth)
            let canShowSidePanel = isWide && panelVisible && contentWidth >= (preferredPlayerWidth + 280 + AureliaSpacing.xl)
            let panelWidth: CGFloat = canShowSidePanel
                ? min(CGFloat(360), max(CGFloat(280), contentWidth - preferredPlayerWidth - AureliaSpacing.xl))
                : 0
            let playerWidth = preferredPlayerWidth
            let baseArtSize = min(
                isWide ? 300 : 260,
                max(170, playerWidth - 120)
            )
            let contentTopPadding: CGFloat = 58
            let centeredContentHeight = max(0, geometry.size.height - contentTopPadding - AureliaSpacing.l)

            ZStack {
                Group {
                    if canShowSidePanel {
                        HStack(alignment: .center, spacing: AureliaSpacing.xl) {
                            primaryColumn(artSize: baseArtSize)
                                .frame(width: playerWidth)

                            panelView
                                .frame(width: panelWidth, height: centeredContentHeight, alignment: .top)
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
                        .padding(.horizontal, horizontalInset)
                        .padding(.top, contentTopPadding)
                        .padding(.bottom, AureliaSpacing.l)
                    } else if isWide {
                        VStack {
                            adaptivePrimaryColumn(
                                artSizes: [
                                    baseArtSize,
                                    max(170, baseArtSize - 30),
                                    max(150, baseArtSize - 55)
                                ],
                                playerWidth: playerWidth
                            )

                            if panelVisible {
                                panelView
                                    .frame(maxHeight: 420)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .frame(height: centeredContentHeight, alignment: .center)
                        .padding(.horizontal, horizontalInset)
                        .padding(.top, contentTopPadding)
                        .padding(.bottom, AureliaSpacing.l)
                    } else {
                        ScrollView(showsIndicators: false) {
                            VStack(spacing: AureliaSpacing.l) {
                                primaryColumn(artSize: baseArtSize)

                                if activePanel != .none {
                                    panelView
                                }
                            }
                            .padding(.horizontal, horizontalInset)
                            .padding(.top, contentTopPadding)
                            .padding(.bottom, AureliaSpacing.xl)
                        }
                    }
                }
            }
            .frame(width: geometry.size.width, height: geometry.size.height, alignment: .center)
            .background {
                PlayerBackdropView(albumArtUrl: viewModel.albumArtUrl)
            }
            .overlay(alignment: .top) {
                topBar
            }
        }
        .onChange(of: playerController.snapshot) { _, snapshot in
            viewModel.updateFrom(snapshot, playerController: playerController)
        }
        .onChange(of: viewModel.showLyrics) { _, isShown in
            if !isShown, activePanel == .lyrics {
                activePanel = .none
            }
        }
        .onAppear {
            viewModel.updateFrom(playerController.snapshot, playerController: playerController)
        }
    }

    private func adaptivePrimaryColumn(artSizes: [CGFloat], playerWidth: CGFloat) -> some View {
        ViewThatFits(in: .vertical) {
            ForEach(Array(artSizes.enumerated()), id: \.offset) { _, artSize in
                primaryColumn(artSize: artSize)
                    .frame(maxWidth: playerWidth)
            }

            ScrollView(showsIndicators: false) {
                primaryColumn(artSize: artSizes.last ?? 170)
                    .frame(maxWidth: playerWidth)
                    .padding(.vertical, AureliaSpacing.s)
            }
        }
    }

    private func primaryColumn(artSize: CGFloat) -> some View {
        VStack(spacing: AureliaSpacing.l) {
            AlbumArtView(url: viewModel.albumArtUrl, size: .extraLarge, customDimension: artSize)
                .shadow(color: Color.black.opacity(colorScheme == .dark ? 0.36 : 0.16), radius: 18, x: 0, y: 10)
                .overlay(
                    RoundedRectangle(cornerRadius: 18, style: .continuous)
                        .stroke(Color.white.opacity(colorScheme == .dark ? 0.18 : 0.30), lineWidth: 1)
                )

            VStack(spacing: AureliaSpacing.m) {
                songInfo
                progressSection
                playbackControls
                secondaryControls
            }
            .padding(.horizontal, AureliaSpacing.l)
            .padding(.vertical, AureliaSpacing.l)
        }
    }

    private var panelView: some View {
        Group {
            switch activePanel {
            case .lyrics:
                lyricsCard
            case .queue:
                queueCard
            case .none:
                EmptyView()
            }
        }
    }

    private var songInfo: some View {
        VStack(spacing: 4) {
            Text(viewModel.title)
                .font(.title2.weight(.bold))
                .lineLimit(2)
                .multilineTextAlignment(.center)
                .minimumScaleFactor(0.8)

            Text(viewModel.artist)
                .font(.headline.weight(.medium))
                .foregroundStyle(.white.opacity(0.82))
                .lineLimit(1)

            if let formatInfo = viewModel.formatInfo {
                Text(formatInfo)
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.58))
            }
        }
        .frame(maxWidth: .infinity)
    }

    private var progressSection: some View {
        VStack(spacing: 6) {
            Slider(
                value: Binding(
                    get: { isDragging ? dragPosition : Double(viewModel.positionMs) },
                    set: { newValue in
                        dragPosition = newValue
                        isDragging = true
                    }
                ),
                in: 0...max(Double(viewModel.durationMs), 1),
                onEditingChanged: { editing in
                    if !editing {
                        viewModel.seekTo(Int64(dragPosition), playerController: playerController)
                        isDragging = false
                    }
                }
            )
            .tint(.white)

            HStack {
                Text(TimeFormatter.formatDuration(isDragging ? Int64(dragPosition) : viewModel.positionMs))
                    .font(.caption2)
                    .foregroundStyle(.white.opacity(0.70))
                    .monospacedDigit()

                Spacer()

                Text(TimeFormatter.formatDuration(viewModel.durationMs))
                    .font(.caption2)
                    .foregroundStyle(.white.opacity(0.70))
                    .monospacedDigit()
            }
        }
    }

    private var playbackControls: some View {
        HStack(spacing: 28) {
            iconControl(
                systemName: "shuffle",
                isActive: viewModel.isShuffled,
                action: { viewModel.toggleShuffle(playerController: playerController) }
            )

            iconControl(
                systemName: "backward.fill",
                isEnabled: viewModel.hasPrevious,
                action: { viewModel.skipPrevious(playerController: playerController) }
            )

            Button {
                viewModel.togglePlayPause(playerController: playerController)
            } label: {
                Image(systemName: viewModel.isPlaying ? "pause.fill" : "play.fill")
                    .font(.system(size: 26, weight: .bold))
                    .frame(width: 68, height: 68)
                    .foregroundStyle(Color.black.opacity(0.9))
                    .background(Color.white, in: Circle())
            }
            .buttonStyle(.plain)

            iconControl(
                systemName: "forward.fill",
                isEnabled: viewModel.hasNext,
                action: { viewModel.skipNext(playerController: playerController) }
            )

            iconControl(
                systemName: viewModel.repeatMode == .one ? "repeat.1" : "repeat",
                isActive: viewModel.repeatMode != .none,
                action: { viewModel.cycleRepeatMode(playerController: playerController) }
            )
        }
    }

    private var secondaryControls: some View {
        HStack(spacing: AureliaSpacing.s) {
            panelChip(
                icon: "quote.bubble",
                title: "Lyrics",
                isActive: activePanel == .lyrics,
                action: toggleLyricsPanel
            )

            panelChip(
                icon: "list.bullet",
                title: "Queue",
                isActive: activePanel == .queue,
                isEnabled: !viewModel.queue.isEmpty,
                action: toggleQueuePanel
            )

            panelChip(
                icon: viewModel.isFavorite ? "heart.fill" : "heart",
                title: "Favorite",
                isActive: viewModel.isFavorite,
                tint: viewModel.isFavorite ? .red : .white,
                isEnabled: !viewModel.isFavoriteLoading,
                action: { viewModel.toggleFavorite() }
            )
        }
        .frame(maxWidth: .infinity)
    }

    private var lyricsCard: some View {
        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            if let lyrics = viewModel.lyrics {
                LyricsView(
                    lyrics: lyrics,
                    positionMs: viewModel.positionMs,
                    onSeekToMs: { targetMs in
                        viewModel.seekTo(targetMs, playerController: playerController)
                    }
                )
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            } else {
                VStack(spacing: AureliaSpacing.s) {
                    ProgressView()
                        .tint(.white)
                    Text("Loading lyrics…")
                        .font(.subheadline)
                        .foregroundStyle(.white.opacity(0.74))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .padding(.horizontal, AureliaSpacing.s)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var queueCard: some View {
        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            panelHeader(title: "Up Next", subtitle: "\(viewModel.queue.count) tracks")

            ScrollView(showsIndicators: false) {
                LazyVStack(spacing: 2) {
                    ForEach(Array(viewModel.queue.enumerated()), id: \.element.id) { index, song in
                        queueRow(song: song, index: index)
                    }
                }
            }
        }
        .padding(.horizontal, AureliaSpacing.s)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var topBar: some View {
        HStack(spacing: AureliaSpacing.s) {
            Button {
                dismiss()
            } label: {
                Image(systemName: "chevron.down")
                    .font(.headline.weight(.semibold))
                    .frame(width: 36, height: 36)
                    .foregroundStyle(.white)
                    .background(.ultraThinMaterial, in: Circle())
            }
            .buttonStyle(.plain)

            Spacer()

            Color.clear.frame(width: 36, height: 36)
        }
        .padding(.horizontal, AureliaSpacing.m)
        .padding(.top, AureliaSpacing.s)
        .padding(.bottom, AureliaSpacing.s)
        .background(
            LinearGradient(
                colors: [Color.black.opacity(0.24), Color.black.opacity(0)],
                startPoint: .top,
                endPoint: .bottom
            )
        )
    }

    private func iconControl(
        systemName: String,
        isEnabled: Bool = true,
        isActive: Bool = false,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 23, weight: .semibold))
                .foregroundStyle(isActive ? .white : .white.opacity(0.84))
                .opacity(isEnabled ? 1 : 0.4)
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }

    private func panelChip(
        icon: String,
        title: String,
        isActive: Bool,
        tint: Color = .white,
        isEnabled: Bool = true,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Label(title, systemImage: icon)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(tint.opacity(isEnabled ? 1 : 0.5))
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(isActive ? Color.white.opacity(0.18) : Color.white.opacity(0.08), in: Capsule())
                .overlay(
                    Capsule()
                        .stroke(Color.white.opacity(0.22), lineWidth: 1)
                )
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }

    private func panelHeader(title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.headline.weight(.semibold))
                .foregroundStyle(.white)

            Text(subtitle)
                .font(.caption)
                .foregroundStyle(.white.opacity(0.58))
        }
    }

    private func queueRow(song: Song, index: Int) -> some View {
        let isCurrent = index == viewModel.currentQueueIndex

        return Button {
            viewModel.playQueueItem(index, playerController: playerController)
        } label: {
            HStack(spacing: AureliaSpacing.s) {
                Text(isCurrent ? "•" : "\(index + 1)")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(isCurrent ? .white : .white.opacity(0.56))
                    .frame(width: 20, alignment: .leading)

                VStack(alignment: .leading, spacing: 2) {
                    Text(song.name)
                        .font(.subheadline.weight(isCurrent ? .semibold : .medium))
                        .lineLimit(1)
                        .foregroundStyle(isCurrent ? .white : .white.opacity(0.84))

                    Text(song.artists?.joined(separator: ", ") ?? "")
                        .font(.caption)
                        .lineLimit(1)
                        .foregroundStyle(.white.opacity(0.56))
                }

                Spacer()
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(isCurrent ? Color.white.opacity(0.14) : Color.clear)
            )
        }
        .buttonStyle(.plain)
    }

    private func toggleLyricsPanel() {
        if activePanel == .lyrics {
            if viewModel.showLyrics {
                viewModel.toggleLyrics()
            }
            activePanel = .none
            return
        }

        if viewModel.showLyrics == false {
            viewModel.toggleLyrics()
        }

        activePanel = .lyrics
    }

    private func toggleQueuePanel() {
        guard !viewModel.queue.isEmpty else { return }

        if activePanel == .queue {
            activePanel = .none
            return
        }

        if viewModel.showLyrics {
            viewModel.toggleLyrics()
        }

        activePanel = .queue
    }
}

private struct PlayerBackdropView: View {
    @Environment(\.colorScheme) private var colorScheme
    let albumArtUrl: String?

    var body: some View {
        ZStack {
            LinearGradient(
                colors: colorScheme == .dark
                    ? [Color(red: 0.10, green: 0.10, blue: 0.12), Color(red: 0.04, green: 0.04, blue: 0.05)]
                    : [Color(red: 0.93, green: 0.94, blue: 0.96), Color(red: 0.84, green: 0.86, blue: 0.90)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .ignoresSafeArea()

            if let albumArtUrl, let url = URL(string: albumArtUrl) {
                CachedImageView(
                    url: url,
                    contentMode: .fill,
                    placeholderColor: .clear,
                    targetSize: CGSize(width: 1200, height: 1200)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .clipped()
                .ignoresSafeArea()
                .blur(radius: 48)
                .saturation(colorScheme == .dark ? 0.95 : 0.72)
                .scaleEffect(1.14)
                .opacity(colorScheme == .dark ? 0.32 : 0.40)
            }

            LinearGradient(
                colors: [Color.black.opacity(0.14), Color.black.opacity(colorScheme == .dark ? 0.48 : 0.24)],
                startPoint: .top,
                endPoint: .bottom
            )
            .ignoresSafeArea()
        }
    }
}

struct LyricsView: View {
    let lyrics: Lyrics
    let positionMs: Int64
    var onSeekToMs: ((Int64) -> Void)? = nil
    @State private var lastCenteredIndex: Int?

    var body: some View {
        if let synced = lyrics.synced, !synced.isEmpty {
            syncedLyricsView(synced)
        } else if let plain = lyrics.plain, !plain.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            plainLyricsView(plain)
        } else {
            ContentUnavailableView("No Lyrics", systemImage: "quote.bubble", description: Text("No lyrics were found for this track."))
                .foregroundStyle(.white.opacity(0.78))
        }
    }

    private func syncedLyricsView(_ synced: [SyncedLine]) -> some View {
        ScrollViewReader { proxy in
            ScrollView(showsIndicators: false) {
                LazyVStack(spacing: 10) {
                    Color.clear.frame(height: 120)

                    ForEach(Array(synced.enumerated()), id: \.offset) { index, line in
                        Button {
                            let targetMs = Int64((line.time * 1000.0).rounded())
                            onSeekToMs?(targetMs)
                            lastCenteredIndex = index
                            withAnimation(.easeInOut(duration: 0.22)) {
                                proxy.scrollTo(index, anchor: .center)
                            }
                        } label: {
                            lyricLine(line.line, isActive: index == activeLineIndex(in: synced))
                                .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                        .id(index)
                    }

                    Color.clear.frame(height: 140)
                }
                .padding(.horizontal, AureliaSpacing.m)
            }
            .mask(
                LinearGradient(
                    colors: [.clear, .black, .black, .clear],
                    startPoint: .top,
                    endPoint: .bottom
                )
            )
            .onAppear {
                centerActiveLine(in: synced, proxy: proxy, animated: false)
            }
            .onChange(of: activeLineIndex(in: synced)) { _, _ in
                centerActiveLine(in: synced, proxy: proxy, animated: true)
            }
        }
    }

    private func plainLyricsView(_ plain: String) -> some View {
        ScrollView(showsIndicators: false) {
            Text(plain)
                .font(.body)
                .lineSpacing(6)
                .foregroundStyle(.white.opacity(0.88))
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, AureliaSpacing.m)
        }
    }

    private func lyricLine(_ text: String, isActive: Bool) -> some View {
        Text(text.isEmpty ? " " : text)
            .font(.system(size: 25, weight: .semibold, design: .rounded))
            .foregroundStyle(.white.opacity(isActive ? 0.98 : 0.50))
            .frame(maxWidth: .infinity, alignment: .leading)
            .multilineTextAlignment(.leading)
            .fixedSize(horizontal: false, vertical: true)
            .scaleEffect(isActive ? 1.015 : 1.0, anchor: .leading)
            .animation(.easeInOut(duration: 0.18), value: isActive)
    }

    private func centerActiveLine(in synced: [SyncedLine], proxy: ScrollViewProxy, animated: Bool) {
        guard let active = activeLineIndex(in: synced), active != lastCenteredIndex else { return }

        DispatchQueue.main.async {
            if animated {
                withAnimation(.easeInOut(duration: 0.3)) {
                    proxy.scrollTo(active, anchor: .center)
                }
            } else {
                proxy.scrollTo(active, anchor: .center)
            }

            lastCenteredIndex = active
        }
    }

    private func activeLineIndex(in synced: [SyncedLine]) -> Int? {
        guard !synced.isEmpty else { return nil }

        let currentTime = Double(positionMs) / 1000.0
        var low = 0
        var high = synced.count - 1
        var bestIndex: Int?

        while low <= high {
            let mid = (low + high) / 2
            if synced[mid].time <= currentTime {
                bestIndex = mid
                low = mid + 1
            } else {
                high = mid - 1
            }
        }

        return bestIndex
    }
}
