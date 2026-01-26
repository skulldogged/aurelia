package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.PlayerSnapshot
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.buildStreamUrl
import uniffi.aurelia_core.fetchSongs
import uniffi.aurelia_core.loadCachedSongs

class LibraryViewModel(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController,
) : ViewModel() {
  private val mutableState = MutableStateFlow(LibraryState())
  val state: StateFlow<LibraryState> = mutableState

  // Cache for song ID lookup - built once when songs load
  private var songIdByTitleArtist: Map<Pair<String, String>, String> = emptyMap()

  // Track last snapshot to avoid redundant updates
  private var lastTitle: String = ""
  private var lastArtist: String = ""
  private var lastIsPlaying: Boolean = false

  init {
    playerController.observe { snapshot ->
      handlePlayerUpdate(snapshot)
    }
  }

  private fun handlePlayerUpdate(snapshot: PlayerSnapshot) {
    // Only update if something meaningful changed
    val titleChanged = snapshot.title != lastTitle
    val artistChanged = snapshot.artist != lastArtist
    val playingChanged = snapshot.isPlaying != lastIsPlaying

    if (!titleChanged && !artistChanged && !playingChanged) {
      return
    }

    lastTitle = snapshot.title
    lastArtist = snapshot.artist
    lastIsPlaying = snapshot.isPlaying

    if (snapshot.title.isBlank()) {
      mutableState.update { it.copy(nowPlaying = null, currentSongId = null) }
      return
    }

    // Look up song ID from cache
    val songId = songIdByTitleArtist[Pair(snapshot.title, snapshot.artist)]

    mutableState.update {
      it.copy(
        nowPlaying =
          NowPlayingState(
            title = snapshot.title,
            artist = snapshot.artist,
            albumArtUrl = snapshot.albumArtUrl,
            isPlaying = snapshot.isPlaying,
            isBuffering = snapshot.isBuffering,
            hasPrevious = snapshot.hasPrevious,
            hasNext = snapshot.hasNext,
            albumId = snapshot.currentAlbumId,
            artistId = snapshot.currentArtistId,
            albumName = snapshot.currentAlbumName,
          ),
        currentSongId = songId,
      )
    }
  }

  private fun buildSongIdCache(songs: List<uniffi.aurelia_core.Song>) {
    songIdByTitleArtist =
      songs.associate { song ->
        val artist = song.artists?.joinToString(", ") ?: ""
        Pair(song.name, artist) to song.id
      }
  }

  fun loadLibrary() {
    val serverUrl = sessionStore.getServerUrl()
    val userId = sessionStore.getUserId()
    val token = sessionStore.getToken()
    val appDataDir = sessionStore.getAppDataDir()

    if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
      mutableState.update { it.copy(error = "Missing session data") }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    if (!appDataDir.isNullOrBlank()) {
      viewModelScope.launch(Dispatchers.IO) {
        try {
          val cachedSongs = loadCachedSongs(appDataDir)
          if (cachedSongs.isNotEmpty()) {
            buildSongIdCache(cachedSongs)
            mutableState.update { it.copy(songs = cachedSongs, isLoading = false) }
          }
        } catch (_: Exception) {
          // Ignore cache errors and fall back to network fetch.
        }
      }
    }

    viewModelScope.launch(Dispatchers.IO) {
      try {
        val songs = fetchSongs(serverUrl, token, userId, appDataDir ?: "")
        buildSongIdCache(songs)
        mutableState.update { it.copy(isLoading = false, songs = songs) }
      } catch (error: AppException) {
        if (!AuthInterceptor.handlePotentialAuthError(error.message)) {
          mutableState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load") }
        }
      } catch (error: Exception) {
        if (!AuthInterceptor.handlePotentialAuthError(error)) {
          mutableState.update { it.copy(isLoading = false, error = "Failed to load") }
        }
      }
    }
  }

  fun play(
    songId: String,
    container: String?,
    title: String,
    artist: String?,
    albumArtUrl: String? = null,
  ) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val url = buildStreamUrl(serverUrl, token, songId, container)
    val song = mutableState.value.songs.firstOrNull { it.id == songId }
    val durationMs = (song?.duration ?: 0.0).let { (it * 1000).toLong() }

    // Update current song ID immediately for responsive UI
    mutableState.update { it.copy(currentSongId = songId) }

    playerController.play(url, songId, title, artist, albumArtUrl, durationMs, song?.albumId, song?.artistIds?.firstOrNull())
  }

  /**
   * Plays a song from the current song list, setting up the full queue for next/previous navigation.
   * @param songId The ID of the song to start playing
   */
  fun playFromList(songId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val songs = mutableState.value.songs

    val startIndex = songs.indexOfFirst { it.id == songId }
    if (startIndex < 0) return

    // Build queue items on background thread to avoid UI stutter
    viewModelScope.launch(Dispatchers.Default) {
      val queueItems =
        songs.map { song ->
          QueueItem(
            id = song.id,
            uri = buildStreamUrl(serverUrl, token, song.id, song.container),
            title = song.name,
            artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
            albumArtUrl = song.albumArtUrl,
            durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
            isFavorite = song.isFavorite ?: false,
            albumId = song.albumId,
            artistId = song.artistIds?.firstOrNull(),
            albumName = song.album,
          )
        }

      // Update current song ID immediately for responsive UI
      mutableState.update { it.copy(currentSongId = songId) }

      playerController.setQueue(queueItems, startIndex)
    }
  }

  fun togglePlayPause() {
    val nowPlaying = mutableState.value.nowPlaying ?: return
    if (nowPlaying.isPlaying) {
      playerController.pause()
    } else {
      playerController.resume()
    }
  }

  fun skipPrevious() {
    playerController.skipPrevious()
  }

  fun skipNext() {
    playerController.skipNext()
  }
}
