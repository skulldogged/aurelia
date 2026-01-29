import SwiftUI
import AureliaCore

struct SearchView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @State private var searchText = ""
    @State private var allSongs: [Song] = []
    @State private var isLoaded = false

    private var filteredSongs: [Song] {
        guard !searchText.isEmpty else { return [] }
        let query = searchText.lowercased()
        return allSongs.filter { song in
            song.name.lowercased().contains(query)
                || (song.artists?.contains { $0.lowercased().contains(query) } ?? false)
                || (song.album?.lowercased().contains(query) ?? false)
        }
        .prefix(50)
        .map { $0 }
    }

    private var filteredAlbums: [AlbumItem] {
        guard !searchText.isEmpty else { return [] }
        let query = searchText.lowercased()
        let albumsMap = Dictionary(grouping: allSongs.filter { $0.albumId != nil && !$0.albumId!.isEmpty }) { $0.albumId! }
        return albumsMap.compactMap { (id, songs) -> AlbumItem? in
            let first = songs[0]
            let name = first.album ?? ""
            let artist = first.artists?.first ?? ""
            guard name.lowercased().contains(query) || artist.lowercased().contains(query) else { return nil }
            return AlbumItem(id: id, name: name, artist: artist, albumArtUrl: first.albumArtUrl, songCount: songs.count)
        }
        .prefix(20)
        .map { $0 }
    }

    var body: some View {
        NavigationStack {
            Group {
                if searchText.isEmpty {
                    ContentUnavailableView("Search Your Library", systemImage: "magnifyingglass", description: Text("Search for songs, albums, or artists"))
                } else if filteredSongs.isEmpty && filteredAlbums.isEmpty {
                    ContentUnavailableView.search(text: searchText)
                } else {
                    List {
                        if !filteredAlbums.isEmpty {
                            Section("Albums") {
                                ForEach(filteredAlbums) { album in
                                    NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                                        HStack(spacing: 12) {
                                            AlbumArtView(url: album.albumArtUrl, size: .small)
                                            VStack(alignment: .leading) {
                                                Text(album.name).lineLimit(1)
                                                Text(album.artist)
                                                    .font(.caption)
                                                    .foregroundStyle(.secondary)
                                                    .lineLimit(1)
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !filteredSongs.isEmpty {
                            Section("Songs") {
                                ForEach(filteredSongs, id: \.id) { song in
                                    SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                                        let queue = filteredSongs
                                        if let idx = queue.firstIndex(where: { $0.id == song.id }),
                                           let serverUrl = SessionStore.shared.serverUrl,
                                           let token = SessionStore.shared.token {
                                            playerController.setQueue(queue, serverUrl: serverUrl, token: token, startIndex: idx)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    .listStyle(.plain)
                }
            }
            .navigationTitle("Search")
            .searchable(text: $searchText, prompt: "Songs, albums, artists")
            .navigationDestination(for: AlbumRoute.self) { route in
                AlbumDetailView(albumId: route.id, albumName: route.name)
            }
            .onAppear { loadSongs() }
        }
    }

    private func loadSongs() {
        guard !isLoaded else { return }
        Task.detached {
            let appDataDir = await SessionStore.shared.getAppDataDir() ?? ""
            let songs = (try? loadCachedSongs(appDataDir: appDataDir)) ?? []
            await MainActor.run {
                self.allSongs = songs
                self.isLoaded = true
            }
        }
    }
}
