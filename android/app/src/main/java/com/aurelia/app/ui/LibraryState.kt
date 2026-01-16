package com.aurelia.app.ui

import uniffi.aurelia_core.Song

data class LibraryState(
  val isLoading: Boolean = false,
  val songs: List<Song> = emptyList(),
  val error: String? = null,
)
