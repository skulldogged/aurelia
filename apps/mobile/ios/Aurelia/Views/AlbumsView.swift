import SwiftUI
import AureliaCore

struct AlbumsView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @State private var albums: [AlbumItem] = []
    @State private var isLoading = false
    @State private var error: String?

    private let columns = [
        GridItem(.adaptive(minimum: 150), spacing: 12)
    ]

    var body: some View {
        NavigationStack {
            Group {
                if isLoading && albums.isEmpty {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let error, albums.isEmpty {
                    ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                } else if albums.isEmpty {
                    ContentUnavailableView("No Albums", systemImage: "square.stack", description: Text("Your library has no albums"))
                } else {
                    ScrollView {
                        LazyVGrid(columns: columns, spacing: 16) {
                            ForEach(albums) { album in
                                NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                                    VStack(alignment: .leading, spacing: 4) {
                                        AlbumArtView(url: album.albumArtUrl, size: .medium)
                                        Text(album.name)
                                            .font(.caption)
                                            .lineLimit(1)
                                        Text(album.artist)
                                            .font(.caption2)
                                            .foregroundStyle(.secondary)
                                            .lineLimit(1)
                                    }
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding()
                    }
                }
            }
            .navigationTitle("Albums")
            .navigationDestination(for: AlbumRoute.self) { route in
                AlbumDetailView(albumId: route.id, albumName: route.name)
            }
            .onAppear { loadAlbums() }
        }
    }

    private func loadAlbums() {
        guard albums.isEmpty else { return }
        let sessionStore = SessionStore.shared
        guard let creds = sessionStore.getCredentials(),
              !creds.serverUrl.isEmpty else { return }

        isLoading = true

        Task.detached {
            let appDataDir = await sessionStore.getAppDataDir() ?? ""
            do {
                let songs = try loadCachedSongs(appDataDir: appDataDir)
                let albumsMap = Dictionary(grouping: songs.filter { $0.albumId != nil && !$0.albumId!.isEmpty }) { $0.albumId! }
                let albumItems = albumsMap.map { (id, albumSongs) -> AlbumItem in
                    let first = albumSongs[0]
                    return AlbumItem(
                        id: id,
                        name: first.album ?? "Unknown Album",
                        artist: first.artists?.first ?? "Unknown Artist",
                        albumArtUrl: first.albumArtUrl,
                        songCount: albumSongs.count
                    )
                }.sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }

                await MainActor.run {
                    self.albums = albumItems
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
}
