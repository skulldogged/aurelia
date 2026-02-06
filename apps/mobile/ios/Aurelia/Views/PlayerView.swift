import SwiftUI

struct PlayerView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel = PlayerViewModel()
    @State private var isDragging = false
    @State private var dragPosition: Double = 0

    var body: some View {
        GeometryReader { geometry in
            let isWide = AureliaLayout.isWide(geometry.size.width)
            let maxArt: CGFloat = isWide ? 320 : 340
            let artInset: CGFloat = isWide ? 160 : 64
            let artSize = max(CGFloat(200), min(maxArt, geometry.size.width - artInset))

            ZStack {
                PlayerBackdropView(albumArtUrl: viewModel.albumArtUrl)

                ScrollView {
                    if isWide {
                        HStack(alignment: .top, spacing: AureliaSpacing.xl) {
                            primaryColumn(artSize: artSize)
                                .frame(maxWidth: 360)

                            VStack(alignment: .leading, spacing: AureliaSpacing.l) {
                                if viewModel.showLyrics, let _ = viewModel.lyrics {
                                    lyricsCard
                                }

                                if !viewModel.queue.isEmpty {
                                    queueCard
                                }
                            }
                            .frame(maxWidth: .infinity)
                        }
                        .padding(.horizontal, AureliaSpacing.xl)
                        .padding(.top, AureliaSpacing.l)
                        .padding(.bottom, AureliaSpacing.l)
                    } else {
                        VStack(spacing: AureliaSpacing.l) {
                            primaryColumn(artSize: artSize)

                            if viewModel.showLyrics, let _ = viewModel.lyrics {
                                lyricsCard
                            }

                            if !viewModel.queue.isEmpty {
                                queueCard
                            }
                        }
                        .padding(.horizontal, AureliaSpacing.m)
                        .padding(.top, AureliaSpacing.l)
                        .padding(.bottom, AureliaSpacing.l)
                    }
                }
            }
            .safeAreaInset(edge: .top) {
                topBar
            }
        }
        .onChange(of: playerController.snapshot) { _, snapshot in
            viewModel.updateFrom(snapshot, playerController: playerController)
        }
        .onAppear {
            viewModel.updateFrom(playerController.snapshot, playerController: playerController)
        }
    }

    private func primaryColumn(artSize: CGFloat) -> some View {
        VStack(spacing: AureliaSpacing.l) {
            AlbumArtView(url: viewModel.albumArtUrl, size: .extraLarge, customDimension: artSize)
                .shadow(radius: 18)

            GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.l) {
                VStack(spacing: AureliaSpacing.m) {
                    songInfo
                    progressSection
                    playbackControls
                    secondaryControls
                }
            }
        }
    }

    private var songInfo: some View {
        VStack(spacing: 4) {
            Text(viewModel.title)
                .font(.title2.bold())
                .lineLimit(1)

            Text(viewModel.artist)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .lineLimit(1)

            if let formatInfo = viewModel.formatInfo {
                Text(formatInfo)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.horizontal)
    }

    private var progressSection: some View {
        VStack(spacing: 4) {
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
            .tint(.primary)

            HStack {
                Text(TimeFormatter.formatDuration(isDragging ? Int64(dragPosition) : viewModel.positionMs))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
                Spacer()
                Text(TimeFormatter.formatDuration(viewModel.durationMs))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }
        }
    }

    private var playbackControls: some View {
        HStack(spacing: 32) {
            Button {
                viewModel.toggleShuffle(playerController: playerController)
            } label: {
                Image(systemName: "shuffle")
                    .font(.title3)
                    .foregroundStyle(viewModel.isShuffled ? .primary : .secondary)
            }

            Button {
                viewModel.skipPrevious(playerController: playerController)
            } label: {
                Image(systemName: "backward.fill")
                    .font(.title)
            }

            Button {
                viewModel.togglePlayPause(playerController: playerController)
            } label: {
                Image(systemName: viewModel.isPlaying ? "pause.circle.fill" : "play.circle.fill")
                    .font(.system(size: 56))
            }

            Button {
                viewModel.skipNext(playerController: playerController)
            } label: {
                Image(systemName: "forward.fill")
                    .font(.title)
            }

            Button {
                viewModel.cycleRepeatMode(playerController: playerController)
            } label: {
                Image(systemName: viewModel.repeatMode == .one ? "repeat.1" : "repeat")
                    .font(.title3)
                    .foregroundStyle(viewModel.repeatMode == .none ? .secondary : .primary)
            }
        }
        .buttonStyle(.plain)
    }

    private var secondaryControls: some View {
        HStack(spacing: 24) {
            Button {
                viewModel.toggleFavorite()
            } label: {
                Image(systemName: viewModel.isFavorite ? "heart.fill" : "heart")
                    .foregroundStyle(viewModel.isFavorite ? .red : .secondary)
            }
            .disabled(viewModel.isFavoriteLoading)

            Button {
                viewModel.toggleLyrics()
            } label: {
                Image(systemName: "text.quote")
                    .foregroundStyle(viewModel.showLyrics ? .primary : .secondary)
            }
        }
        .font(.title3)
        .buttonStyle(.plain)
    }

    private var lyricsCard: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m) {
            if let lyrics = viewModel.lyrics {
                LyricsView(lyrics: lyrics, positionMs: viewModel.positionMs)
                    .frame(maxHeight: 320)
            }
        }
    }

    private var queueCard: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Queue")
                    .font(.headline)

                LazyVStack(spacing: 0) {
                    ForEach(Array(viewModel.queue.enumerated()), id: \.element.id) { index, song in
                        SongRow(
                            song: song,
                            isPlaying: index == viewModel.currentQueueIndex
                        ) {
                            viewModel.playQueueItem(index, playerController: playerController)
                        }
                        if index != viewModel.queue.count - 1 {
                            Divider()
                        }
                    }
                }
            }
        }
    }

    private var topBar: some View {
        HStack(spacing: AureliaSpacing.s) {
            Button {
                dismiss()
            } label: {
                Image(systemName: "chevron.down")
                    .font(.headline.weight(.semibold))
                    .frame(width: 36, height: 36)
                    .background(.ultraThinMaterial, in: Circle())
            }
            .buttonStyle(.plain)

            Spacer()

            VStack(spacing: 0) {
                Text("Now Playing")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                if let albumName = viewModel.currentAlbumName {
                    Text(albumName)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                        .lineLimit(1)
                }
            }

            Spacer()

            Color.clear.frame(width: 36, height: 36)
        }
        .padding(.horizontal, AureliaSpacing.m)
        .padding(.top, AureliaSpacing.s)
        .padding(.bottom, AureliaSpacing.xs)
    }
}

private struct PlayerBackdropView: View {
    let albumArtUrl: String?

    var body: some View {
        ZStack {
            LinearGradient(
                colors: [
                    Color(.secondarySystemBackground),
                    Color(.systemBackground)
                ],
                startPoint: .top,
                endPoint: .bottom
            )
            if let albumArtUrl, let url = URL(string: albumArtUrl) {
                CachedImageView(
                    url: url,
                    contentMode: .fill,
                    placeholderColor: .clear,
                    targetSize: CGSize(width: 700, height: 700)
                )
                .blur(radius: 36)
                .opacity(0.28)
                .ignoresSafeArea()
            }
        }
        .ignoresSafeArea()
    }
}

struct LyricsView: View {
    let lyrics: Lyrics
    let positionMs: Int64

    var body: some View {
        ScrollView {
            if let synced = lyrics.synced {
                let activeIndex = activeLineIndex(in: synced)
                LazyVStack(spacing: 8) {
                    ForEach(Array(synced.enumerated()), id: \.offset) { index, line in
                        let isActive = index == activeIndex
                        Text(line.line)
                            .font(.body)
                            .fontWeight(isActive ? .bold : .regular)
                            .foregroundStyle(isActive ? .primary : .secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            } else if let plain = lyrics.plain {
                Text(plain)
                    .font(.body)
                    .foregroundStyle(.secondary)
            }
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
