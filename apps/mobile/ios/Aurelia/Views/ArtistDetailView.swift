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
                        ZStack {
                            CachedImageView(
                                url: artist?.imageUrl.flatMap { URL(string: $0) },
                                contentMode: .fill,
                                targetSize: CGSize(width: 160, height: 160)
                            )
                            if artist?.imageUrl == nil {
                                Image(systemName: "music.mic")
                                    .font(.system(size: 48))
                                    .foregroundStyle(.secondary)
                            }
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

                        if songs.isEmpty && albums.isEmpty {
                            Text("No songs available for this artist yet.")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                                .padding(.top, 8)
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

            if let cachedArtist = try? getCachedArtist(appDataDir: appDataDir, artistId: self.artistId) {
                await MainActor.run {
                    self.artist = cachedArtist
                    self.isLoading = false
                }
            }

            // Load songs for this artist from cache first
            if let allSongs = try? loadCachedSongs(appDataDir: appDataDir) {
                let artistSongs = Self.songsForArtist(
                    from: allSongs,
                    artistId: self.artistId,
                    artistName: self.artistName
                )
                let albumItems = Self.albumItems(from: artistSongs, fallbackArtistName: self.artistName)

                await MainActor.run {
                    self.songs = artistSongs
                    self.albums = albumItems
                    self.isLoading = false
                }
            }

            // Fetch freshest artist metadata in background
            do {
                let fetched = try await fetchArtist(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    artistId: self.artistId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.artist = fetched
                    self.isLoading = false
                    self.error = nil
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        self.isLoading = false
                        if self.songs.isEmpty && self.artist == nil {
                            self.error = error.localizedDescription
                        }
                    }
                }
            }
        }
    }

    private nonisolated static func songsForArtist(from allSongs: [Song], artistId: String, artistName: String) -> [Song] {
        let byId = allSongs.filter { song in
            guard let ids = song.artistIds else { return false }
            return ids.contains(where: { splitValues($0).contains(artistId) })
        }
        if !byId.isEmpty {
            return sortSongs(byId)
        }

        let normalizedArtistName = artistName.trimmingCharacters(in: .whitespacesAndNewlines)
        if normalizedArtistName.isEmpty {
            return []
        }

        let byName = allSongs.filter { song in
            (song.artists ?? []).contains(where: { rawValue in
                splitValues(rawValue).contains(where: {
                    $0.localizedCaseInsensitiveCompare(normalizedArtistName) == .orderedSame
                })
            })
        }
        return sortSongs(byName)
    }

    private nonisolated static func albumItems(from songs: [Song], fallbackArtistName: String) -> [AlbumItem] {
        let grouped = Dictionary(grouping: songs.filter { song in
            if let albumId = song.albumId {
                return !albumId.isEmpty
            }
            return false
        }) { $0.albumId! }

        return grouped.map { albumId, albumSongs in
            let first = albumSongs[0]
            return AlbumItem(
                id: albumId,
                name: first.album ?? "Unknown Album",
                artist: first.artists?.first ?? fallbackArtistName,
                albumArtUrl: first.albumArtUrl,
                songCount: albumSongs.count
            )
        }
        .sorted { lhs, rhs in
            lhs.name.localizedCaseInsensitiveCompare(rhs.name) == .orderedAscending
        }
    }

    private nonisolated static func splitValues(_ rawValue: String) -> [String] {
        rawValue
            .split(whereSeparator: { $0 == "\u{001F}" || $0 == "|" || $0 == ";" })
            .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
    }

    private nonisolated static func sortSongs(_ songs: [Song]) -> [Song] {
        songs.sorted { lhs, rhs in
            let lhsAlbum = lhs.album ?? ""
            let rhsAlbum = rhs.album ?? ""
            if lhsAlbum != rhsAlbum {
                return lhsAlbum.localizedCaseInsensitiveCompare(rhsAlbum) == .orderedAscending
            }

            let lhsDisc = Int(lhs.discNumber ?? 1)
            let rhsDisc = Int(rhs.discNumber ?? 1)
            if lhsDisc != rhsDisc {
                return lhsDisc < rhsDisc
            }

            let lhsTrack = Int(lhs.trackNumber ?? Int32.max)
            let rhsTrack = Int(rhs.trackNumber ?? Int32.max)
            if lhsTrack != rhsTrack {
                return lhsTrack < rhsTrack
            }

            return lhs.name.localizedCaseInsensitiveCompare(rhs.name) == .orderedAscending
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
