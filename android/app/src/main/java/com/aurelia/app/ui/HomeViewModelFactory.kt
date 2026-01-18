package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore

class HomeViewModelFactory(
  private val sessionStore: SessionStore,
  private val playerController: PlayerController
) : ViewModelProvider.Factory {
  @Suppress("UNCHECKED_CAST")
  override fun <T : ViewModel> create(modelClass: Class<T>): T {
    if (modelClass.isAssignableFrom(HomeViewModel::class.java))
      return HomeViewModel(sessionStore, playerController) as T

    throw IllegalArgumentException("Unknown ViewModel class")
  }
}
