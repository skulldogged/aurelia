package com.aurelia.app.ui

import uniffi.aurelia_core.BackendProvider

enum class LoginProviderSelection {
  AUTO,
  JELLYFIN,
  NAVIDROME,
}

data class LoginState(
  val serverUrl: String = "",
  val username: String = "",
  val password: String = "",
  val providerSelection: LoginProviderSelection = LoginProviderSelection.AUTO,
  val detectedProvider: BackendProvider? = null,
  val isDetectingProvider: Boolean = false,
  val isSubmitting: Boolean = false,
  val error: String? = null,
  val token: String? = null,
  val userId: String? = null,
  val useDynamicColor: Boolean = true,
)
