package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.player.PlayerController

class PlayerViewModelFactory(
  private val playerController: PlayerController,
) : ViewModelProvider.Factory {
  override fun <T : ViewModel> create(modelClass: Class<T>): T {
    if (modelClass.isAssignableFrom(PlayerViewModel::class.java)) {
      @Suppress("UNCHECKED_CAST")
      return PlayerViewModel(playerController) as T
    }
    throw IllegalArgumentException("Unknown ViewModel class")
  }
}
