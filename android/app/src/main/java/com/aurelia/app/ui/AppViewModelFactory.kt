package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.aurelia.app.storage.SessionStore

class AppViewModelFactory(private val sessionStore: SessionStore) : ViewModelProvider.Factory {
  override fun <T : ViewModel> create(modelClass: Class<T>): T {
    if (modelClass.isAssignableFrom(AppViewModel::class.java)) {
      @Suppress("UNCHECKED_CAST")
      return AppViewModel(sessionStore) as T
    }
    throw IllegalArgumentException("Unknown ViewModel class")
  }
}
