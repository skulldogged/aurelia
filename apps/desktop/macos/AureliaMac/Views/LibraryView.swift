import AureliaCore
import SwiftUI

struct LibraryView: View {
    @EnvironmentObject private var playerController: AudioPlayerController
    @StateObject private var viewModel = LibraryViewModel()

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.songs.isEmpty {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let error = viewModel.error, viewModel.songs.isEmpty {
                    ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                } else if viewModel.songs.isEmpty {
                    ContentUnavailableView("No Songs", systemImage: "music.note", description: Text("Your library is empty"))
                } else {
                    List(viewModel.songs, id: \.id) { song in
                        SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                            viewModel.playFromList(song.id, playerController: playerController)
                        }
                    }
                    .scrollIndicators(.hidden)
                    .listStyle(.inset)
                }
            }
            .navigationTitle("Songs")
            .onAppear { viewModel.loadLibrary() }
        }
        .aureliaScreen()
    }
}
