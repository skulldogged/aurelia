import SwiftUI

struct PlayerView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel = PlayerViewModel()
    @State private var isDragging = false
    @State private var dragPosition: Double = 0

    var body: some View {
        NavigationStack {
            GeometryReader { geometry in
                ScrollView {
                    VStack(spacing: 24) {
                        // Album Art
                        AlbumArtView(url: viewModel.albumArtUrl, size: .extraLarge)
                            .frame(width: {
                                let size = max(0, min(geometry.size.width - 64, 340))
                                return size
                            }(), height: {
                                let size = max(0, min(geometry.size.width - 64, 340))
                                return size
                            }())
                            .shadow(radius: 20)

                        // Song Info
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

                        // Progress Slider
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
                        .padding(.horizontal)

                        // Playback Controls
                        HStack(spacing: 32) {
                            // Shuffle
                            Button {
                                viewModel.toggleShuffle(playerController: playerController)
                            } label: {
                                Image(systemName: "shuffle")
                                    .font(.title3)
                                    .foregroundStyle(viewModel.isShuffled ? .primary : .secondary)
                            }

                            // Previous
                            Button {
                                viewModel.skipPrevious(playerController: playerController)
                            } label: {
                                Image(systemName: "backward.fill")
                                    .font(.title)
                            }

                            // Play/Pause
                            Button {
                                viewModel.togglePlayPause(playerController: playerController)
                            } label: {
                                Image(systemName: viewModel.isPlaying ? "pause.circle.fill" : "play.circle.fill")
                                    .font(.system(size: 56))
                            }

                            // Next
                            Button {
                                viewModel.skipNext(playerController: playerController)
                            } label: {
                                Image(systemName: "forward.fill")
                                    .font(.title)
                            }

                            // Repeat
                            Button {
                                viewModel.cycleRepeatMode(playerController: playerController)
                            } label: {
                                Image(systemName: viewModel.repeatMode == .one ? "repeat.1" : "repeat")
                                    .font(.title3)
                                    .foregroundStyle(viewModel.repeatMode == .none ? .secondary : .primary)
                            }
                        }
                        .buttonStyle(.plain)

                        // Secondary Controls
                        HStack(spacing: 24) {
                            // Favorite
                            Button {
                                viewModel.toggleFavorite()
                            } label: {
                                Image(systemName: viewModel.isFavorite ? "heart.fill" : "heart")
                                    .foregroundStyle(viewModel.isFavorite ? .red : .secondary)
                            }
                            .disabled(viewModel.isFavoriteLoading)

                            // Lyrics toggle
                            Button {
                                viewModel.toggleLyrics()
                            } label: {
                                Image(systemName: "text.quote")
                                    .foregroundStyle(viewModel.showLyrics ? .primary : .secondary)
                            }
                        }
                        .font(.title3)
                        .buttonStyle(.plain)

                        // Lyrics
                        if viewModel.showLyrics, let lyrics = viewModel.lyrics {
                            LyricsView(lyrics: lyrics, positionMs: viewModel.positionMs)
                                .frame(maxHeight: 300)
                                .padding(.horizontal)
                        }

                        // Queue
                        if !viewModel.queue.isEmpty {
                            VStack(alignment: .leading, spacing: 8) {
                                Text("Queue")
                                    .font(.headline)
                                    .padding(.horizontal)

                                LazyVStack(spacing: 0) {
                                    ForEach(Array(viewModel.queue.enumerated()), id: \.element.id) { index, song in
                                        SongRow(
                                            song: song,
                                            isPlaying: index == viewModel.currentQueueIndex
                                        ) {
                                            viewModel.playQueueItem(index, playerController: playerController)
                                        }
                                        Divider()
                                    }
                                }
                                .padding(.horizontal)
                            }
                        }
                    }
                    .padding(.vertical, 32)
                }
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "chevron.down")
                    }
                }

                ToolbarItem(placement: .principal) {
                    VStack(spacing: 0) {
                        Text("Now Playing")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        if let albumName = viewModel.currentAlbumName {
                            Text(albumName)
                                .font(.caption2)
                                .foregroundStyle(.tertiary)
                        }
                    }
                }
            }
        }
        .onChange(of: playerController.snapshot) { _, snapshot in
            viewModel.updateFrom(snapshot, playerController: playerController)
        }
        .onAppear {
            viewModel.updateFrom(playerController.snapshot, playerController: playerController)
        }
    }
}

// MARK: - Lyrics View

struct LyricsView: View {
    let lyrics: Lyrics
    let positionMs: Int64

    var body: some View {
        ScrollView {
            if let synced = lyrics.synced {
                LazyVStack(spacing: 8) {
                    ForEach(Array(synced.enumerated()), id: \.offset) { index, line in
                        let isActive = isLineActive(index: index, synced: synced)
                        Text(line.line)
                            .font(.body)
                            .fontWeight(isActive ? .bold : .regular)
                            .foregroundStyle(isActive ? .primary : .secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .animation(.easeInOut(duration: 0.3), value: isActive)
                    }
                }
            } else if let plain = lyrics.plain {
                Text(plain)
                    .font(.body)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func isLineActive(index: Int, synced: [SyncedLine]) -> Bool {
        let currentTime = Double(positionMs) / 1000.0
        let lineTime = synced[index].time
        let nextTime = index + 1 < synced.count ? synced[index + 1].time : Double.infinity
        return currentTime >= lineTime && currentTime < nextTime
    }
}

