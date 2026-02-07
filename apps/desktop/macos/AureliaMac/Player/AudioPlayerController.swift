import AVFoundation
import AureliaCore
import Foundation

@MainActor
final class AudioPlayerController: ObservableObject {
    @Published private(set) var snapshot = PlayerSnapshot()

    private let player = AVPlayer()
    private var songQueue: [Song] = []
    private var currentIndex: Int = -1
    private var serverUrl = ""
    private var token = ""
    private var timeObserver: Any?

    init() {
        timeObserver = player.addPeriodicTimeObserver(
            forInterval: CMTime(seconds: 0.5, preferredTimescale: 600),
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor in
                self?.updateSnapshot()
            }
        }

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleTrackEnded),
            name: .AVPlayerItemDidPlayToEndTime,
            object: nil
        )
    }

    func setQueue(_ songs: [Song], serverUrl: String, token: String, startIndex: Int = 0) {
        self.songQueue = songs
        self.serverUrl = serverUrl
        self.token = token

        guard !songs.isEmpty else {
            stop()
            return
        }

        playQueueItem(startIndex)
    }

    func getQueue() -> [Song] { songQueue }
    func getCurrentQueueIndex() -> Int { currentIndex }

    func playQueueItem(_ index: Int) {
        guard index >= 0, index < songQueue.count else { return }
        currentIndex = index

        let song = songQueue[index]
        let streamUrl = buildStreamUrl(
            serverUrl: serverUrl,
            token: token,
            itemId: song.id,
            container: song.container
        )

        guard let url = URL(string: streamUrl) else { return }

        player.replaceCurrentItem(with: AVPlayerItem(url: url))
        player.play()
        updateSnapshot()
    }

    func pause() {
        player.pause()
        updateSnapshot()
    }

    func resume() {
        player.play()
        updateSnapshot()
    }

    func togglePlayPause() {
        if snapshot.isPlaying {
            pause()
        } else {
            resume()
        }
    }

    func skipNext() {
        guard currentIndex + 1 < songQueue.count else { return }
        playQueueItem(currentIndex + 1)
    }

    func skipPrevious() {
        guard currentIndex > 0 else {
            player.seek(to: .zero)
            return
        }
        playQueueItem(currentIndex - 1)
    }

    func seekTo(positionMs: Int64) {
        let time = CMTime(seconds: Double(max(0, positionMs)) / 1000.0, preferredTimescale: 600)
        player.seek(to: time)
    }

    func stop() {
        player.pause()
        player.replaceCurrentItem(with: nil)
        currentIndex = -1
        songQueue = []
        updateSnapshot()
    }

    @objc private func handleTrackEnded() {
        if currentIndex + 1 < songQueue.count {
            playQueueItem(currentIndex + 1)
        } else {
            updateSnapshot()
        }
    }

    private func updateSnapshot() {
        let currentSong = (currentIndex >= 0 && currentIndex < songQueue.count) ? songQueue[currentIndex] : nil
        let currentTime = player.currentTime().seconds
        let duration = player.currentItem?.duration.seconds ?? 0

        snapshot = PlayerSnapshot(
            title: currentSong?.name ?? "",
            artist: currentSong?.artists?.joined(separator: ", ") ?? "",
            albumArtUrl: currentSong?.albumArtUrl,
            isPlaying: player.rate > 0,
            positionMs: currentTime.isFinite ? Int64(currentTime * 1000) : 0,
            durationMs: duration.isFinite ? Int64(duration * 1000) : 0,
            hasPrevious: currentIndex > 0,
            hasNext: currentIndex >= 0 && currentIndex < songQueue.count - 1,
            currentSongId: currentSong?.id
        )
    }
}
