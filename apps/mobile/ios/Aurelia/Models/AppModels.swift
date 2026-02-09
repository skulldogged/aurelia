import Foundation

// MARK: - Now Playing State

struct NowPlayingState: Equatable {
    var title: String
    var artist: String
    var albumArtUrl: String?
    var isPlaying: Bool
    var isBuffering: Bool = false
    var hasPrevious: Bool = false
    var hasNext: Bool = false
    var albumId: String? = nil
    var artistId: String? = nil
    var albumName: String? = nil
}

// MARK: - Player Snapshot

struct PlayerSnapshot: Equatable {
    var title: String = ""
    var artist: String = ""
    var albumArtUrl: String? = nil
    var isPlaying: Bool = false
    var isBuffering: Bool = false
    var hasPrevious: Bool = false
    var hasNext: Bool = false
    var isShuffled: Bool = false
    var repeatMode: RepeatMode = .none
    var currentSongId: String? = nil
    var currentAlbumId: String? = nil
    var currentArtistId: String? = nil
    var currentAlbumName: String? = nil
    var playbackSpeed: Float = 1.0
    var codec: String? = nil
    var bitRate: Int32? = nil
    var sampleRate: Int32? = nil
}

// MARK: - Playback Position

/// Lightweight struct for frequently-updating playback position.
/// Separated from `PlayerSnapshot` so that views reading only display state
/// (title, art, controls) are not invalidated by position ticks.
struct PlaybackPosition: Equatable {
    var positionMs: Int64 = 0
    var durationMs: Int64 = 0
    var updateTimeMs: Int64 = 0
}

// MARK: - Repeat Mode

enum RepeatMode: Equatable {
    case none
    case one
    case all
}

// MARK: - Featured Album

struct FeaturedAlbum: Identifiable, Equatable {
    var id: String
    var name: String
    var artist: String
    var albumArtUrl: String?
    var songCount: Int
}

// MARK: - Album Item

struct AlbumItem: Identifiable, Equatable {
    var id: String
    var name: String
    var artist: String
    var albumArtUrl: String?
    var songCount: Int
}

// MARK: - Lyrics

struct SyncedWord: Equatable {
    var time: TimeInterval
    var endTime: TimeInterval?
    var word: String
}

struct SyncedLine: Equatable {
    var time: TimeInterval
    var endTime: TimeInterval?
    var line: String
    var words: [SyncedWord]?
    var agentId: String?
}

struct LyricsSection: Equatable {
    var name: String
    var startTime: TimeInterval
    var endTime: TimeInterval
    var lines: [SyncedLine]
    var agentId: String?
}

struct LyricsAgent: Equatable {
    var id: String
    var agentType: String
}

struct Lyrics: Equatable {
    var plain: String?
    var synced: [SyncedLine]?
    var sections: [LyricsSection]?
    var agents: [LyricsAgent]?
    var songwriters: [String]?
    var language: String?
    var areFromRemote: Bool

    var isValid: Bool {
        (plain != nil && !plain!.isEmpty) || (synced != nil && !synced!.isEmpty)
    }

    /// Check if an agent ID refers to a background/other voice.
    func isBackgroundVocal(_ agentId: String?) -> Bool {
        guard let agentId = agentId, let agents = agents else { return false }
        return agents.first(where: { $0.id == agentId })?.agentType == "other"
    }
}

// MARK: - UI Constants

enum UIConstants {
    static let mostPlayedLimit = 10
    static let recentlyPlayedLimit = 10
    static let albumSectionLimit = 10
    static let featuredAlbumsLimit = 5
}
