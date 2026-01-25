package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class AppViewModel(
  private val sessionStore: SessionStore,
) : ViewModel() {
  private val mutableState = MutableStateFlow(AppState())
  val state: StateFlow<AppState> = mutableState

  fun checkSession() {
    val hasSession =
      !sessionStore.getServerUrl().isNullOrBlank() &&
        !sessionStore.getUserId().isNullOrBlank() &&
        !sessionStore.getToken().isNullOrBlank()
    mutableState.value = AppState(isLoggedIn = hasSession)
  }
}
