import AureliaCore
import SwiftUI

struct AlbumDetailView: View {
    let albumId: String
    let albumName: String

    @Environment(AudioPlayerController.self) private var playerController
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var album: Album?
    @State private var songs: [Song] = []
    @State private var isLoading = true
    @State private var error: String?

    private var isWide: Bool {
        horizontalSizeClass == .regular
    }

    var body: some View {
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
            if songs.isEmpty {
                Text("No songs available for this album yet.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.vertical, AureliaSpacing.l)
            } else {
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
    }

    private func groupSongsByDisc(_ songs: [Song]) -> [DiscGroup] {
        let sorted = Self.sortSongs(songs)
        let grouped = Dictionary(grouping: sorted) { Int($0.discNumber ?? 1) }
        let discs = grouped.keys.sorted()
        let showDiscHeader = discs.count > 1

        return discs.compactMap { disc in
            guard let discSongs = grouped[disc] else { return nil }
            return DiscGroup(discNumber: disc, songs: discSongs, showDiscHeader: showDiscHeader)
        }
    }

    private nonisolated static func sortSongs(_ songs: [Song]) -> [Song] {
        songs.sorted { lhs, rhs in
            let lhsDisc = Int(lhs.discNumber ?? 1)
            let rhsDisc = Int(rhs.discNumber ?? 1)
            if lhsDisc != rhsDisc {
                return lhsDisc < rhsDisc
            }

            let lhsTrack = Int(lhs.trackNumber ?? Int32.max)
            let rhsTrack = Int(rhs.trackNumber ?? Int32.max)
            if lhsTrack != rhsTrack {
                return lhsTrack < rhsTrack
            }

            return lhs.name.localizedCaseInsensitiveCompare(rhs.name) == .orderedAscending
        }
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

            if let cachedSongs = try? loadCachedSongs(appDataDir: appDataDir) {
                let albumSongs = Self.sortSongs(cachedSongs.filter { $0.albumId == albumId })
                if !albumSongs.isEmpty {
                    await MainActor.run {
                        songs = albumSongs
                        isLoading = false
                    }
                }
            }

            if let cached = try? getCachedAlbum(appDataDir: appDataDir, albumId: albumId) {
                await MainActor.run {
                    album = cached
                    let embeddedSongs = Self.sortSongs(cached.songs ?? [])
                    if !embeddedSongs.isEmpty {
                        songs = embeddedSongs
                    }
                    isLoading = false
                }
            }

            do {
                let fetched = try await fetchAlbum(
                    serverUrl: creds.serverUrl,
                    token: creds.token,
                    userId: creds.userId,
                    albumId: albumId,
                    appDataDir: appDataDir
                )
                await MainActor.run {
                    album = fetched
                    let embeddedSongs = Self.sortSongs(fetched.songs ?? [])
                    if !embeddedSongs.isEmpty {
                        songs = embeddedSongs
                    }
                    isLoading = false
                    error = nil
                }
            } catch {
                if await !AuthInterceptor.shared.handlePotentialAuthError(error) {
                    await MainActor.run {
                        if songs.isEmpty, album == nil {
                            self.error = error.localizedDescription
                        }
                        isLoading = false
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
