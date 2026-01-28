package com.aurelia.app.ui

import com.aurelia.app.player.PlayerSnapshot

/**
 * Encapsulates player snapshot diffing logic shared between HomeViewModel and LibraryViewModel.
 * Tracks last-seen title/artist/isPlaying to avoid redundant state updates.
 */
class NowPlayingMapper {
    private var lastTitle: String = ""
    private var lastArtist: String = ""
    private var lastIsPlaying: Boolean = false

    /**
     * Returns true if the snapshot contains meaningful changes worth propagating.
     */
    fun shouldUpdate(snapshot: PlayerSnapshot): Boolean {
        val titleChanged = snapshot.title != lastTitle
        val artistChanged = snapshot.artist != lastArtist
        val playingChanged = snapshot.isPlaying != lastIsPlaying

        if (!titleChanged && !artistChanged && !playingChanged) {
            return false
        }

        lastTitle = snapshot.title
        lastArtist = snapshot.artist
        lastIsPlaying = snapshot.isPlaying
        return true
    }

    /**
     * Maps a player snapshot to NowPlayingState and resolved song ID.
     *
     * @param snapshot The current player snapshot
     * @param songIdCache A (title, artist) -> songId lookup map
     * @param includeNavigation Whether to populate hasPrevious/hasNext fields
     * @return Pair of (NowPlayingState?, resolvedSongId?) - both null if title is blank
     */
    fun mapToNowPlaying(
        snapshot: PlayerSnapshot,
        songIdCache: Map<Pair<String, String>, String>,
        includeNavigation: Boolean = false,
    ): Pair<NowPlayingState?, String?> {
        if (snapshot.title.isBlank()) {
            return Pair(null, null)
        }

        val songId = songIdCache[Pair(snapshot.title, snapshot.artist)]

        val nowPlaying = NowPlayingState(
            title = snapshot.title,
            artist = snapshot.artist,
            albumArtUrl = snapshot.albumArtUrl,
            isPlaying = snapshot.isPlaying,
            isBuffering = snapshot.isBuffering,
            hasPrevious = if (includeNavigation) snapshot.hasPrevious else false,
            hasNext = if (includeNavigation) snapshot.hasNext else false,
            albumId = snapshot.currentAlbumId,
            artistId = snapshot.currentArtistId,
            albumName = snapshot.currentAlbumName,
        )

        return Pair(nowPlaying, songId)
    }
}
