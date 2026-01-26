package com.aurelia.app.ui

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.launch
import uniffi.aurelia_core.buildStreamUrl

class SharedPlayerControllerViewModel(application: Application) : AndroidViewModel(application) {
  val playerController = PlayerController(application)
  private var isInitialized = false

  fun initialize(sessionStore: SessionStore) {
    if (isInitialized) return
    isInitialized = true

    val appDataDir = sessionStore.getAppDataDir()
    val serverUrl = sessionStore.getServerUrl()
    val token = sessionStore.getToken()

    if (!appDataDir.isNullOrBlank() && !serverUrl.isNullOrBlank() && !token.isNullOrBlank()) {
      // Wait for controller to connect before restoring state
      viewModelScope.launch {
        playerController.awaitConnection()
        playerController.restoreState(appDataDir) { songId, container ->
          buildStreamUrl(serverUrl, token, songId, container)
        }
      }
    }
  }

  fun saveState(sessionStore: SessionStore) {
    sessionStore.getAppDataDir()?.let { appDataDir ->
      playerController.saveState(appDataDir)
    }
  }

  override fun onCleared() {
    super.onCleared()
    playerController.release()
  }
}
