import SwiftUI
import AureliaCore

struct MainView: View {
    @State private var selection: MainDestination = .home
    @State private var showNowPlaying = false
    @EnvironmentObject private var playerController: AudioPlayerController

    var body: some View {
        ZStack {
            NavigationSplitView {
                List(selection: selectionBinding) {
                    ForEach(MainDestination.allCases) { destination in
                        NavigationLink(value: destination) {
                            Label(destination.title, systemImage: destination.systemImage)
                        }
                    }
                }
                .scrollIndicators(.hidden)
                .listStyle(.sidebar)
                .navigationTitle("Aurelia")
                .controlSize(.large)
                .environment(\.defaultMinListRowHeight, 34)
            } detail: {
                selection.destinationView()
                    .controlSize(.large)
                    .environment(\.defaultMinListRowHeight, 40)
                    .safeAreaInset(edge: .bottom) {
                        MiniPlayerView {
                            showNowPlaying = true
                        }
                        .controlSize(.regular)
                    }
            }
            .navigationSplitViewStyle(.balanced)
            .aureliaScreen()

            if showNowPlaying {
                NowPlayingImmersiveView(isPresented: $showNowPlaying)
                    .environmentObject(playerController)
                    .transition(.opacity)
                    .zIndex(2)
            }
        }
        .animation(.easeInOut(duration: 0.18), value: showNowPlaying)
    }

    private var selectionBinding: Binding<MainDestination?> {
        Binding(
            get: { selection },
            set: { newValue in
                if let newValue {
                    selection = newValue
                }
            }
        )
    }
}

private struct NowPlayingImmersiveView: View {
    private enum Panel {
        case none
        case lyrics
        case queue
    }

    @Binding var isPresented: Bool
    @EnvironmentObject private var playerController: AudioPlayerController

    @State private var isDragging = false
    @State private var dragPosition: Double = 0
    @State private var activePanel: Panel = .none
    @State private var lyrics: ImmersiveLyrics?
    @State private var lyricsSongId: String?
    @State private var isLoadingLyrics = false

    private var snapshot: PlayerSnapshot {
        playerController.snapshot
    }

    private var queue: [Song] {
        playerController.getQueue()
    }

    private var currentQueueIndex: Int {
        playerController.getCurrentQueueIndex()
    }

    var body: some View {
        GeometryReader { geometry in
            let horizontalInset: CGFloat = geometry.size.width >= 1080 ? AureliaSpacing.xl : AureliaSpacing.m
            let contentWidth = max(geometry.size.width - (horizontalInset * 2), 320)
            let panelVisible = activePanel != .none
            let preferredPlayerWidth = min(CGFloat(520), contentWidth)
            let canShowSidePanel = panelVisible && contentWidth >= (preferredPlayerWidth + 300 + AureliaSpacing.xl)
            let panelWidth: CGFloat = canShowSidePanel
                ? min(CGFloat(360), max(CGFloat(290), contentWidth - preferredPlayerWidth - AureliaSpacing.xl))
                : 0
            let playerWidth = preferredPlayerWidth
            let baseArtSize = min(
                geometry.size.width >= 1080 ? 320 : 280,
                max(190, playerWidth - 140)
            )

            ZStack {
                PlayerBackdrop(albumArtUrl: snapshot.albumArtUrl)
                    .ignoresSafeArea()

                Group {
                    if canShowSidePanel {
                        HStack(alignment: .center, spacing: AureliaSpacing.xl) {
                            playerColumn(artSize: baseArtSize)
                                .frame(width: playerWidth)

                            panelView
                                .frame(width: panelWidth)
                                .frame(maxHeight: 560)
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
                        .padding(.horizontal, horizontalInset)
                        .padding(.top, 52)
                        .padding(.bottom, AureliaSpacing.l)
                    } else {
                        VStack(spacing: AureliaSpacing.l) {
                            playerColumn(artSize: baseArtSize)
                                .frame(maxWidth: playerWidth)

                            if panelVisible {
                                panelView
                                    .frame(maxWidth: min(contentWidth, 660))
                                    .frame(maxHeight: 300)
                            }
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
                        .padding(.horizontal, horizontalInset)
                        .padding(.top, 52)
                        .padding(.bottom, AureliaSpacing.l)
                    }
                }
            }
            .overlay(alignment: .top) {
                topBar
            }
            .onAppear {
                if activePanel == .lyrics {
                    loadLyricsIfNeeded(force: false)
                }
            }
            .onChange(of: snapshot.currentSongId) { _, _ in
                if activePanel == .lyrics {
                    loadLyricsIfNeeded(force: true)
                }
            }
        }
    }

    private var topBar: some View {
        HStack(spacing: AureliaSpacing.s) {
            Button {
                isPresented = false
            } label: {
                Image(systemName: "chevron.down")
                    .font(.headline.weight(.semibold))
                    .frame(width: 36, height: 36)
                    .foregroundStyle(.white)
            }
            .buttonStyle(.glass)
            .buttonBorderShape(.circle)

            Spacer()
        }
        .padding(.horizontal, AureliaSpacing.m)
        .padding(.top, AureliaSpacing.m)
        .padding(.bottom, AureliaSpacing.s)
        .background(
            LinearGradient(
                colors: [Color.black.opacity(0.30), Color.black.opacity(0)],
                startPoint: .top,
                endPoint: .bottom
            )
        )
    }

    private func panelToggle(icon: String, title: String, panel: Panel) -> some View {
        let isActive = activePanel == panel

        return Button {
            if isActive {
                activePanel = .none
            } else {
                activePanel = panel
                if panel == .lyrics {
                    loadLyricsIfNeeded(force: false)
                }
            }
        } label: {
            Label(title, systemImage: icon)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.white.opacity(isActive ? 1 : 0.84))
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(isActive ? Color.white.opacity(0.18) : Color.white.opacity(0.08), in: Capsule())
                .overlay(
                    Capsule()
                        .stroke(Color.white.opacity(0.22), lineWidth: 1)
                )
        }
        .buttonStyle(.plain)
    }

    private func playerColumn(artSize: CGFloat) -> some View {
        VStack(spacing: AureliaSpacing.l) {
            AlbumArtView(url: snapshot.albumArtUrl, size: .large, customDimension: artSize)
                .shadow(color: Color.black.opacity(0.30), radius: 18, x: 0, y: 10)

            VStack(spacing: AureliaSpacing.m) {
                VStack(spacing: 4) {
                    Text(snapshot.title.isEmpty ? "Nothing playing" : snapshot.title)
                        .font(.title2.weight(.bold))
                        .lineLimit(2)
                        .multilineTextAlignment(.center)

                    Text(snapshot.artist)
                        .font(.headline.weight(.medium))
                        .lineLimit(1)
                        .foregroundStyle(.white.opacity(0.84))
                }

                progressSection
                playbackControls
            }
            .padding(.horizontal, AureliaSpacing.l)
        }
        .frame(maxWidth: .infinity)
    }

    private var progressSection: some View {
        VStack(spacing: 8) {
            Slider(
                value: Binding(
                    get: { isDragging ? dragPosition : Double(snapshot.positionMs) },
                    set: { newValue in
                        dragPosition = newValue
                        isDragging = true
                    }
                ),
                in: 0...max(Double(snapshot.durationMs), 1),
                onEditingChanged: { editing in
                    if !editing {
                        playerController.seekTo(positionMs: Int64(dragPosition))
                        isDragging = false
                    }
                }
            )
            .tint(.white)

            HStack {
                Text(TimeFormatter.formatDuration(isDragging ? Int64(dragPosition) : snapshot.positionMs))
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.white.opacity(0.72))

                Spacer()

                Text(TimeFormatter.formatDuration(snapshot.durationMs))
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.white.opacity(0.72))
            }
        }
    }

    private var playbackControls: some View {
        VStack(spacing: AureliaSpacing.m) {
            HStack(spacing: 28) {
                iconControl(
                    systemName: "backward.fill",
                    isEnabled: snapshot.hasPrevious,
                    action: { playerController.skipPrevious() }
                )

                Button {
                    playerController.togglePlayPause()
                } label: {
                    Image(systemName: snapshot.isPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 34, weight: .bold))
                        .frame(width: 68, height: 68)
                        .foregroundStyle(.white)
                }
                .buttonStyle(.plain)

                iconControl(
                    systemName: "forward.fill",
                    isEnabled: snapshot.hasNext,
                    action: { playerController.skipNext() }
                )
            }

            HStack(spacing: AureliaSpacing.s) {
                panelToggle(
                    icon: "quote.bubble",
                    title: "Lyrics",
                    panel: .lyrics
                )

                if !queue.isEmpty {
                    panelToggle(
                        icon: "music.note.list",
                        title: "Queue",
                        panel: .queue
                    )
                }
            }
            .frame(maxWidth: .infinity)
        }
    }

    @ViewBuilder
    private var panelView: some View {
        switch activePanel {
        case .none:
            EmptyView()
        case .lyrics:
            lyricsPanel
        case .queue:
            queuePanel
        }
    }

    private var lyricsPanel: some View {
        Group {
            if isLoadingLyrics {
                VStack(spacing: AureliaSpacing.s) {
                    ProgressView().tint(.white)
                    Text("Loading lyrics…")
                        .font(.subheadline)
                        .foregroundStyle(.white.opacity(0.78))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let lyrics {
                ImmersiveLyricsView(lyrics: lyrics, positionMs: snapshot.positionMs) { targetMs in
                    playerController.seekTo(positionMs: targetMs)
                }
            } else {
                ContentUnavailableView(
                    "No Lyrics",
                    systemImage: "quote.bubble",
                    description: Text("No lyrics were found for this track.")
                )
                .foregroundStyle(.white.opacity(0.78))
            }
        }
    }

    private var queuePanel: some View {
        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Up Next")
                    .font(.headline.weight(.semibold))
                    .foregroundStyle(.white)

                Text("\(queue.count) tracks")
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.66))
            }

            ScrollView(showsIndicators: false) {
                LazyVStack(spacing: 2) {
                    ForEach(Array(queue.enumerated()), id: \.element.id) { index, song in
                        queueRow(song: song, index: index)
                    }
                }
            }
            .scrollIndicators(.hidden)
        }
        .padding(.horizontal, AureliaSpacing.s)
        .padding(.vertical, AureliaSpacing.xs)
    }

    private func queueRow(song: Song, index: Int) -> some View {
        let isCurrent = index == currentQueueIndex

        return Button {
            playerController.playQueueItem(index)
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
                        .foregroundStyle(isCurrent ? .white : .white.opacity(0.88))

                    Text(song.artists?.joined(separator: ", ") ?? "")
                        .font(.caption)
                        .lineLimit(1)
                        .foregroundStyle(.white.opacity(0.58))
                }

                Spacer()
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .fill(isCurrent ? Color.white.opacity(0.12) : Color.clear)
            )
        }
        .buttonStyle(.plain)
    }

    private func iconControl(
        systemName: String,
        isEnabled: Bool = true,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 23, weight: .semibold))
                .foregroundStyle(.white.opacity(isEnabled ? 0.88 : 0.42))
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }

    private func loadLyricsIfNeeded(force: Bool) {
        guard let songId = snapshot.currentSongId,
              !songId.isEmpty else {
            lyrics = nil
            lyricsSongId = nil
            isLoadingLyrics = false
            return
        }

        if !force, lyricsSongId == songId {
            return
        }

        guard let serverUrl = SessionStore.shared.serverUrl,
              let token = SessionStore.shared.token else {
            lyrics = nil
            lyricsSongId = songId
            isLoadingLyrics = false
            return
        }

        isLoadingLyrics = true
        lyricsSongId = songId

        let artist = snapshot.artist
        let title = snapshot.title

        Task {
            let parsed = await getParsedLyrics(
                serverUrl: serverUrl,
                token: token,
                itemId: songId,
                artist: artist,
                title: title
            )

            let mapped = ImmersiveLyrics.fromParsed(parsed)

            if lyricsSongId == songId {
                lyrics = mapped.isValid ? mapped : nil
                isLoadingLyrics = false
            }
        }
    }
}

private struct PlayerBackdrop: View {
    let albumArtUrl: String?

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                Color.black

                CachedImageView(
                    url: albumArtUrl.flatMap { URL(string: $0) },
                    contentMode: .fill,
                    placeholderColor: Color(nsColor: .windowBackgroundColor),
                    targetSize: CGSize(
                        width: (geometry.size.width + 220) * 1.2,
                        height: (geometry.size.height + 220) * 1.2
                    )
                )
                .frame(width: geometry.size.width + 220, height: geometry.size.height + 220)
                .blur(radius: 48)
                .saturation(1.16)
            }
            .frame(width: geometry.size.width, height: geometry.size.height)
            .clipped()
            .overlay(Color.black.opacity(0.50))
        }
    }
}

private struct ImmersiveLyricsWord: Equatable {
    var time: TimeInterval
    var word: String
}

private struct ImmersiveLyricsLine: Equatable {
    var time: TimeInterval
    var line: String
    var words: [ImmersiveLyricsWord]?
}

private struct ImmersiveLyrics: Equatable {
    var plain: String?
    var synced: [ImmersiveLyricsLine]?

    var isValid: Bool {
        (plain != nil && !(plain ?? "").isEmpty) || (synced != nil && !(synced ?? []).isEmpty)
    }

    static func fromParsed(_ parsed: ParsedLyrics) -> ImmersiveLyrics {
        let synced = parsed.synced.isEmpty
            ? nil
            : parsed.synced.map {
                ImmersiveLyricsLine(
                    time: Double($0.timeMs) / 1000.0,
                    line: $0.line,
                    words: $0.words?.map {
                        ImmersiveLyricsWord(
                            time: Double($0.timeMs) / 1000.0,
                            word: $0.word
                        )
                    }
                )
            }

        let plain = parsed.plain.isEmpty ? nil : parsed.plain.joined(separator: "\n")

        return ImmersiveLyrics(plain: plain, synced: synced)
    }
}

private struct ImmersiveLyricsView: View {
    let lyrics: ImmersiveLyrics
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

    private func syncedLyricsView(_ synced: [ImmersiveLyricsLine]) -> some View {
        ScrollViewReader { proxy in
            ScrollView(showsIndicators: false) {
                LazyVStack(spacing: 10) {
                    Color.clear.frame(height: 110)

                    ForEach(Array(synced.enumerated()), id: \.offset) { index, line in
                        Button {
                            let targetMs = Int64((line.time * 1000.0).rounded())
                            onSeekToMs?(targetMs)
                            lastCenteredIndex = index
                            withAnimation(.easeInOut(duration: 0.22)) {
                                proxy.scrollTo(index, anchor: .center)
                            }
                        } label: {
                            Text(line.line.isEmpty ? " " : line.line)
                                .font(.system(size: 23, weight: .semibold, design: .rounded))
                                .foregroundStyle(.white.opacity(index == activeLineIndex(in: synced) ? 0.98 : 0.54))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .multilineTextAlignment(.leading)
                                .fixedSize(horizontal: false, vertical: true)
                        }
                        .buttonStyle(.plain)
                        .id(index)
                    }

                    Color.clear.frame(height: 130)
                }
                .padding(.horizontal, AureliaSpacing.s)
            }
            .scrollIndicators(.hidden)
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
                .padding(.horizontal, AureliaSpacing.s)
        }
        .scrollIndicators(.hidden)
    }

    private func activeLineIndex(in synced: [ImmersiveLyricsLine]) -> Int? {
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

    private func centerActiveLine(in synced: [ImmersiveLyricsLine], proxy: ScrollViewProxy, animated: Bool) {
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
}
