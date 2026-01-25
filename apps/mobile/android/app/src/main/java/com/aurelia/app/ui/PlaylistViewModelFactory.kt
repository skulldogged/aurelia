package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore

class PlaylistViewModelFactory(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController,
) : ViewModelProvider.Factory {
  override fun <T : ViewModel> create(modelClass: Class<T>): T {
    if (modelClass.isAssignableFrom(PlaylistViewModel::class.java)) {
      @Suppress("UNCHECKED_CAST")
      return PlaylistViewModel(sessionStore, playerController) as T
    }
    throw IllegalArgumentException("Unknown ViewModel class")
  }
}
