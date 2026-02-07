import AureliaCore
import SwiftUI

struct PlaylistDetailView: View {
    let playlistId: String
    let playlistName: String

    @EnvironmentObject private var playerController: AudioPlayerController
    @StateObject private var viewModel = PlaylistViewModel()

    var body: some View {
        Group {
            if viewModel.detailIsLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let error = viewModel.detailError {
                ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
            } else if viewModel.detailSongs.isEmpty {
                ContentUnavailableView("Empty Playlist", systemImage: "music.note.list", description: Text("This playlist has no songs"))
            } else {
                ScrollView {
                    HStack(alignment: .top, spacing: AureliaSpacing.xl) {
                        playlistSummary
                            .frame(maxWidth: 360)

                        playlistSongs
                            .frame(maxWidth: .infinity, alignment: .top)
                    }
                    .padding(.horizontal, AureliaSpacing.xl)
                    .padding(.vertical, AureliaSpacing.l)
                }
                .scrollIndicators(.hidden)
            }
        }
        .navigationTitle(playlistName)
        .onAppear { viewModel.loadPlaylistDetail(playlistId: playlistId) }
        .aureliaScreen()
    }

    private var playlistSummary: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.l) {
            VStack(spacing: AureliaSpacing.m) {
                Image(systemName: "music.note.list")
                    .font(.system(size: 64))
                    .frame(width: 160, height: 160)
                    .glassEffect()
                    .clipShape(RoundedRectangle(cornerRadius: AureliaRadius.l, style: .continuous))

                Text(playlistName)
                    .font(.title2.bold())
                Text("\(viewModel.detailSongs.count) songs")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                HStack(spacing: 16) {
                    Button {
                        viewModel.playPlaylist(playerController: playerController)
                    } label: {
                        Label("Play", systemImage: "play.fill")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.glass)

                    Button {
                        viewModel.shufflePlaylist(playerController: playerController)
                    } label: {
                        Label("Shuffle", systemImage: "shuffle")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.glass)
                }
            }
        }
    }

    private var playlistSongs: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m) {
            LazyVStack(spacing: 0) {
                ForEach(Array(viewModel.detailSongs.enumerated()), id: \.element.id) { index, song in
                    SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                        viewModel.playPlaylist(startIndex: index, playerController: playerController)
                    }
                    if index != viewModel.detailSongs.count - 1 {
                        Divider()
                    }
                }
            }
        }
    }
}
