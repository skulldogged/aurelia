import Foundation

struct PlayerSnapshot: Equatable {
    var title: String = ""
    var artist: String = ""
    var albumArtUrl: String? = nil
    var isPlaying: Bool = false
    var positionMs: Int64 = 0
    var durationMs: Int64 = 0
    var hasPrevious: Bool = false
    var hasNext: Bool = false
    var currentSongId: String? = nil
}

struct FeaturedAlbum: Identifiable, Equatable {
    var id: String
    var name: String
    var artist: String
    var albumArtUrl: String?
    var songCount: Int
}

struct AlbumItem: Identifiable, Equatable {
    var id: String
    var name: String
    var artist: String
    var albumArtUrl: String?
    var songCount: Int
}

struct AlbumRoute: Hashable, Identifiable {
    var id: String
    var name: String
}

struct ArtistRoute: Hashable, Identifiable {
    var id: String
    var name: String
}

struct PlaylistRoute: Hashable, Identifiable {
    var id: String
    var name: String
}

enum UIConstants {
    static let mostPlayedLimit = 12
    static let recentlyPlayedLimit = 12
    static let albumSectionLimit = 12
    static let featuredAlbumsLimit = 8
}
