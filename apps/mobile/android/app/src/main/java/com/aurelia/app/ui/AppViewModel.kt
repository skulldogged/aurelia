package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class AppViewModel(
  private val sessionStore: SessionStore,
) : ViewModel() {
  private val mutableState = MutableStateFlow(checkSessionState())
  val state: StateFlow<AppState> = mutableState

  private fun checkSessionState(sessionVersion: Int = mutableState.value.sessionVersion): AppState {
    val hasSession =
      !sessionStore.getServerUrl().isNullOrBlank() &&
        !sessionStore.getUserId().isNullOrBlank() &&
        !sessionStore.getToken().isNullOrBlank()
    return AppState(isLoading = false, isLoggedIn = hasSession, sessionVersion = sessionVersion)
  }

  fun checkSession() {
    mutableState.value = checkSessionState(mutableState.value.sessionVersion)
  }

  fun refreshForSessionSwitch() {
    val nextVersion = mutableState.value.sessionVersion + 1
    mutableState.value = checkSessionState(nextVersion)
  }
}
