import AureliaCore
import SwiftUI

struct HomeView: View {
    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var viewModel = HomeViewModel()
    @State private var featuredSelection: AlbumRoute? = nil

    private var isWide: Bool {
        horizontalSizeClass == .regular
    }

    private var cardWidth: CGFloat {
        isWide ? 180 : 140
    }

    var body: some View {
        NavigationStack {
            GeometryReader { proxy in
                ScrollView {
                    if viewModel.isLoading, viewModel.featuredAlbums.isEmpty {
                        ProgressView()
                            .frame(maxWidth: .infinity, minHeight: 200)
                    } else if let error = viewModel.error, viewModel.featuredAlbums.isEmpty {
                        ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                    } else {
                        LazyVStack(alignment: .leading, spacing: AureliaSpacing.l) {
                            if !viewModel.featuredAlbums.isEmpty {
                                FeaturedHeroView(albums: viewModel.featuredAlbums, isWide: isWide, availableWidth: proxy.size.width) { album in
                                    featuredSelection = AlbumRoute(id: album.id, name: album.name)
                                }
                            }

                            if !viewModel.mostPlayed.isEmpty {
                                songSection(title: "Most Played", songs: viewModel.mostPlayed, cardWidth: cardWidth)
                            }

                            if !viewModel.recentlyPlayed.isEmpty {
                                songSection(title: "Recently Played", songs: viewModel.recentlyPlayed, cardWidth: cardWidth)
                            }

                            if !viewModel.recentlyAddedAlbums.isEmpty {
                                albumSection(title: "Recently Added", albums: viewModel.recentlyAddedAlbums, cardWidth: cardWidth)
                            }

                            if !viewModel.randomAlbums.isEmpty {
                                albumSection(title: "From Your Library", albums: viewModel.randomAlbums, cardWidth: cardWidth)
                            }
                        }
                        .padding(.vertical, AureliaSpacing.l)
                    }
                }
                .scrollIndicators(.hidden)
            }
            .aureliaRootTabHeader("Home")
            .navigationDestination(for: AlbumRoute.self) { route in
                AlbumDetailView(albumId: route.id, albumName: route.name)
            }
            .navigationDestination(item: $featuredSelection) { route in
                AlbumDetailView(albumId: route.id, albumName: route.name)
            }
            .onAppear { viewModel.loadHomeData() }
        }
        .aureliaScreen()
    }

    private func songSection(title: String, songs: [Song], cardWidth: CGFloat) -> some View {
        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            AureliaSectionHeader(title: title)

            ScrollView(.horizontal, showsIndicators: false) {
                LazyHStack(spacing: AureliaSpacing.m) {
                    ForEach(songs, id: \.id) { song in
                        Button {
                            viewModel.playSongFromList(song.id, songList: songs, playerController: playerController)
                        } label: {
                            VStack(alignment: .leading, spacing: 6) {
                                AlbumArtView(url: song.albumArtUrl, size: .medium, customDimension: cardWidth)
                                Text(song.name)
                                    .font(.caption)
                                    .lineLimit(1)
                                Text(song.artists?.first ?? "")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                            }
                            .frame(width: cardWidth)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, AureliaSpacing.m)
            }
        }
    }

    private func albumSection(title: String, albums: [AlbumItem], cardWidth: CGFloat) -> some View {
        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            AureliaSectionHeader(title: title)

            ScrollView(.horizontal, showsIndicators: false) {
                LazyHStack(spacing: AureliaSpacing.m) {
                    ForEach(albums) { album in
                        NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                            VStack(alignment: .leading, spacing: 6) {
                                AlbumArtView(url: album.albumArtUrl, size: .medium, customDimension: cardWidth)
                                Text(album.name)
                                    .font(.caption)
                                    .lineLimit(1)
                                Text(album.artist)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                            }
                            .frame(width: cardWidth)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, AureliaSpacing.m)
            }
        }
    }
}

struct AlbumRoute: Hashable {
    let id: String
    let name: String
}

extension AlbumRoute: Identifiable {}

struct ArtistRoute: Hashable {
    let id: String
    let name: String
}

struct PlaylistRoute: Hashable {
    let id: String
    let name: String
}
