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
        GeometryReader { proxy in
            let isWide = AureliaLayout.isWide(proxy.size.width)
            let horizontalPadding: CGFloat = isWide ? AureliaSpacing.xl : AureliaSpacing.m
            let artDimension: CGFloat = isWide ? 260 : 220

            Group {
                if isLoading {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let error {
                    ContentUnavailableView("Failed to Load", systemImage: "exclamationmark.triangle", description: Text(error))
                } else {
                    ScrollView {
                        if isWide {
                            HStack(alignment: .top, spacing: AureliaSpacing.xl) {
                                albumSummary(artDimension: artDimension)
                                    .frame(maxWidth: 360)

                                albumSongs
                                    .frame(maxWidth: .infinity)
                            }
                            .padding(.horizontal, horizontalPadding)
                            .padding(.vertical, AureliaSpacing.l)
                        } else {
                            VStack(spacing: AureliaSpacing.l) {
                                albumSummary(artDimension: artDimension)
                                albumSongs
                            }
                            .padding(.horizontal, horizontalPadding)
                            .padding(.vertical, AureliaSpacing.l)
                        }
                    }
                }
            }
        }
        .navigationTitle(albumName)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear { loadAlbum() }
        .aureliaScreen()
    }

    private func albumSummary(artDimension: CGFloat) -> some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.l) {
            VStack(spacing: AureliaSpacing.m) {
                AlbumArtView(url: album?.albumArtUrl ?? songs.first?.albumArtUrl, size: .large, customDimension: artDimension)

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
                    .buttonStyle(.borderedProminent)

                    Button {
                        shuffleSongs()
                    } label: {
                        Label("Shuffle", systemImage: "shuffle")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                }
            }
        }
    }

    private var albumSongs: some View {
        GlassCard(cornerRadius: AureliaRadius.l, padding: AureliaSpacing.m) {
            LazyVStack(spacing: 0) {
                let groupedSongs = groupSongsByDisc(songs)
                ForEach(groupedSongs) { group in
                    if group.showDiscHeader {
                        HStack {
                            Text("Disc \(group.discNumber)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .padding(.vertical, 6)
                                .padding(.horizontal, 12)
                                .background(.ultraThinMaterial, in: Capsule())
                            Spacer()
                        }
                        .padding(.vertical, 6)
                    }

                    ForEach(Array(group.songs.enumerated()), id: \.element.id) { index, song in
                        let globalIndex = songs.firstIndex(where: { $0.id == song.id }) ?? 0
                        SongRow(
                            song: song,
                            isPlaying: song.id == playerController.snapshot.currentSongId,
                            showTrackNumber: true
                        ) {
                            playSongs(startIndex: globalIndex)
                        }
                        if index != group.songs.count - 1 {
                            Divider()
                        }
                    }
                }
            }
        }
    }

    private func groupSongsByDisc(_ songs: [Song]) -> [DiscGroup] {
        let sorted = songs.sorted(by: {
            ($0.trackNumber ?? 0) < ($1.trackNumber ?? 0)
        })

        return [
            DiscGroup(
                discNumber: 1,
                songs: sorted,
                showDiscHeader: false
            )
        ]
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

            if let cached = try? getCachedAlbum(appDataDir: appDataDir, albumId: self.albumId) {
                await MainActor.run {
                    self.album = cached
                    self.songs = cached.songs ?? []
                    self.isLoading = false
                }
            }

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
                    self.songs = (fetched.songs ?? []).sorted(by: {
                        ($0.trackNumber ?? 0) < ($1.trackNumber ?? 0)
                    })
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

struct DiscGroup: Identifiable {
    let id = UUID()
    let discNumber: Int
    let songs: [Song]
    let showDiscHeader: Bool
}
