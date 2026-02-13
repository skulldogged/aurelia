import AureliaCore
import SwiftUI

struct SearchView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var searchText = ""
    @State private var allSongs: [Song] = []
    @State private var isLoaded = false

    private var isWide: Bool {
        horizontalSizeClass == .regular
    }

    private var filteredSongs: [Song] {
        guard !searchText.isEmpty else { return [] }
        let query = searchText.lowercased()
        return allSongs.filter { song in
            song.name.lowercased().contains(query)
                || (song.artists?.contains { $0.lowercased().contains(query) } ?? false)
                || (song.album?.lowercased().contains(query) ?? false)
        }
        .prefix(50)
        .map(\.self)
    }

    private var filteredAlbums: [AlbumItem] {
        guard !searchText.isEmpty else { return [] }
        let query = searchText.lowercased()
        let songsByAlbumId = allSongs.compactMap { song -> (String, Song)? in
            guard let albumId = song.albumId, !albumId.isEmpty else { return nil }
            return (albumId, song)
        }
        let albumsMap = Dictionary(grouping: songsByAlbumId, by: { $0.0 })
        return albumsMap.compactMap { id, songs -> AlbumItem? in
            let first = songs[0].1
            let name = first.album ?? ""
            let artist = first.artists?.first ?? ""
            guard name.lowercased().contains(query) || artist.lowercased().contains(query) else { return nil }
            return AlbumItem(id: id, name: name, artist: artist, albumArtUrl: first.albumArtUrl, songCount: songs.count)
        }
        .prefix(20)
        .map(\.self)
    }

    var body: some View {
        NavigationStack {
            Group {
                if searchText.isEmpty {
                    ContentUnavailableView("Search Your Library", systemImage: "magnifyingglass", description: Text("Search for songs, albums, or artists"))
                } else if filteredSongs.isEmpty, filteredAlbums.isEmpty {
                    ContentUnavailableView.search(text: searchText)
                } else {
                    if isWide {
                        ScrollView {
                            HStack(alignment: .top, spacing: AureliaSpacing.l) {
                                albumResultsColumn
                                    .frame(maxWidth: .infinity)

                                songResultsColumn
                                    .frame(maxWidth: .infinity)
                            }
                            .padding(.horizontal, AureliaSpacing.xl)
                            .padding(.vertical, AureliaSpacing.l)
                        }
                    } else {
                        List {
                            if !filteredAlbums.isEmpty {
                                Section("Albums") {
                                    ForEach(filteredAlbums) { album in
                                        NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                                            GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s, showsShadow: false) {
                                                HStack(spacing: 12) {
                                                    AlbumArtView(url: album.albumArtUrl, size: .small)
                                                    VStack(alignment: .leading, spacing: 2) {
                                                        Text(album.name).lineLimit(1)
                                                        Text(album.artist)
                                                            .font(.caption)
                                                            .foregroundStyle(.secondary)
                                                            .lineLimit(1)
                                                    }
                                                    Spacer()
                                                }
                                            }
                                        }
                                        .listRowSeparator(.hidden)
                                        .listRowBackground(Color.clear)
                                        .listRowInsets(EdgeInsets(top: 6, leading: AureliaSpacing.m, bottom: 6, trailing: AureliaSpacing.m))
                                    }
                                }
                            }

                            if !filteredSongs.isEmpty {
                                Section("Songs") {
                                    ForEach(filteredSongs, id: \.id) { song in
                                        GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s, showsShadow: false) {
                                            SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                                                let queue = filteredSongs
                                                if let idx = queue.firstIndex(where: { $0.id == song.id }),
                                                   let serverUrl = SessionStore.shared.serverUrl,
                                                   let token = SessionStore.shared.token
                                                {
                                                    playerController.setQueue(queue, serverUrl: serverUrl, token: token, startIndex: idx)
                                                }
                                            }
                                        }
                                        .listRowSeparator(.hidden)
                                        .listRowBackground(Color.clear)
                                        .listRowInsets(EdgeInsets(top: 6, leading: AureliaSpacing.m, bottom: 6, trailing: AureliaSpacing.m))
                                    }
                                }
                            }
                        }
                        .listStyle(.plain)
                        .scrollContentBackground(.hidden)
                    }
                }
            }
            .aureliaRootTabHeader("Search")
            .searchable(text: $searchText, prompt: "Songs, albums, artists")
            .navigationDestination(for: AlbumRoute.self) { route in
                AlbumDetailView(albumId: route.id, albumName: route.name)
            }
            .onAppear { loadSongs() }
        }
        .aureliaScreen()
    }

    private var albumResultsColumn: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m, showsShadow: false) {
            VStack(alignment: .leading, spacing: AureliaSpacing.s) {
                Text("Albums")
                    .font(.headline)

                if filteredAlbums.isEmpty {
                    Text("No album matches")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else {
                    LazyVStack(spacing: 10) {
                        ForEach(filteredAlbums) { album in
                            NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                                HStack(spacing: 12) {
                                    AlbumArtView(url: album.albumArtUrl, size: .small)
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(album.name)
                                            .lineLimit(1)
                                        Text(album.artist)
                                            .font(.caption)
                                            .foregroundStyle(.secondary)
                                            .lineLimit(1)
                                    }
                                    Spacer()
                                }
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
        }
    }

    private var songResultsColumn: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m, showsShadow: false) {
            VStack(alignment: .leading, spacing: AureliaSpacing.s) {
                Text("Songs")
                    .font(.headline)

                if filteredSongs.isEmpty {
                    Text("No song matches")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else {
                    LazyVStack(spacing: 0) {
                        ForEach(filteredSongs, id: \.id) { song in
                            SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                                let queue = filteredSongs
                                if let idx = queue.firstIndex(where: { $0.id == song.id }),
                                   let serverUrl = SessionStore.shared.serverUrl,
                                   let token = SessionStore.shared.token
                                {
                                    playerController.setQueue(queue, serverUrl: serverUrl, token: token, startIndex: idx)
                                }
                            }
                            if song.id != filteredSongs.last?.id {
                                Divider()
                            }
                        }
                    }
                }
            }
        }
    }

    private func loadSongs() {
        guard !isLoaded else { return }
        Task.detached {
            let appDataDir = await SessionStore.shared.getAppDataDir() ?? ""
            let songs = (try? loadCachedSongs(appDataDir: appDataDir)) ?? []
            await MainActor.run {
                allSongs = songs
                isLoaded = true
            }
        }
    }
}
