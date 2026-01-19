package com.aurelia.app.ui

import uniffi.aurelia_core.Song

/**
 * Featured album data for the hero section
 */
data class FeaturedAlbum(
    val id: String,
    val name: String,
    val artist: String,
    val albumArtUrl: String?,
    val songCount: Int,
)

/**
 * State for the Home screen
 */
data class HomeState(
    val isLoading: Boolean = false,
    val error: String? = null,
    // Featured albums for hero carousel
    val featuredAlbums: List<FeaturedAlbum> = emptyList(),
    val currentFeaturedIndex: Int = 0,
    // Most played songs (sorted by playCount)
    val mostPlayed: List<Song> = emptyList(),
    // Recently played songs (sorted by datePlayed)
    val recentlyPlayed: List<Song> = emptyList(),
    // Recently added albums (sorted by dateCreated)
    val recentlyAddedAlbums: List<AlbumItem> = emptyList(),
    // Random albums from library
    val randomAlbums: List<AlbumItem> = emptyList(),
    // Player state
    val nowPlaying: NowPlayingState? = null,
    val currentSongId: String? = null,
)
