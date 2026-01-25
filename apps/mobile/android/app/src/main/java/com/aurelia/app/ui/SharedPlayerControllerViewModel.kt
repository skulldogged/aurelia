package com.aurelia.app.ui

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
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

    if (!appDataDir.isNullOrBlank() && !serverUrl.isNullOrBlank() && !token.isNullOrBlank())
      playerController.restoreState(appDataDir) { songId, container ->
        buildStreamUrl(serverUrl, token, songId, container)
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
