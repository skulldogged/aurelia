import SwiftUI
import AureliaCore

struct LibraryView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @State private var viewModel = LibraryViewModel()

    var body: some View {
        NavigationStack {
            GeometryReader { proxy in
                let isWide = AureliaLayout.isWide(proxy.size.width)
                let columnCount = proxy.size.width > 1000 ? 3 : 2
                let columns = Array(repeating: GridItem(.flexible(), spacing: AureliaSpacing.m), count: columnCount)

                Group {
                    if viewModel.isLoading && viewModel.songs.isEmpty {
                        ProgressView()
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else if let error = viewModel.error, viewModel.songs.isEmpty {
                        ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                    } else if viewModel.songs.isEmpty {
                        ContentUnavailableView("No Songs", systemImage: "music.note", description: Text("Your library is empty"))
                    } else {
                        if isWide {
                            ScrollView {
                                LazyVGrid(columns: columns, spacing: AureliaSpacing.m) {
                                    ForEach(viewModel.songs, id: \.id) { song in
                                        GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s, showsShadow: false) {
                                            SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                                                viewModel.playFromList(song.id, playerController: playerController)
                                            }
                                        }
                                    }
                                }
                                .padding(.horizontal, AureliaSpacing.xl)
                                .padding(.vertical, AureliaSpacing.l)
                            }
                        } else {
                            List(viewModel.songs, id: \.id) { song in
                                GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s, showsShadow: false) {
                                    SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                                        viewModel.playFromList(song.id, playerController: playerController)
                                    }
                                }
                                .listRowSeparator(.hidden)
                                .listRowBackground(Color.clear)
                                .listRowInsets(EdgeInsets(top: 6, leading: AureliaSpacing.m, bottom: 6, trailing: AureliaSpacing.m))
                            }
                            .listStyle(.plain)
                            .scrollContentBackground(.hidden)
                        }
                    }
                }
            }
            .navigationTitle("Songs")
            .onAppear { viewModel.loadLibrary() }
        }
        .aureliaScreen()
    }
}
