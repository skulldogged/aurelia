package com.aurelia.app.ui

import uniffi.aurelia_core.Song

data class LibraryState(
    val isLoading: Boolean = false,
    val songs: List<Song> = emptyList(),
    val error: String? = null,
    val nowPlaying: NowPlayingState? = null,
    val currentSongId: String? = null,
)

data class NowPlayingState(
    val title: String,
    val artist: String,
    val albumArtUrl: String?,
    val isPlaying: Boolean,
    val isBuffering: Boolean = false,
    val hasPrevious: Boolean = false,
    val hasNext: Boolean = false,
)
