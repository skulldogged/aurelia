package com.aurelia.app.ui

data class AppState(
  val isLoading: Boolean = true,
  val isLoggedIn: Boolean = false,
  val sessionVersion: Int = 0,
)
