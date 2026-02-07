import AureliaCore
import SwiftUI

struct AlbumDetailView: View {
    let albumId: String
    let albumName: String

    @EnvironmentObject private var playerController: AudioPlayerController
    @State private var album: Album?
    @State private var songs: [Song] = []
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
                    HStack(alignment: .top, spacing: AureliaSpacing.xl) {
                        playlistSummary
                            .frame(maxWidth: 360)

                        playlistSongs
                            .frame(maxWidth: .infinity, alignment: .top)
                    }
                    .padding(.horizontal, AureliaSpacing.xl)
                    .padding(.vertical, AureliaSpacing.l)
                }
                .scrollIndicators(.hidden)
            }
        }
        .navigationTitle(albumName)
        .onAppear { loadAlbum() }
        .aureliaScreen()
    }

    private var playlistSummary: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.l) {
            VStack(spacing: AureliaSpacing.m) {
                AlbumArtView(url: album?.albumArtUrl ?? songs.first?.albumArtUrl, size: .large, customDimension: 240)

                VStack(spacing: 4) {
                    Text(album?.name ?? albumName)
                        .font(.title2.bold())
                    Text(album?.artist ?? songs.first?.artists?.first ?? "")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                HStack(spacing: 16) {
                    Button {
                        playSongs(startIndex: 0)
                    } label: {
                        Label("Play", systemImage: "play.fill")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.glass)

                    Button {
                        shuffleSongs()
                    } label: {
                        Label("Shuffle", systemImage: "shuffle")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.glass)
                }
            }
        }
    }

    private var playlistSongs: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m) {
            if songs.isEmpty {
                Text("No songs available for this album yet.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.vertical, AureliaSpacing.l)
            } else {
                LazyVStack(spacing: 0) {
                    ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                        SongRow(song: song, isPlaying: song.id == playerController.snapshot.currentSongId, showTrackNumber: true) {
                            playSongs(startIndex: index)
                        }
                        if index != songs.count - 1 {
                            Divider()
                        }
                    }
                }
            }
        }
    }

    private func loadAlbum() {
        let sessionStore = SessionStore.shared
        guard let creds = sessionStore.getCredentials() else {
            error = "Missing session data"
            isLoading = false
            return
        }

        Task {
            let appDataDir = sessionStore.getAppDataDir() ?? ""

            if let cachedSongs = try? loadCachedSongs(appDataDir: appDataDir) {
                let albumSongs = cachedSongs.filter { $0.albumId == albumId }
                if !albumSongs.isEmpty {
                    songs = albumSongs
                    isLoading = false
                }
            }

            if let cached = try? getCachedAlbum(appDataDir: appDataDir, albumId: albumId) {
                album = cached
                if let embedded = cached.songs, !embedded.isEmpty {
                    songs = embedded
                }
                isLoading = false
            }

            do {
                let fetched = try await fetchAlbum(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    albumId: albumId,
                    appDataDir: appDataDir
                )
                album = fetched
                if let embeddedSongs = fetched.songs, !embeddedSongs.isEmpty {
                    songs = embeddedSongs
                }
                isLoading = false
                error = nil
            } catch {
                if !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    if songs.isEmpty && album == nil {
                        self.error = error.localizedDescription
                    }
                    isLoading = false
                }
            }
        }
    }

    private func playSongs(startIndex: Int) {
        guard let serverUrl = SessionStore.shared.serverUrl,
              let token = SessionStore.shared.token,
              !songs.isEmpty else { return }
        playerController.setQueue(songs, serverUrl: serverUrl, token: token, startIndex: startIndex)
    }

    private func shuffleSongs() {
        guard let serverUrl = SessionStore.shared.serverUrl,
              let token = SessionStore.shared.token,
              !songs.isEmpty else { return }
        playerController.setQueue(songs.shuffled(), serverUrl: serverUrl, token: token)
    }
}
