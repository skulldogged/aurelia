import AureliaCore
import SwiftUI

struct AlbumsView: View {
    @State private var albums: [AlbumItem] = []
    @State private var isLoading = false
    @State private var error: String?

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
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 170), spacing: AureliaSpacing.m)], spacing: AureliaSpacing.m) {
                            ForEach(albums) { album in
                                NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                                    GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s) {
                                        VStack(alignment: .leading, spacing: 6) {
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
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.horizontal, AureliaSpacing.l)
                        .padding(.vertical, AureliaSpacing.l)
                    }
                    .scrollIndicators(.hidden)
                }
            }
            .navigationTitle("Albums")
            .navigationDestination(for: AlbumRoute.self) { route in
                AlbumDetailView(albumId: route.id, albumName: route.name)
            }
            .onAppear { loadAlbums() }
        }
        .aureliaScreen()
    }

    private func loadAlbums() {
        guard albums.isEmpty else { return }
        let sessionStore = SessionStore.shared

        isLoading = true

        Task {
            let appDataDir = sessionStore.getAppDataDir() ?? ""
            do {
                let songs = try loadCachedSongs(appDataDir: appDataDir)
                let albumsMap = Dictionary(grouping: songs.filter { $0.albumId != nil && !$0.albumId!.isEmpty }) { $0.albumId! }
                let albumItems = albumsMap.map { id, albumSongs in
                    let first = albumSongs[0]
                    return AlbumItem(
                        id: id,
                        name: first.album ?? "Unknown Album",
                        artist: first.artists?.first ?? "Unknown Artist",
                        albumArtUrl: first.albumArtUrl,
                        songCount: albumSongs.count
                    )
                }
                .sorted { (lhs: AlbumItem, rhs: AlbumItem) in
                    lhs.name.localizedCaseInsensitiveCompare(rhs.name) == .orderedAscending
                }

                albums = albumItems
                isLoading = false
            } catch {
                self.error = error.localizedDescription
                isLoading = false
            }
        }
    }
}
