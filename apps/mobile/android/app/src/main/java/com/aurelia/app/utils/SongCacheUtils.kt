package com.aurelia.app.utils

import uniffi.aurelia_core.Song

/**
 * Builds a lookup cache mapping (title, artist) pairs to song IDs.
 * Used by HomeViewModel and LibraryViewModel for matching player snapshots to songs.
 */
fun buildSongIdCache(songs: List<Song>): Map<Pair<String, String>, String> =
    songs.associate { song ->
        val artist = song.artists?.joinToString(", ") ?: ""
        Pair(song.name, artist) to song.id
    }
