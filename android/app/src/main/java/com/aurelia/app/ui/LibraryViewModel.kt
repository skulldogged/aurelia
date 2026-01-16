package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.buildStreamUrl
import uniffi.aurelia_core.fetchSongs

class LibraryViewModel(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController,
) : ViewModel() {
  private val mutableState = MutableStateFlow(LibraryState())
  val state: StateFlow<LibraryState> = mutableState

  fun loadLibrary() {
    val serverUrl = sessionStore.getServerUrl()
    val userId = sessionStore.getUserId()
    val token = sessionStore.getToken()

    if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
      mutableState.update { it.copy(error = "Missing session data") }
      return
    }

    mutableState.update { it.copy(isLoading = true, error = null) }

    viewModelScope.launch(Dispatchers.IO) {
      try {
        val songs = fetchSongs(serverUrl, token, userId)
        mutableState.update { it.copy(isLoading = false, songs = songs) }
      } catch (error: AppException) {
        mutableState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load") }
      } catch (_: Exception) {
        mutableState.update { it.copy(isLoading = false, error = "Failed to load") }
      }
    }
  }

  fun play(songId: String, container: String?) {
    val serverUrl = sessionStore.getServerUrl() ?: return
    val token = sessionStore.getToken() ?: return
    val url = buildStreamUrl(serverUrl, token, songId, container)
    playerController.play(url)
  }

  override fun onCleared() {
    super.onCleared()
    playerController.release()
  }
}
