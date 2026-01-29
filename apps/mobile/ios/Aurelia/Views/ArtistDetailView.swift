import SwiftUI
import AureliaCore

struct ArtistDetailView: View {
    let artistId: String
    let artistName: String

    @Environment(AudioPlayerController.self) private var playerController
    @State private var artist: Artist?
    @State private var songs: [Song] = []
    @State private var albums: [AlbumItem] = []
    @State private var isLoading = true
    @State private var error: String?

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let error {
                ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
            } else {
                ScrollView {
                    VStack(spacing: 16) {
                        // Artist header
                        AsyncImage(url: artist?.imageUrl.flatMap { URL(string: $0) }) { image in
                            image.resizable().scaledToFill()
                        } placeholder: {
                            Image(systemName: "music.mic")
                                .font(.system(size: 48))
                                .foregroundStyle(.secondary)
                        }
                        .frame(width: 160, height: 160)
                        .clipShape(Circle())

                        Text(artist?.name ?? artistName)
                            .font(.title2.bold())

                        if let overview = artist?.overview, !overview.isEmpty {
                            Text(overview)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .multilineTextAlignment(.center)
                                .padding(.horizontal)
                        }

                        // Play / Shuffle
                        HStack(spacing: 16) {
                            Button {
                                playAllSongs(startIndex: 0)
                            } label: {
                                Label("Play All", systemImage: "play.fill")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.borderedProminent)

                            Button {
                                playAllSongs(shuffled: true)
                            } label: {
                                Label("Shuffle", systemImage: "shuffle")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.bordered)
                        }
                        .padding(.horizontal)

                        // Albums section
                        if !albums.isEmpty {
                            VStack(alignment: .leading, spacing: 8) {
                                Text("Albums")
                                    .font(.title3.bold())
                                    .padding(.horizontal)

                                ScrollView(.horizontal, showsIndicators: false) {
                                    LazyHStack(spacing: 12) {
                                        ForEach(albums) { album in
                                            NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                                                VStack(alignment: .leading, spacing: 4) {
                                                    AlbumArtView(url: album.albumArtUrl, size: .medium)
                                                    Text(album.name)
                                                        .font(.caption)
                                                        .lineLimit(1)
                                                }
                                                .frame(width: 140)
                                            }
                                            .buttonStyle(.plain)
                                        }
                                    }
                                    .padding(.horizontal)
                                }
                            }
                        }

                        // Songs
                        if !songs.isEmpty {
                            VStack(alignment: .leading, spacing: 8) {
                                Text("Songs")
                                    .font(.title3.bold())
                                    .padding(.horizontal)

                                LazyVStack(spacing: 0) {
                                    ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                                        SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId) {
                                            playAllSongs(startIndex: index)
                                        }
                                        Divider()
                                    }
                                }
                                .padding(.horizontal)
                            }
                        }
                    }
                    .padding(.vertical)
                }
            }
        }
        .navigationTitle(artistName)
        .navigationBarTitleDisplayMode(.inline)
        .navigationDestination(for: AlbumRoute.self) { route in
            AlbumDetailView(albumId: route.id, albumName: route.name)
        }
        .onAppear { loadArtist() }
    }

    private func loadArtist() {
        let sessionStore = SessionStore.shared
        guard let creds = sessionStore.getCredentials() else {
            error = "Missing session data"
            isLoading = false
            return
        }

        Task.detached {
            let appDataDir = await sessionStore.getAppDataDir() ?? ""

            // Load songs for this artist from cache
            if let allSongs = try? loadCachedSongs(appDataDir: appDataDir) {
                let artistSongs = allSongs.filter { $0.artistIds?.contains(self.artistId) ?? false }
                let albumsMap = Dictionary(grouping: artistSongs.filter { $0.albumId != nil }) { $0.albumId! }
                let albumItems = albumsMap.map { (id, songs) -> AlbumItem in
                    let first = songs[0]
                    return AlbumItem(id: id, name: first.album ?? "Unknown", artist: first.artists?.first ?? "", albumArtUrl: first.albumArtUrl, songCount: songs.count)
                }

                await MainActor.run {
                    self.songs = artistSongs
                    self.albums = albumItems
                }
            }

            // Fetch artist details
            do {
                let fetched = try fetchArtist(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    artistId: self.artistId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.artist = fetched
                    self.isLoading = false
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        self.isLoading = false
                        if self.songs.isEmpty { self.error = error.localizedDescription }
                    }
                }
            }
        }
    }

    private func playAllSongs(startIndex: Int = 0, shuffled: Bool = false) {
        guard let serverUrl = SessionStore.shared.serverUrl,
              let token = SessionStore.shared.token,
              !songs.isEmpty else { return }
        let queue = shuffled ? songs.shuffled() : Array(songs)
        playerController.setQueue(queue, serverUrl: serverUrl, token: token, startIndex: shuffled ? 0 : startIndex)
    }
}
