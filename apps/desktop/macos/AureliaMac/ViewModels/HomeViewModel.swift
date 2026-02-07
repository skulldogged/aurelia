import AureliaCore
import Foundation

@MainActor
final class HomeViewModel: ObservableObject {
    @Published var isLoading = false
    @Published var error: String?
    @Published var featuredAlbums: [FeaturedAlbum] = []
    @Published var mostPlayed: [Song] = []
    @Published var recentlyPlayed: [Song] = []
    @Published var recentlyAddedAlbums: [AlbumItem] = []
    @Published var randomAlbums: [AlbumItem] = []

    private let sessionStore = SessionStore.shared
    private var hasLoadedInitialData = false

    func loadHomeData() {
        guard !hasLoadedInitialData else { return }

        isLoading = true
        error = nil

        Task {
            guard let creds = sessionStore.getCredentials(),
                  !creds.serverUrl.isEmpty, !creds.token.isEmpty, !creds.userId.isEmpty else {
                isLoading = false
                error = "Missing session data"
                return
            }

            let appDataDir = sessionStore.getAppDataDir() ?? ""
            var loadedCache = false

            if !appDataDir.isEmpty {
                do {
                    let cached = try loadCachedSongs(appDataDir: appDataDir)
                    if !cached.isEmpty {
                        loadedCache = true
                        applyHomeSections(from: cached)
                        isLoading = false
                    }
                } catch {}
            }

            if loadedCache && !sessionStore.shouldRefreshLibrary() {
                hasLoadedInitialData = true
                return
            }

            do {
                let songs = try await fetchSongs(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    appDataDir: appDataDir
                )
                applyHomeSections(from: songs)
                isLoading = false
                hasLoadedInitialData = true
                sessionStore.markLibraryRefreshed()
            } catch {
                if !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    isLoading = false
                    self.error = error.localizedDescription
                }
            }
        }
    }

    func playSongFromList(_ songId: String, songList: [Song], playerController: AudioPlayerController) {
        guard let serverUrl = sessionStore.serverUrl, let token = sessionStore.token else { return }
        guard let startIndex = songList.firstIndex(where: { $0.id == songId }) else { return }
        playerController.setQueue(songList, serverUrl: serverUrl, token: token, startIndex: startIndex)
    }

    private func applyHomeSections(from songs: [Song]) {
        let derived = deriveMobileHomeData(
            songs: songs,
            mostPlayedLimit: Int64(UIConstants.mostPlayedLimit),
            recentlyPlayedLimit: Int64(UIConstants.recentlyPlayedLimit),
            albumSectionLimit: Int64(UIConstants.albumSectionLimit),
            featuredAlbumsLimit: Int64(UIConstants.featuredAlbumsLimit)
        )

        mostPlayed = derived.mostPlayed
        recentlyPlayed = derived.recentlyPlayed
        recentlyAddedAlbums = derived.recentlyAdded.map {
            AlbumItem(
                id: $0.id ?? "",
                name: $0.name,
                artist: $0.artist,
                albumArtUrl: $0.albumArtUrl,
                songCount: Int($0.songCount)
            )
        }
        randomAlbums = derived.randomAlbums.map {
            AlbumItem(
                id: $0.id ?? "",
                name: $0.name,
                artist: $0.artist,
                albumArtUrl: $0.albumArtUrl,
                songCount: Int($0.songCount)
            )
        }
        featuredAlbums = derived.featuredAlbums.map {
            FeaturedAlbum(
                id: $0.id ?? "",
                name: $0.name,
                artist: $0.artist,
                albumArtUrl: $0.albumArtUrl,
                songCount: Int($0.songCount)
            )
        }
    }
}
