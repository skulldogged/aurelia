import SwiftUI
import AureliaCore

struct HomeView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @State private var viewModel = HomeViewModel()

    var body: some View {
        NavigationStack {
            ScrollView {
                if viewModel.isLoading && viewModel.featuredAlbums.isEmpty {
                    ProgressView()
                        .frame(maxWidth: .infinity, minHeight: 200)
                } else if let error = viewModel.error, viewModel.featuredAlbums.isEmpty {
                    ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                } else {
                    LazyVStack(alignment: .leading, spacing: 24) {
                        // Featured Albums Carousel
                        if !viewModel.featuredAlbums.isEmpty {
                            featuredSection
                        }

                        // Most Played
                        if !viewModel.mostPlayed.isEmpty {
                            songSection(title: "Most Played", songs: viewModel.mostPlayed)
                        }

                        // Recently Played
                        if !viewModel.recentlyPlayed.isEmpty {
                            songSection(title: "Recently Played", songs: viewModel.recentlyPlayed)
                        }

                        // Recently Added Albums
                        if !viewModel.recentlyAddedAlbums.isEmpty {
                            albumSection(title: "Recently Added", albums: viewModel.recentlyAddedAlbums)
                        }

                        // From Your Library
                        if !viewModel.randomAlbums.isEmpty {
                            albumSection(title: "From Your Library", albums: viewModel.randomAlbums)
                        }
                    }
                    .padding(.vertical)
                }
            }
            .navigationTitle("Home")
            .onAppear { viewModel.loadHomeData() }
        }
    }

    // MARK: - Featured Albums

    private var featuredSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            TabView(selection: $viewModel.currentFeaturedIndex) {
                ForEach(Array(viewModel.featuredAlbums.enumerated()), id: \.element.id) { index, album in
                    NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                        AlbumArtView(url: album.albumArtUrl, size: .large)
                            .overlay(alignment: .bottomLeading) {
                                VStack(alignment: .leading) {
                                    Text(album.name)
                                        .font(.headline)
                                    Text(album.artist)
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }
                                .padding()
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(.ultraThinMaterial)
                            }
                            .clipShape(RoundedRectangle(cornerRadius: 16))
                    }
                    .tag(index)
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .automatic))
            .frame(height: 240)
            .padding(.horizontal)
        }
    }

    // MARK: - Song Section (horizontal scroll)

    private func songSection(title: String, songs: [Song]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.title3.bold())
                .padding(.horizontal)

            ScrollView(.horizontal, showsIndicators: false) {
                LazyHStack(spacing: 12) {
                    ForEach(songs, id: \.id) { song in
                        Button {
                            viewModel.playSongFromList(song.id, songList: songs, playerController: playerController)
                        } label: {
                            VStack(alignment: .leading, spacing: 4) {
                                AlbumArtView(url: song.albumArtUrl, size: .medium)
                                Text(song.name)
                                    .font(.caption)
                                    .lineLimit(1)
                                Text(song.artists?.first ?? "")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
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

    // MARK: - Album Section (horizontal scroll)

    private func albumSection(title: String, albums: [AlbumItem]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
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
                                Text(album.artist)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
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
}

// MARK: - Navigation Routes

struct AlbumRoute: Hashable {
    let id: String
    let name: String
}

struct ArtistRoute: Hashable {
    let id: String
    let name: String
}

struct PlaylistRoute: Hashable {
    let id: String
    let name: String
}
