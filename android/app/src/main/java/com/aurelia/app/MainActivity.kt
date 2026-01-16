package com.aurelia.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.AppViewModel
import com.aurelia.app.ui.AppViewModelFactory
import com.aurelia.app.ui.LibraryScreen
import com.aurelia.app.ui.LoginScreen
import androidx.lifecycle.viewmodel.compose.viewModel

class MainActivity : ComponentActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    setContent {
      AureliaApp()
    }
  }
}

@Composable
private fun AureliaApp() {
  val sessionStore = SessionStore(context = androidx.compose.ui.platform.LocalContext.current)
  val playerController = PlayerController(context = androidx.compose.ui.platform.LocalContext.current)
  val appViewModel: AppViewModel = viewModel(
    factory = AppViewModelFactory(sessionStore)
  )
  val appState by appViewModel.state.collectAsState()

  LaunchedEffect(Unit) {
    appViewModel.checkSession()
  }

  MaterialTheme {
    if (appState.isLoggedIn) {
      LibraryScreen(sessionStore, playerController)
    } else {
      LoginScreen(sessionStore)
    }
  }
}

