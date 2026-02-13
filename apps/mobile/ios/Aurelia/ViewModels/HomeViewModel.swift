import AureliaCore
import Foundation
import Observation
import os

@Observable
final class HomeViewModel: @unchecked Sendable {
    var isLoading = false
    var error: String?
    var featuredAlbums: [FeaturedAlbum] = []
    var currentFeaturedIndex = 0
    var mostPlayed: [Song] = []
    var recentlyPlayed: [Song] = []
    var recentlyAddedAlbums: [AlbumItem] = []
    var randomAlbums: [AlbumItem] = []
    var nowPlaying: NowPlayingState?
    var currentSongId: String?

    private let sessionStore = SessionStore.shared
    private let logger = Logger(subsystem: "com.aurelia.app", category: "HomeViewModel")
    private var hasLoadedInitialData = false
    private var allSongs: [Song] = []
    private var songIdByTitleArtist: [String: String] = [:]
    private nonisolated static let useSharedHomeDerivation = true

    private struct HomeSections {
        let mostPlayed: [Song]
        let recentlyPlayed: [Song]
        let recentlyAddedAlbums: [AlbumItem]
        let randomAlbums: [AlbumItem]
        let featuredAlbums: [FeaturedAlbum]
    }

    private struct HomeSectionLimits: Sendable {
        let mostPlayed: Int
        let recentlyPlayed: Int
        let albumSection: Int
        let featuredAlbums: Int
    }

    func loadHomeData() {
        guard !hasLoadedInitialData else { return }

        isLoading = true
        error = nil

        let sectionLimits = HomeSectionLimits(
            mostPlayed: UIConstants.mostPlayedLimit,
            recentlyPlayed: UIConstants.recentlyPlayedLimit,
            albumSection: UIConstants.albumSectionLimit,
            featuredAlbums: UIConstants.featuredAlbumsLimit
        )

        Task.detached { [self] in
            guard let creds = await sessionStore.getCredentialsAsync(),
                  !creds.serverUrl.isEmpty, !creds.token.isEmpty, !creds.userId.isEmpty
            else {
                await MainActor.run {
                    self.isLoading = false
                    self.error = "Missing session data"
                }
                return
            }

            let appDataDir = await MainActor.run { sessionStore.getAppDataDir() ?? "" }
            let shouldRefresh = await sessionStore.shouldRefreshLibraryAsync()

            // Load cached data first
            var loadedCache = false
            if !appDataDir.isEmpty {
                do {
                    let cached = try loadCachedSongs(appDataDir: appDataDir)
                    if !cached.isEmpty {
                        loadedCache = true
                        let sections = Self.computeHomeSections(cached, limits: sectionLimits)
                        let songIdCache = Self.buildSongIdCache(cached)
                        await MainActor.run {
                            self.allSongs = cached
                            self.songIdByTitleArtist = songIdCache
                            self.applyHomeSections(sections)
                        }
                    }
                } catch {
                    logger.warning("Failed to load cached songs: \(error)")
                }
            }

            if loadedCache, !shouldRefresh {
                await MainActor.run {
                    self.hasLoadedInitialData = true
                }
                return
            }

            // Fetch fresh data
            do {
                let songs = try await fetchSongs(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    appDataDir: appDataDir
                )
                let sections = Self.computeHomeSections(songs, limits: sectionLimits)
                let songIdCache = Self.buildSongIdCache(songs)
                await MainActor.run {
                    self.allSongs = songs
                    self.songIdByTitleArtist = songIdCache
                    self.applyHomeSections(sections)
                    self.hasLoadedInitialData = true
                    self.sessionStore.markLibraryRefreshed()
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        self.isLoading = false
                        self.error = error.localizedDescription
                    }
                }
            }
        }
    }

    @MainActor
    private func applyHomeSections(_ sections: HomeSections) {
        mostPlayed = sections.mostPlayed
        recentlyPlayed = sections.recentlyPlayed
        recentlyAddedAlbums = sections.recentlyAddedAlbums
        randomAlbums = sections.randomAlbums
        featuredAlbums = sections.featuredAlbums
        isLoading = false
    }

    private nonisolated static func computeHomeSections(_ songs: [Song], limits: HomeSectionLimits) -> HomeSections {
        if useSharedHomeDerivation {
            let derived = deriveMobileHomeData(
                songs: songs,
                mostPlayedLimit: Int64(limits.mostPlayed),
                recentlyPlayedLimit: Int64(limits.recentlyPlayed),
                albumSectionLimit: Int64(limits.albumSection),
                featuredAlbumsLimit: Int64(limits.featuredAlbums)
            )

            return HomeSections(
                mostPlayed: derived.mostPlayed,
                recentlyPlayed: derived.recentlyPlayed,
                recentlyAddedAlbums: derived.recentlyAdded.map {
                    AlbumItem(
                        id: $0.id ?? "",
                        name: $0.name,
                        artist: $0.artist,
                        albumArtUrl: $0.albumArtUrl,
                        songCount: Int($0.songCount)
                    )
                },
                randomAlbums: derived.randomAlbums.map {
                    AlbumItem(
                        id: $0.id ?? "",
                        name: $0.name,
                        artist: $0.artist,
                        albumArtUrl: $0.albumArtUrl,
                        songCount: Int($0.songCount)
                    )
                },
                featuredAlbums: derived.featuredAlbums.map {
                    FeaturedAlbum(
                        id: $0.id ?? "",
                        name: $0.name,
                        artist: $0.artist,
                        albumArtUrl: $0.albumArtUrl,
                        songCount: Int($0.songCount)
                    )
                }
            )
        }

        // Most played
        let mostPlayed = songs
            .filter { ($0.playCount ?? 0) > 0 }
            .sorted { ($0.playCount ?? 0) > ($1.playCount ?? 0) }
            .prefix(limits.mostPlayed)
            .map(\.self)

        // Recently played
        let recentlyPlayed = songs
            .filter { $0.datePlayed?.isEmpty == false }
            .sorted { ($0.datePlayed ?? "") > ($1.datePlayed ?? "") }
            .prefix(limits.recentlyPlayed)
            .map(\.self)

        // Group by album
        let songsByAlbumId = songs.compactMap { song -> (String, Song)? in
            guard let albumId = song.albumId, !albumId.isEmpty else { return nil }
            return (albumId, song)
        }
        let albumsMap = Dictionary(grouping: songsByAlbumId, by: { $0.0 })

        // Recently added albums
        let recentlyAddedAlbums = albumsMap
            .map { albumId, albumEntries -> (AlbumItem, String) in
                let albumSongs = albumEntries.map(\.1)
                let firstSong = albumSongs.max(by: { ($0.dateCreated ?? "") < ($1.dateCreated ?? "") }) ?? albumSongs[0]
                return (
                    AlbumItem(
                        id: albumId,
                        name: firstSong.album ?? "Unknown Album",
                        artist: firstSong.artists?.first ?? "Unknown Artist",
                        albumArtUrl: firstSong.albumArtUrl,
                        songCount: albumSongs.count
                    ),
                    firstSong.dateCreated ?? ""
                )
            }
            .sorted { $0.1 > $1.1 }
            .prefix(limits.albumSection)
            .map(\.0)

        // Random albums
        let randomAlbums = albumsMap
            .map { albumId, albumEntries -> AlbumItem in
                let albumSongs = albumEntries.map(\.1)
                let firstSong = albumSongs[0]
                return AlbumItem(
                    id: albumId,
                    name: firstSong.album ?? "Unknown Album",
                    artist: firstSong.artists?.first ?? "Unknown Artist",
                    albumArtUrl: firstSong.albumArtUrl,
                    songCount: albumSongs.count
                )
            }
            .shuffled()
            .prefix(limits.albumSection)
            .map(\.self)

        // Featured albums
        let featuredAlbums = albumsMap
            .filter { $0.value.contains { $0.1.albumArtUrl?.isEmpty == false } }
            .map { albumId, albumEntries -> FeaturedAlbum in
                let albumSongs = albumEntries.map(\.1)
                let firstSong = albumSongs[0]
                return FeaturedAlbum(
                    id: albumId,
                    name: firstSong.album ?? "Unknown Album",
                    artist: firstSong.artists?.joined(separator: ", ") ?? "Unknown Artist",
                    albumArtUrl: firstSong.albumArtUrl,
                    songCount: albumSongs.count
                )
            }
            .shuffled()
            .prefix(limits.featuredAlbums)
            .map(\.self)

        return HomeSections(
            mostPlayed: mostPlayed,
            recentlyPlayed: recentlyPlayed,
            recentlyAddedAlbums: recentlyAddedAlbums,
            randomAlbums: randomAlbums,
            featuredAlbums: featuredAlbums
        )
    }

    // MARK: - Playback

    func playSongFromList(_ songId: String, songList: [Song], playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        guard let startIndex = songList.firstIndex(where: { $0.id == songId }) else { return }
        currentSongId = songId
        playerController.setQueue(songList, serverUrl: serverUrl, token: token, startIndex: startIndex)
    }

    func playAlbum(_ albumId: String, playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        let albumSongs = allSongs
            .filter { $0.albumId == albumId }
            .sorted { ($0.trackNumber ?? 0) < ($1.trackNumber ?? 0) }
        guard !albumSongs.isEmpty else { return }
        currentSongId = albumSongs[0].id
        playerController.setQueue(albumSongs, serverUrl: serverUrl, token: token)
    }

    func shuffleAlbum(_ albumId: String, playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        let albumSongs = allSongs.filter { $0.albumId == albumId }.shuffled()
        guard !albumSongs.isEmpty else { return }
        currentSongId = albumSongs[0].id
        playerController.setQueue(albumSongs, serverUrl: serverUrl, token: token)
    }

    // MARK: - Featured Navigation

    func nextFeaturedAlbum() {
        guard !featuredAlbums.isEmpty else { return }
        currentFeaturedIndex = (currentFeaturedIndex + 1) % featuredAlbums.count
    }

    func previousFeaturedAlbum() {
        guard !featuredAlbums.isEmpty else { return }
        currentFeaturedIndex = currentFeaturedIndex > 0 ? currentFeaturedIndex - 1 : featuredAlbums.count - 1
    }

    // MARK: - Helpers

    private nonisolated static func buildSongIdCache(_ songs: [Song]) -> [String: String] {
        var cache: [String: String] = [:]
        for song in songs {
            let key = "\(song.name)_\(song.artists?.first ?? "")"
            cache[key] = song.id
        }
        return cache
    }
}
