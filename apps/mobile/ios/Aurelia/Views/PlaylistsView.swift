import SwiftUI
import AureliaCore

struct PlaylistsView: View {
    @State private var viewModel = PlaylistViewModel()
    @State private var showCreateDialog = false
    @State private var newPlaylistName = ""

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if viewModel.playlists.isEmpty {
                    ContentUnavailableView("No Playlists", systemImage: "music.note.list", description: Text("Create a playlist to get started"))
                } else {
                    List {
                        ForEach(viewModel.playlists, id: \.id) { playlist in
                            NavigationLink(value: PlaylistRoute(id: playlist.id, name: playlist.name)) {
                                HStack(spacing: 12) {
                                    Image(systemName: "music.note.list")
                                        .frame(width: 48, height: 48)
                                        .background(.quaternary)
                                        .clipShape(RoundedRectangle(cornerRadius: 8))

                                    VStack(alignment: .leading) {
                                        Text(playlist.name)
                                        if let count = playlist.childCount {
                                            Text("\(count) songs")
                                                .font(.caption)
                                                .foregroundStyle(.secondary)
                                        }
                                    }
                                }
                            }
                            .swipeActions(edge: .trailing) {
                                Button(role: .destructive) {
                                    viewModel.deletePlaylist(playlist.id)
                                } label: {
                                    Label("Delete", systemImage: "trash")
                                }
                            }
                        }
                    }
                    .listStyle(.plain)
                }
            }
            .navigationTitle("Playlists")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        showCreateDialog = true
                    } label: {
                        Image(systemName: "plus")
                    }
                }
            }
            .navigationDestination(for: PlaylistRoute.self) { route in
                PlaylistDetailView(playlistId: route.id, playlistName: route.name)
            }
            .alert("New Playlist", isPresented: $showCreateDialog) {
                TextField("Playlist name", text: $newPlaylistName)
                Button("Create") {
                    viewModel.createPlaylist(name: newPlaylistName)
                    newPlaylistName = ""
                }
                Button("Cancel", role: .cancel) {
                    newPlaylistName = ""
                }
            }
            .onAppear { viewModel.loadPlaylists() }
        }
    }
}
