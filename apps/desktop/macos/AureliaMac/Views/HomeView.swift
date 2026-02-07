import AureliaCore
import SwiftUI

struct HomeView: View {
    @EnvironmentObject private var playerController: AudioPlayerController
    @StateObject private var viewModel = HomeViewModel()
    @State private var featuredSelection: AlbumRoute? = nil

    var body: some View {
        NavigationStack {
            GeometryReader { proxy in
                let isWide = AureliaLayout.isWide(proxy.size.width)
                let cardWidth: CGFloat = isWide ? 176 : 156

                ScrollView {
                    if viewModel.isLoading && viewModel.featuredAlbums.isEmpty {
                        ProgressView()
                            .frame(maxWidth: .infinity, minHeight: 200)
                    } else if let error = viewModel.error, viewModel.featuredAlbums.isEmpty {
                        ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                    } else {
                        LazyVStack(alignment: .leading, spacing: AureliaSpacing.l) {
                            if !viewModel.featuredAlbums.isEmpty {
                                FeaturedHeroView(albums: viewModel.featuredAlbums, availableWidth: proxy.size.width) { album in
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
            .navigationTitle("Home")
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
                            GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s) {
                                VStack(alignment: .leading, spacing: 6) {
                                    AlbumArtView(url: song.albumArtUrl, size: .medium, customDimension: cardWidth)
                                    Text(song.name)
                                        .font(.subheadline.weight(.semibold))
                                        .lineLimit(1)
                                    Text(song.artists?.first ?? "")
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(1)
                                }
                                .frame(width: cardWidth)
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, AureliaSpacing.m)
            }
            .scrollIndicators(.hidden)
        }
    }

    private func albumSection(title: String, albums: [AlbumItem], cardWidth: CGFloat) -> some View {
        VStack(alignment: .leading, spacing: AureliaSpacing.s) {
            AureliaSectionHeader(title: title)

            ScrollView(.horizontal, showsIndicators: false) {
                LazyHStack(spacing: AureliaSpacing.m) {
                    ForEach(albums) { album in
                        NavigationLink(value: AlbumRoute(id: album.id, name: album.name)) {
                            GlassCard(cornerRadius: AureliaRadius.m, padding: AureliaSpacing.s) {
                                VStack(alignment: .leading, spacing: 6) {
                                    AlbumArtView(url: album.albumArtUrl, size: .medium, customDimension: cardWidth)
                                    Text(album.name)
                                        .font(.subheadline.weight(.semibold))
                                        .lineLimit(1)
                                    Text(album.artist)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(1)
                                }
                                .frame(width: cardWidth)
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, AureliaSpacing.m)
            }
            .scrollIndicators(.hidden)
        }
    }
}
