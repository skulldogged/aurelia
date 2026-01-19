package com.aurelia.app.ui.navigation

sealed class Screen(
    val route: String,
) {
    data object Home : Screen("home")

    data object Songs : Screen("songs")

    data object Albums : Screen("albums")

    data object Artists : Screen("artists")

    data object Playlists : Screen("playlists")

    data object Search : Screen("search")

    data object Settings : Screen("settings")

    // Detail screens with parameters
    data class AlbumDetail(
        val albumId: String,
        val albumName: String,
    ) : Screen("album/$albumId")

    data class ArtistDetail(
        val artistId: String,
        val artistName: String,
    ) : Screen("artist/$artistId")
}
