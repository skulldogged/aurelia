import SwiftUI
import AureliaCore

struct AlbumDetailView: View {
    let albumId: String
    let albumName: String

    @Environment(AudioPlayerController.self) private var playerController
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
                    VStack(spacing: 16) {
                        // Album Header
                        AlbumArtView(url: album?.albumArtUrl ?? songs.first?.albumArtUrl, size: .large)
                            .frame(width: 220, height: 220)

                        VStack(spacing: 4) {
                            Text(album?.name ?? albumName)
                                .font(.title2.bold())
                            Text(album?.artist ?? songs.first?.artists?.first ?? "")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
//                            if let year = album?.year {
//                                Text(String(year))
//                                    .font(.caption)
//                                    .foregroundStyle(.tertiary)
//                            }
                        }

                        // Play / Shuffle buttons
                        HStack(spacing: 16) {
                            Button {
                                playSongs(startIndex: 0)
                            } label: {
                                Label("Play", systemImage: "play.fill")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.borderedProminent)

                            Button {
                                shuffleSongs()
                            } label: {
                                Label("Shuffle", systemImage: "shuffle")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.bordered)
                        }
                        .padding(.horizontal)

                        // Song list
                        LazyVStack(spacing: 0) {
                            ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                                SongRow(
                                    song: song,
                                    isPlaying: song.id == playerController.snapshot.currentSongId,
                                    showTrackNumber: true
                                ) {
                                    playSongs(startIndex: index)
                                }
                                Divider()
                            }
                        }
                        .padding(.horizontal)
                    }
                    .padding(.vertical)
                }
            }
        }
        .navigationTitle(albumName)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear { loadAlbum() }
    }

    private func loadAlbum() {
        let sessionStore = SessionStore.shared
        guard let creds = sessionStore.getCredentials() else {
            error = "Missing session data"
            isLoading = false
            return
        }

        Task.detached {
            let appDataDir = await sessionStore.getAppDataDir() ?? ""

            // Try cache first
            if let cached = try? getCachedAlbum(appDataDir: appDataDir, albumId: self.albumId) {
                await MainActor.run {
                    self.album = cached
                    self.songs = cached.songs ?? []
                    self.isLoading = false
                }
            }

            // Fetch fresh
            do {
                let fetched = try fetchAlbum(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    albumId: self.albumId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    self.album = fetched
                    self.songs = (fetched.songs ?? []).sorted { ($0.trackNumber ?? 0) < ($1.trackNumber ?? 0) }
                    self.isLoading = false
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        if self.songs.isEmpty {
                            self.error = error.localizedDescription
                        }
                        self.isLoading = false
                    }
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
