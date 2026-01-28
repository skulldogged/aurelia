package com.aurelia.app.ui

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.utils.buildSongIdCache
import com.aurelia.app.utils.validateSession
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
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

  private val nowPlayingMapper = NowPlayingMapper()

  init {
    playerController.observe { snapshot ->
      if (!nowPlayingMapper.shouldUpdate(snapshot)) return@observe
      val (nowPlaying, songId) = nowPlayingMapper.mapToNowPlaying(
        snapshot, songIdByTitleArtist, includeNavigation = true,
      )
      mutableState.update { it.copy(nowPlaying = nowPlaying, currentSongId = songId) }
    }
  }

  fun loadLibrary() {
    val session = validateSession(sessionStore)
    if (session == null) {
      mutableState.update { it.copy(error = "Missing session data") }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    if (!session.appDataDir.isNullOrBlank()) {
      viewModelScope.launch(Dispatchers.IO) {
        try {
          val cachedSongs = loadCachedSongs(session.appDataDir)
          if (cachedSongs.isNotEmpty()) {
            songIdByTitleArtist = buildSongIdCache(cachedSongs)
            mutableState.update { it.copy(songs = cachedSongs, isLoading = false) }
          }
        } catch (e: Exception) {
          Log.w("LibraryViewModel", "Failed to load cached songs", e)
        }
      }
    }

    viewModelScope.launch(Dispatchers.IO) {
      try {
        val songs = fetchSongs(session.serverUrl, session.token, session.userId, session.appDataDir ?: "")
        songIdByTitleArtist = buildSongIdCache(songs)
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

  fun play(songId: String) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val song = mutableState.value.songs.firstOrNull { it.id == songId } ?: return

    // Update current song ID immediately for responsive UI
    mutableState.update { it.copy(currentSongId = songId) }

    playerController.play(song, serverUrl, token)
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

    // Update current song ID immediately for responsive UI
    mutableState.update { it.copy(currentSongId = songId) }

    playerController.setQueue(songs, serverUrl, token, startIndex)
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
