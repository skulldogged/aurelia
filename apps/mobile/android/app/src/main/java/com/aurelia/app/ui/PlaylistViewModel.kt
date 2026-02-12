package com.aurelia.app.ui

import android.os.SystemClock
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.utils.validateSession
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.PlaylistCreateData
import uniffi.aurelia_core.addPlaylistItems
import uniffi.aurelia_core.createPlaylist
import uniffi.aurelia_core.deletePlaylist
import uniffi.aurelia_core.getPlaylistItems
import uniffi.aurelia_core.getPlaylists

class PlaylistViewModel(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController,
) : ViewModel() {
  private val mutableState = MutableStateFlow(PlaylistsState())
  val state: StateFlow<PlaylistsState> = mutableState

  private val mutableDetailState = MutableStateFlow(PlaylistDetailState())
  val detailState: StateFlow<PlaylistDetailState> = mutableDetailState
  private var loadJob: Job? = null
  private var lastLoadedAtMs: Long = 0L

  fun ensureLoaded(force: Boolean = false) {
    if (!force && loadJob?.isActive == true) return
    val hasPlaylists = mutableState.value.playlists.isNotEmpty()
    val isFresh = SystemClock.elapsedRealtime() - lastLoadedAtMs < LOAD_FRESHNESS_MS
    if (!force && hasPlaylists && isFresh) return

    val session = validateSession(sessionStore)
    if (session == null) {
      mutableState.update { it.copy(error = "Missing session data", isLoading = false) }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    loadJob = viewModelScope.launch(Dispatchers.IO) {
      try {
        val playlists = getPlaylists(session.serverUrl, session.token, session.userId)
        mutableState.update { it.copy(isLoading = false, playlists = playlists) }
        lastLoadedAtMs = SystemClock.elapsedRealtime()
      } catch (error: AppException) {
        if (!AuthInterceptor.handlePotentialAuthError(error.message)) {
          mutableState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load playlists") }
        }
      } catch (error: Exception) {
        if (!AuthInterceptor.handlePotentialAuthError(error)) {
          mutableState.update { it.copy(isLoading = false, error = "Failed to load playlists") }
        }
      }
    }
  }

  fun loadPlaylists() {
    ensureLoaded(force = false)
  }

  fun createPlaylist(
    name: String,
    songIds: List<String>? = null,
  ) {
    val session = validateSession(sessionStore) ?: return

    mutableState.update { it.copy(isCreating = true) }

    viewModelScope.launch(Dispatchers.IO) {
      try {
        val data =
          PlaylistCreateData(
            name = name,
            ids = songIds,
            userId = session.userId,
            isPublic = false,
          )
        val newPlaylist = createPlaylist(session.serverUrl, session.token, data)
        mutableState.update { current ->
          current.copy(
            isCreating = false,
            playlists = current.playlists + newPlaylist,
          )
        }
      } catch (error: Exception) {
        mutableState.update { it.copy(isCreating = false, error = "Failed to create playlist") }
      }
    }
  }

  fun deletePlaylist(playlistId: String) {
    val session = validateSession(sessionStore) ?: return

    mutableState.update { it.copy(isDeleting = true) }

    viewModelScope.launch(Dispatchers.IO) {
      try {
        deletePlaylist(session.serverUrl, session.token, playlistId)
        mutableState.update { current ->
          current.copy(
            isDeleting = false,
            playlists = current.playlists.filter { it.id != playlistId },
          )
        }
      } catch (error: Exception) {
        mutableState.update { it.copy(isDeleting = false, error = "Failed to delete playlist") }
      }
    }
  }

  fun loadPlaylistDetail(
    playlistId: String,
    playlistName: String,
  ) {
    val session = validateSession(sessionStore)
    if (session == null) {
      mutableDetailState.update { it.copy(error = "Missing session data", isLoading = false) }
      return
    }

    mutableDetailState.update { it.copy(isLoading = true, error = null) }

    viewModelScope.launch(Dispatchers.IO) {
      try {
        val songs = getPlaylistItems(session.serverUrl, session.token, playlistId)
        // Find the playlist from the main state if available
        val playlist = mutableState.value.playlists.find { it.id == playlistId }
        mutableDetailState.update {
          it.copy(
            isLoading = false,
            playlist = playlist,
            songs = songs,
          )
        }
      } catch (error: AppException) {
        if (!AuthInterceptor.handlePotentialAuthError(error.message)) {
          mutableDetailState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load playlist") }
        }
      } catch (error: Exception) {
        if (!AuthInterceptor.handlePotentialAuthError(error)) {
          mutableDetailState.update { it.copy(isLoading = false, error = "Failed to load playlist") }
        }
      }
    }
  }

  fun addSongsToPlaylist(
    playlistId: String,
    songIds: List<String>,
  ) {
    val session = validateSession(sessionStore) ?: return

    viewModelScope.launch(Dispatchers.IO) {
      try {
        addPlaylistItems(session.serverUrl, session.token, playlistId, songIds)
        // Reload playlist detail to reflect changes
        loadPlaylistDetail(playlistId, "")
        // Also reload playlists to update child count
        ensureLoaded(force = true)
      } catch (error: Exception) {
        android.util.Log.e("PlaylistViewModel", "Failed to add songs to playlist", error)
        mutableState.update { it.copy(error = "Failed to add songs to playlist") }
      }
    }
  }

  fun playPlaylist(startIndex: Int = 0) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val songs = mutableDetailState.value.songs

    if (songs.isEmpty()) return

    playerController.setQueue(songs, serverUrl, token, startIndex)
  }

  fun shufflePlaylist() {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val songs = mutableDetailState.value.songs.shuffled()

    if (songs.isEmpty()) return

    playerController.setQueue(songs, serverUrl, token)
  }

  fun clearError() {
    mutableState.update { it.copy(error = null) }
    mutableDetailState.update { it.copy(error = null) }
  }
}

private const val LOAD_FRESHNESS_MS = 60_000L
