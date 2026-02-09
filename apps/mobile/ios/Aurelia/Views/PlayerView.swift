import SwiftUI
import AureliaCore
import UIKit

struct PlayerView: View {
    private enum Panel {
        case none
        case lyrics
        case queue
    }

    @Environment(\.colorScheme) private var colorScheme
    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.dismiss) private var dismiss
    var onClose: (() -> Void)? = nil

    @State private var viewModel = PlayerViewModel()
    @State private var isDragging = false
    @State private var dragPosition: Double = 0
    @State private var activePanel: Panel = .none

    var body: some View {
        GeometryReader { geometry in
            let isWide = UIDevice.current.userInterfaceIdiom == .pad || AureliaLayout.isWide(geometry.size.width)
            let horizontalInset: CGFloat = isWide ? AureliaSpacing.xxl : AureliaSpacing.m
            let topSafeInset = max(geometry.safeAreaInsets.top, statusBarTopInset)
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
            let compactTopBarReservedHeight = topSafeInset + 36 + (AureliaSpacing.s * 2)
            let contentTopPadding: CGFloat = isWide ? 58 : compactTopBarReservedHeight
            let bottomContentPadding = max(geometry.safeAreaInsets.bottom + AureliaSpacing.s, AureliaSpacing.l)
            let centeredContentHeight = max(0, geometry.size.height - contentTopPadding - bottomContentPadding)

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
                        .padding(.bottom, bottomContentPadding)
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
                        .padding(.bottom, bottomContentPadding)
                    } else {
                        compactPlayerLayout(
                            playerWidth: playerWidth,
                            horizontalInset: horizontalInset,
                            contentTopPadding: contentTopPadding,
                            bottomPadding: bottomContentPadding,
                            centeredContentHeight: centeredContentHeight
                        )
                    }
                }
            }
            .frame(width: geometry.size.width, height: geometry.size.height, alignment: .center)
            .background {
                PlayerBackdropView(albumArtUrl: viewModel.albumArtUrl)
            }
            .overlay(alignment: .top) {
                topBar(topSafeInset: topSafeInset)
            }
        }
        .onChange(of: playerController.snapshot) { _, snapshot in
            viewModel.updateFrom(snapshot, position: playerController.playbackPosition, playerController: playerController)
        }
        .onChange(of: playerController.playbackPosition) { _, position in
            viewModel.updateFrom(playerController.snapshot, position: position, playerController: playerController)
        }
        .onChange(of: viewModel.showLyrics) { _, isShown in
            if !isShown, activePanel == .lyrics {
                activePanel = .none
            }
        }
        .onAppear {
            viewModel.updateFrom(playerController.snapshot, position: playerController.playbackPosition, playerController: playerController)
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

    private func compactPlayerLayout(
        playerWidth: CGFloat,
        horizontalInset: CGFloat,
        contentTopPadding: CGFloat,
        bottomPadding: CGFloat,
        centeredContentHeight: CGFloat
    ) -> some View {
        let artworkWidth = playerWidth

        return ViewThatFits(in: .vertical) {
            VStack(spacing: AureliaSpacing.l) {
                compactArtworkPanelSlot(width: artworkWidth)
                controlsColumn(horizontalPadding: AureliaSpacing.m)
                    .frame(maxWidth: playerWidth)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
            .frame(height: centeredContentHeight, alignment: .bottom)
            .padding(.horizontal, horizontalInset)
            .padding(.top, contentTopPadding)
            .padding(.bottom, bottomPadding)

            ScrollView(showsIndicators: false) {
                VStack(spacing: AureliaSpacing.l) {
                    compactArtworkPanelSlot(width: playerWidth)

                    controlsColumn(horizontalPadding: AureliaSpacing.m)
                        .frame(maxWidth: playerWidth)
                }
                .frame(maxWidth: .infinity)
                .padding(.horizontal, horizontalInset)
                .padding(.top, contentTopPadding)
                .padding(.bottom, max(bottomPadding, AureliaSpacing.xl))
            }
        }
    }

    private func compactArtworkPanelSlot(width: CGFloat) -> some View {
        Group {
            if activePanel == .none {
                artworkHero(artSize: width)
                    .frame(width: width, height: width, alignment: .center)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
            } else {
                panelView
                    .frame(width: width, alignment: .topLeading)
                    .frame(maxHeight: .infinity, alignment: .topLeading)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
            }
        }
        .frame(
            maxWidth: .infinity,
            maxHeight: .infinity,
            alignment: activePanel == .none ? .bottom : .top
        )
    }

    private func primaryColumn(artSize: CGFloat) -> some View {
        VStack(spacing: AureliaSpacing.l) {
            artworkHero(artSize: artSize)
            controlsColumn(horizontalPadding: AureliaSpacing.l)
        }
    }

    private func artworkHero(artSize: CGFloat) -> some View {
        AlbumArtView(url: viewModel.albumArtUrl, size: .extraLarge, customDimension: artSize)
            .shadow(color: Color.black.opacity(colorScheme == .dark ? 0.36 : 0.16), radius: 18, x: 0, y: 10)
            .overlay(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(Color.white.opacity(colorScheme == .dark ? 0.18 : 0.30), lineWidth: 1)
            )
    }

    private func controlsColumn(horizontalPadding: CGFloat) -> some View {
        VStack(spacing: AureliaSpacing.m) {
            songInfo
            progressSection
            playbackControls
            secondaryControls
        }
        .padding(.horizontal, horizontalPadding)
        .padding(.vertical, AureliaSpacing.l)
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
                    .font(.system(size: 34, weight: .bold))
                    .frame(width: 68, height: 68)
                    .foregroundStyle(.white)
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
                    isPlaying: viewModel.isPlaying,
                    updateTimeMs: viewModel.positionUpdateTimeMs,
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

    private func topBar(topSafeInset: CGFloat) -> some View {
        HStack(spacing: AureliaSpacing.s) {
            Button {
                if let onClose {
                    onClose()
                } else {
                    dismiss()
                }
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
        .padding(.top, topSafeInset + AureliaSpacing.s)
        .padding(.bottom, AureliaSpacing.s)
        .background(
            LinearGradient(
                colors: [Color.black.opacity(0.24), Color.black.opacity(0)],
                startPoint: .top,
                endPoint: .bottom
            )
        )
    }

    private var statusBarTopInset: CGFloat {
        UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .flatMap { $0.windows }
            .map { $0.safeAreaInsets.top }
            .max() ?? 0
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
                .blur(radius: 48)
                .saturation(colorScheme == .dark ? 0.95 : 0.72)
                .scaleEffect(1.14)
                .opacity(colorScheme == .dark ? 0.32 : 0.40)
                .mask(Rectangle())
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
    let isPlaying: Bool
    let updateTimeMs: Int64
    var onSeekToMs: ((Int64) -> Void)? = nil
    @State private var lastCenteredIndex: Int?

    /// Whether these lyrics contain any word-level sync data.
    private var hasWordSync: Bool {
        lyrics.synced?.contains(where: { $0.words != nil && !$0.words!.isEmpty }) ?? false
    }

    var body: some View {
        if let synced = lyrics.synced, !synced.isEmpty {
            if hasWordSync {
                // Use TimelineView for high-frequency updates (~60fps) to drive
                // smooth word-level karaoke fill, similar to Apple Music.
                TimelineView(.animation(minimumInterval: nil, paused: !isPlaying)) { timeline in
                    let interpolatedMs = interpolatedPositionMs(at: timeline.date)
                    syncedLyricsView(synced, currentPositionMs: interpolatedMs)
                }
            } else {
                syncedLyricsView(synced, currentPositionMs: positionMs)
            }
        } else if let plain = lyrics.plain, !plain.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            plainLyricsView(plain)
        } else {
            ContentUnavailableView("No Lyrics", systemImage: "quote.bubble", description: Text("No lyrics were found for this track."))
                .foregroundStyle(.white.opacity(0.78))
        }
    }

    // MARK: - Time Interpolation

    /// Interpolate the current playback position from the last known snapshot
    /// using system uptime, giving us sub-frame accuracy for word fill.
    private func interpolatedPositionMs(at date: Date) -> Int64 {
        guard isPlaying, updateTimeMs > 0 else { return positionMs }
        let nowMs = Int64(ProcessInfo.processInfo.systemUptime * 1000)
        let elapsed = nowMs - updateTimeMs
        return positionMs + max(0, elapsed)
    }

    // MARK: - Section Labels

    /// Build a lookup of line start-time to section name for displaying dividers.
    private var sectionLabels: [TimeInterval: String] {
        guard let sections = lyrics.sections else { return [:] }
        var labels: [TimeInterval: String] = [:]
        for section in sections {
            if !section.name.isEmpty, let firstLine = section.lines.first {
                labels[firstLine.time] = section.name
            }
        }
        return labels
    }

    // MARK: - Line State

    /// Describes a line's visual state for rendering.
    /// - `active`: the primary line being sung — words fill with karaoke animation.
    /// - `finishing`: a line that was preempted by an overlapping vocal but whose
    ///   end time hasn't been reached yet. Words that were already sung stay bright
    ///   and the fill continues, but scale/glow animate down so focus shifts to the
    ///   new active line.
    /// - `inactive`: fully past — uniform dim opacity.
    private enum LineState: Equatable {
        case active
        case finishing
        case inactive
    }

    // MARK: - Active Line / Word Detection

    private func activeLineIndex(in synced: [SyncedLine], currentPositionMs: Int64) -> Int? {
        guard !synced.isEmpty else { return nil }

        let currentTime = Double(currentPositionMs) / 1000.0
        let tolerance = 0.01
        var low = 0
        var high = synced.count - 1
        var bestIndex: Int?

        while low <= high {
            let mid = (low + high) / 2
            if synced[mid].time <= currentTime + tolerance {
                bestIndex = mid
                low = mid + 1
            } else {
                high = mid - 1
            }
        }

        if let idx = bestIndex, let endTime = synced[idx].endTime {
            if currentTime > endTime {
                if idx + 1 < synced.count && synced[idx + 1].time <= currentTime + tolerance {
                    // Next line is active, binary search already found it
                } else {
                    // In a gap — keep current as active for continuity
                }
            }
        }

        return bestIndex
    }

    /// Determine the visual state of a line given its index and the active line.
    /// A line is "finishing" when it was preempted by an overlapping line but
    /// its end time hasn't been reached yet — its word highlights should keep
    /// animating rather than snapping to inactive.
    private func lineState(for index: Int, line: SyncedLine, activeIdx: Int?, currentPositionMs: Int64) -> LineState {
        guard let activeIdx else { return .inactive }
        if index == activeIdx { return .active }

        // Only lines *before* the active line can be "finishing".
        guard index < activeIdx else { return .inactive }

        let currentTime = Double(currentPositionMs) / 1000.0

        // Determine when this line's content actually ends.
        // Prefer the last word's end time for precision, fall back to line endTime.
        let lineContentEnd: TimeInterval? = {
            if let words = line.words, let lastWord = words.last {
                return lastWord.endTime ?? line.endTime
            }
            return line.endTime
        }()

        if let endTime = lineContentEnd, currentTime <= endTime {
            // Line hasn't finished its content yet — keep it animating
            return .finishing
        }

        return .inactive
    }

    /// For a line, determine which word is currently being sung.
    private func activeWordIndex(in line: SyncedLine, currentPositionMs: Int64) -> Int? {
        guard let words = line.words, !words.isEmpty else { return nil }
        let currentTime = Double(currentPositionMs) / 1000.0
        let tolerance = 0.01

        var result: Int?
        for i in stride(from: words.count - 1, through: 0, by: -1) {
            if words[i].time <= currentTime + tolerance {
                result = i
                break
            }
        }
        return result
    }

    /// Compute the progress (0...1) through a specific word for the gradient fill.
    private func wordProgress(word: SyncedWord, nextWord: SyncedWord?, lineEndTime: TimeInterval?, currentPositionMs: Int64) -> Double {
        let currentTime = Double(currentPositionMs) / 1000.0
        let endTime = word.endTime
            ?? nextWord?.time
            ?? lineEndTime
            ?? (word.time + 0.5)
        let duration = endTime - word.time
        guard duration > 0 else { return 1 }
        let elapsed = currentTime - word.time
        return min(1, max(0, elapsed / duration))
    }

    // MARK: - Synced Lyrics View

    private func syncedLyricsView(_ synced: [SyncedLine], currentPositionMs: Int64) -> some View {
        let labels = sectionLabels
        let activeIdx = activeLineIndex(in: synced, currentPositionMs: currentPositionMs)
        return ScrollViewReader { proxy in
            ScrollView(showsIndicators: false) {
                LazyVStack(spacing: 10) {
                    Color.clear.frame(height: 120)

                    ForEach(Array(synced.enumerated()), id: \.offset) { index, line in
                        if let label = labels[line.time] {
                            Text(label.uppercased())
                                .font(.caption2)
                                .fontWeight(.semibold)
                                .tracking(1.5)
                                .foregroundStyle(.white.opacity(0.30))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.top, index == 0 ? 0 : 8)
                        }

                        Button {
                            let targetMs = Int64((line.time * 1000.0).rounded())
                            onSeekToMs?(targetMs)
                            lastCenteredIndex = index
                            withAnimation(.easeInOut(duration: 0.22)) {
                                proxy.scrollTo(index, anchor: .center)
                            }
                        } label: {
                            let state = lineState(for: index, line: line, activeIdx: activeIdx, currentPositionMs: currentPositionMs)
                            let isBackground = lyrics.isBackgroundVocal(line.agentId)
                            let isWordSynced = line.words != nil && !line.words!.isEmpty

                            if isWordSynced {
                                wordSyncedLine(
                                    line: line,
                                    state: state,
                                    isBackground: isBackground,
                                    currentPositionMs: currentPositionMs
                                )
                                .contentShape(Rectangle())
                            } else {
                                lyricLine(
                                    line.line,
                                    isActive: state == .active,
                                    isBackground: isBackground
                                )
                                .contentShape(Rectangle())
                            }
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
                centerActiveLine(in: synced, proxy: proxy, currentPositionMs: currentPositionMs, animated: false)
            }
            .onChange(of: activeLineIndex(in: synced, currentPositionMs: currentPositionMs)) { _, _ in
                centerActiveLine(in: synced, proxy: proxy, currentPositionMs: currentPositionMs, animated: true)
            }
        }
    }

    // MARK: - Word-Synced Line (Apple Music Style)

    /// Renders a line with per-word karaoke fill.
    ///
    /// Each word is an individual `Text` inside a `WordFlowLayout` so that:
    /// 1. Each word's gradient is scoped to its own bounds (not the whole line).
    /// 2. Words wrap naturally across visual lines.
    /// 3. Repeated words ("la la la") maintain stable identity via their index.
    ///
    /// Three visual states driven by `LineState`:
    /// - **active**: full karaoke — words fill progressively, glow, scale up.
    /// - **finishing**: line was preempted by an overlap but its words keep
    ///   animating. Sung words stay bright, the current word's fill continues,
    ///   but scale and glow ease down so focus shifts to the new active line.
    /// - **inactive**: fully past — uniform dim opacity, no glow.
    private func wordSyncedLine(line: SyncedLine, state: LineState, isBackground: Bool, currentPositionMs: Int64) -> some View {
        let words = line.words!
        let isAnimating = state == .active || state == .finishing
        let activeWord = isAnimating ? activeWordIndex(in: line, currentPositionMs: currentPositionMs) : nil

        let brightOpacity: Double = isBackground ? 0.75 : 0.98
        let inactiveOpacity: Double = isBackground ? 0.25 : 0.50

        // Word colors are the same for active and finishing states — the
        // finishing fade is handled by an overall opacity on the container,
        // not by changing individual word colors. This avoids a jarring
        // color jump when the state changes.
        let brightColor: Color = .white.opacity(brightOpacity)
        let dimColor: Color = .white.opacity(inactiveOpacity)
        let inactiveColor: Color = .white.opacity(inactiveOpacity)

        // Overall opacity: finishing lines fade to ~75% so focus shifts
        // to the new active line, but the transition is smooth.
        let containerOpacity: Double = switch state {
        case .active: 1.0
        case .finishing: isBackground ? 0.65 : 0.75
        case .inactive: 1.0  // inactive word colors already handle dimming
        }

        // Glow: active lines glow proportionally to progress; finishing lines don't glow.
        let glowRadius: CGFloat = if let aw = activeWord, state == .active {
            CGFloat(6.0 * Double(aw + 1) / Double(words.count))
        } else {
            0
        }

        let isScaledUp = state == .active

        // Precompute which words have a leading space (inter-word gap from TTML/LRC).
        // We strip it from the display text and let the layout handle spacing,
        // so wrapped lines don't start with visible indentation.
        let wordEntries: [(text: String, hasGap: Bool)] = words.map { w in
            let hasGap = w.word.hasPrefix(" ")
            let trimmed = hasGap ? String(w.word.drop(while: { $0 == " " })) : w.word
            return (text: trimmed, hasGap: hasGap)
        }

        return WordFlowLayout {
            ForEach(Array(words.enumerated()), id: \.offset) { wIdx, word in
                let entry = wordEntries[wIdx]
                // Determine the solid color for this word.
                // For finishing lines, the "active" word is treated as sung
                // (bright) so there's no gradient outlier — the container
                // opacity handles the overall fade instead.
                let isSung: Bool = if let aw = activeWord {
                    wIdx < aw || (wIdx == aw && state == .finishing)
                } else {
                    false
                }
                let wordColor: Color = if !isAnimating {
                    inactiveColor
                } else if isSung {
                    brightColor
                } else if activeWord != wIdx {
                    dimColor
                } else {
                    dimColor // placeholder; active word gradient below
                }

                // Only the active line gets the sweep gradient on the
                // current word. Finishing lines use solid colors for all
                // words so the container opacity fade looks uniform.
                if activeWord == wIdx, state == .active {
                    let nextWord = wIdx + 1 < words.count ? words[wIdx + 1] : nil
                    let progress = wordProgress(word: word, nextWord: nextWord, lineEndTime: line.endTime, currentPositionMs: currentPositionMs)
                    Text(entry.text)
                        .foregroundStyle(
                            .linearGradient(
                                stops: [
                                    .init(color: brightColor, location: max(0, progress - 0.01)),
                                    .init(color: dimColor, location: min(1, progress + 0.01)),
                                ],
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .layoutValue(key: WordGapKey.self, value: entry.hasGap)
                } else {
                    Text(entry.text)
                        .foregroundStyle(
                            .linearGradient(
                                stops: [.init(color: wordColor, location: 0)],
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .layoutValue(key: WordGapKey.self, value: entry.hasGap)
                }
            }
        }
        .font(.system(size: 25, weight: .semibold, design: .rounded))
        .italic(isBackground)
        .opacity(containerOpacity)
        .shadow(color: .white.opacity(state == .active ? 0.45 : 0), radius: glowRadius, x: 0, y: 0)
        .frame(maxWidth: .infinity, alignment: .leading)
        .fixedSize(horizontal: false, vertical: true)
        .scaleEffect(isScaledUp ? 1.015 : 1.0, anchor: .leading)
        .animation(.easeInOut(duration: 0.35), value: state)
    }

    // MARK: - Plain Line (line-synced / no word data)

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

    private func lyricLine(_ text: String, isActive: Bool, isBackground: Bool = false) -> some View {
        let opacity: Double = if isActive {
            isBackground ? 0.75 : 0.98
        } else {
            isBackground ? 0.25 : 0.50
        }

        return Text(text.isEmpty ? " " : text)
            .font(.system(size: 25, weight: .semibold, design: .rounded))
            .italic(isBackground)
            .foregroundStyle(.white.opacity(opacity))
            .frame(maxWidth: .infinity, alignment: .leading)
            .multilineTextAlignment(.leading)
            .fixedSize(horizontal: false, vertical: true)
            .scaleEffect(isActive ? 1.015 : 1.0, anchor: .leading)
            .animation(.easeInOut(duration: 0.35), value: isActive)
    }

    // MARK: - Scroll Centering

    private func centerActiveLine(in synced: [SyncedLine], proxy: ScrollViewProxy, currentPositionMs: Int64, animated: Bool) {
        guard let active = activeLineIndex(in: synced, currentPositionMs: currentPositionMs), active != lastCenteredIndex else { return }

        DispatchQueue.main.async {
            if animated {
                withAnimation(.easeInOut(duration: 0.35)) {
                    proxy.scrollTo(active, anchor: .center)
                }
            } else {
                proxy.scrollTo(active, anchor: .center)
            }

            lastCenteredIndex = active
        }
    }
}

// MARK: - Word Flow Layout

/// Layout value key that marks whether a word had a leading space (inter-word gap).
/// Used by `WordFlowLayout` to add spacing only between words that originally had
/// a space separator — and to suppress that space at the start of a wrapped row
/// so there's no visible indentation.
private struct WordGapKey: LayoutValueKey {
    static let defaultValue: Bool = false
}

/// A custom `Layout` that arranges children inline like text, wrapping to the
/// next line when children exceed the available width. Used for word-synced
/// lyrics so each word is its own view (with its own gradient bounds) while
/// still wrapping naturally.
///
/// Uses `WordGapKey` to determine inter-word spacing: words that had a leading
/// space in the source data get a space-width gap before them, except when they
/// land at the start of a wrapped row.
private struct WordFlowLayout: Layout {
    /// Approximate width of a space in the lyrics font.
    /// Measured for .system(size: 25, weight: .semibold, design: .rounded).
    private static let spaceWidth: CGFloat = 7.5

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = computeRows(proposal: proposal, subviews: subviews)
        guard let lastRow = rows.last else { return .zero }
        let height = lastRow.yOffset + lastRow.height
        let width = rows.map(\.width).max() ?? 0
        return CGSize(width: min(width, proposal.width ?? .infinity), height: height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let rows = computeRows(proposal: proposal, subviews: subviews)
        var subviewIndex = 0
        for row in rows {
            var x = bounds.minX
            for posInRow in 0..<row.count {
                let subview = subviews[subviewIndex]
                let size = subview.sizeThatFits(.unspecified)
                // Add inter-word gap if this word had a leading space — but not
                // at the start of a row (to avoid indentation on wrapped lines).
                if posInRow > 0 && subview[WordGapKey.self] {
                    x += Self.spaceWidth
                }
                subview.place(at: CGPoint(x: x, y: bounds.minY + row.yOffset), proposal: .unspecified)
                x += size.width
                subviewIndex += 1
            }
        }
    }

    private struct Row {
        var count: Int
        var width: CGFloat
        var height: CGFloat
        var yOffset: CGFloat
    }

    private func computeRows(proposal: ProposedViewSize, subviews: Subviews) -> [Row] {
        let maxWidth = proposal.width ?? .infinity
        var rows: [Row] = []
        var currentRow = Row(count: 0, width: 0, height: 0, yOffset: 0)

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            let gap: CGFloat = (currentRow.count > 0 && subview[WordGapKey.self]) ? Self.spaceWidth : 0
            let neededWidth = size.width + gap

            if currentRow.count > 0 && currentRow.width + neededWidth > maxWidth {
                // Wrap to next row — no gap at the start of a new row.
                rows.append(currentRow)
                let nextY = currentRow.yOffset + currentRow.height
                currentRow = Row(count: 1, width: size.width, height: size.height, yOffset: nextY)
            } else {
                currentRow.width += neededWidth
                currentRow.height = max(currentRow.height, size.height)
                currentRow.count += 1
            }
        }

        if currentRow.count > 0 {
            rows.append(currentRow)
        }

        return rows
    }
}
