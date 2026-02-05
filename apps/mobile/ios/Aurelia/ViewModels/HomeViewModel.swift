import Foundation
import Observation
import os
import AureliaCore

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

    func loadHomeData() {
        guard !hasLoadedInitialData else { return }
        guard let creds = sessionStore.getCredentials(),
              !creds.serverUrl.isEmpty, !creds.token.isEmpty, !creds.userId.isEmpty else {
            error = "Missing session data"
            return
        }

        isLoading = true
        error = nil

        let appDataDir = sessionStore.getAppDataDir() ?? ""

        // Load cached data first
        Task.detached { [self] in
            if !appDataDir.isEmpty {
                do {
                    let cached = try loadCachedSongs(appDataDir: appDataDir)
                    if !cached.isEmpty {
                        await MainActor.run {
                            self.allSongs = cached
                            self.songIdByTitleArtist = Self.buildSongIdCache(cached)
                            self.processHomeData(cached)
                        }
                    }
                } catch {
                    self.logger.warning("Failed to load cached songs: \(error)")
                }
            }

            // Fetch fresh data
            do {
                let songs = try await fetchSongs(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.allSongs = songs
                    self.songIdByTitleArtist = Self.buildSongIdCache(songs)
                    self.processHomeData(songs)
                    self.hasLoadedInitialData = true
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

    private func processHomeData(_ songs: [Song]) {
        // Most played
        mostPlayed = songs
            .filter { ($0.playCount ?? 0) > 0 }
            .sorted { ($0.playCount ?? 0) > ($1.playCount ?? 0) }
            .prefix(UIConstants.mostPlayedLimit)
            .map { $0 }

        // Recently played
        recentlyPlayed = songs
            .filter { $0.datePlayed != nil && !$0.datePlayed!.isEmpty }
            .sorted { ($0.datePlayed ?? "") > ($1.datePlayed ?? "") }
            .prefix(UIConstants.recentlyPlayedLimit)
            .map { $0 }

        // Group by album
        let albumsMap = Dictionary(grouping: songs.filter { $0.albumId != nil && !$0.albumId!.isEmpty }) { $0.albumId! }

        // Recently added albums
        recentlyAddedAlbums = albumsMap
            .map { (albumId, albumSongs) -> (AlbumItem, String) in
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
            .prefix(UIConstants.albumSectionLimit)
            .map(\.0)

        // Random albums
        randomAlbums = albumsMap
            .map { (albumId, albumSongs) -> AlbumItem in
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
            .prefix(UIConstants.albumSectionLimit)
            .map { $0 }

        // Featured albums
        featuredAlbums = albumsMap
            .filter { $0.value.contains { $0.albumArtUrl != nil && !$0.albumArtUrl!.isEmpty } }
            .map { (albumId, albumSongs) -> FeaturedAlbum in
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
            .prefix(UIConstants.featuredAlbumsLimit)
            .map { $0 }

        isLoading = false
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

    private static func buildSongIdCache(_ songs: [Song]) -> [String: String] {
        var cache: [String: String] = [:]
        for song in songs {
            let key = "\(song.name)_\(song.artists?.first ?? "")"
            cache[key] = song.id
        }
        return cache
    }
}
