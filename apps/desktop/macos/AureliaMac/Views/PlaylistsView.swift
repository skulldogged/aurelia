import AureliaCore
import SwiftUI

struct PlaylistsView: View {
    @StateObject private var viewModel = PlaylistViewModel()
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
                                        .frame(width: 40, height: 40)
                                        .glassEffect()
                                        .clipShape(RoundedRectangle(cornerRadius: AureliaRadius.s, style: .continuous))

                                    VStack(alignment: .leading) {
                                        Text(playlist.name)
                                        if let count = playlist.childCount {
                                            Text("\(count) songs")
                                                .font(.caption)
                                                .foregroundStyle(.secondary)
                                        }
                                    }
                                    Spacer()
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
                    .listStyle(.inset)
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
                    .buttonStyle(.glass)
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
        .aureliaScreen()
    }
}
