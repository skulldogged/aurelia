package com.aurelia.app.ui

import uniffi.aurelia_core.Playlist
import uniffi.aurelia_core.Song

data class PlaylistsState(
  val isLoading: Boolean = true,
  val playlists: List<Playlist> = emptyList(),
  val error: String? = null,
  val isCreating: Boolean = false,
  val isDeleting: Boolean = false,
)

data class PlaylistDetailState(
  val isLoading: Boolean = true,
  val playlist: Playlist? = null,
  val songs: List<Song> = emptyList(),
  val error: String? = null,
)
