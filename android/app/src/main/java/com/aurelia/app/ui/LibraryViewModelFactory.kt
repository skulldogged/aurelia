package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore

class LibraryViewModelFactory(
    private val sessionStore: SessionStore,
    private val playerController: PlayerController,
) : ViewModelProvider.Factory {
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(LibraryViewModel::class.java)) {
            @Suppress("UNCHECKED_CAST")
            return LibraryViewModel(sessionStore, playerController) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class")
    }
}
