package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.storage.SessionStore

class SettingsViewModelFactory(
  private val sessionStore: SessionStore,
) : ViewModelProvider.Factory {
  override fun <T : ViewModel> create(modelClass: Class<T>): T {
    if (modelClass.isAssignableFrom(SettingsViewModel::class.java)) {
      @Suppress("UNCHECKED_CAST")
      return SettingsViewModel(sessionStore) as T
    }
    throw IllegalArgumentException("Unknown ViewModel class")
  }
}
