import AureliaCore
import SwiftUI

struct ArtistsView: View {
    @State private var artists: [(id: String, name: String, artUrl: String?, songCount: Int)] = []
    @State private var isLoading = false
    @State private var error: String?

    var body: some View {
        NavigationStack {
            Group {
                if isLoading && artists.isEmpty {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let error, artists.isEmpty {
                    ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                } else if artists.isEmpty {
                    ContentUnavailableView("No Artists", systemImage: "music.mic", description: Text("Your library has no artists"))
                } else {
                    List(artists, id: \.id) { artist in
                        NavigationLink(value: ArtistRoute(id: artist.id, name: artist.name)) {
                            HStack(spacing: 12) {
                                if let url = artist.artUrl {
                                    AlbumArtView(url: url, size: .small)
                                        .clipShape(Circle())
                                } else {
                                    Circle()
                                        .fill(.quaternary)
                                        .frame(width: 44, height: 44)
                                        .overlay {
                                            Image(systemName: "music.mic")
                                                .foregroundStyle(.secondary)
                                        }
                                }

                                VStack(alignment: .leading) {
                                    Text(artist.name)
                                    Text("\(artist.songCount) songs")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                            }
                        }
                    }
                    .scrollIndicators(.hidden)
                    .listStyle(.inset)
                }
            }
            .navigationTitle("Artists")
            .navigationDestination(for: ArtistRoute.self) { route in
                ArtistDetailView(artistId: route.id, artistName: route.name)
            }
            .onAppear { loadArtists() }
        }
        .aureliaScreen()
    }

    private func loadArtists() {
        guard artists.isEmpty else { return }
        isLoading = true

        Task {
            let appDataDir = SessionStore.shared.getAppDataDir() ?? ""
            do {
                let songs = try loadCachedSongs(appDataDir: appDataDir)
                var artistMap: [String: (name: String, artUrl: String?, count: Int)] = [:]
                for song in songs {
                    guard let artistId = song.artistIds?.first, !artistId.isEmpty else { continue }
                    let name = song.artists?.first ?? "Unknown Artist"
                    if var existing = artistMap[artistId] {
                        existing.count += 1
                        artistMap[artistId] = existing
                    } else {
                        artistMap[artistId] = (name: name, artUrl: nil, count: 1)
                    }
                }
                artists = artistMap
                    .map { (id: $0.key, name: $0.value.name, artUrl: $0.value.artUrl, songCount: $0.value.count) }
                    .sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
                isLoading = false
            } catch {
                self.error = error.localizedDescription
                isLoading = false
            }
        }
    }
}
