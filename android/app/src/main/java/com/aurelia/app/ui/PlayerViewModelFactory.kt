package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore

class PlayerViewModelFactory(
    private val playerController: PlayerController,
    private val sessionStore: SessionStore,
) : ViewModelProvider.Factory {
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(PlayerViewModel::class.java)) {
            @Suppress("UNCHECKED_CAST")
            return PlayerViewModel(playerController, sessionStore) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class")
    }
}
