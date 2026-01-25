package com.aurelia.app

import android.Manifest
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.AppViewModel
import com.aurelia.app.ui.AppViewModelFactory
import com.aurelia.app.ui.LoginScreen
import com.aurelia.app.ui.MainScreen
import com.aurelia.app.ui.SharedPlayerControllerViewModel
import com.aurelia.app.ui.theme.AureliaTheme

class MainActivity : ComponentActivity() {
  private val notificationPermissionLauncher =
    registerForActivityResult(
      ActivityResultContracts.RequestPermission(),
    ) { _ ->
      // Permission granted, notifications will work when playback starts
    }

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    enableEdgeToEdge()
    checkNotificationPermission()
    setContent { AureliaApp() }
  }

  private fun checkNotificationPermission() {
    val permissionStatus = checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
    if (permissionStatus != PackageManager.PERMISSION_GRANTED)
      notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
  }
}

@Composable
private fun AureliaApp() {
  val context = LocalContext.current
  val sessionStore = remember { SessionStore(context) }

  if (sessionStore.getAppDataDir().isNullOrBlank())
    sessionStore.setAppDataDir(context.filesDir.absolutePath)

  // Use SharedPlayerControllerViewModel to ensure PlayerController survives configuration changes
  val sharedPlayerViewModel: SharedPlayerControllerViewModel = viewModel()
  val playerController = sharedPlayerViewModel.playerController

  // Initialize player state (restores state if needed)
  LaunchedEffect(sharedPlayerViewModel) {
    sharedPlayerViewModel.initialize(sessionStore)
  }

  // Save player state when app goes to background
  val lifecycleOwner = LocalLifecycleOwner.current
  DisposableEffect(lifecycleOwner) {
    val observer = LifecycleEventObserver { _, event ->
      if (event == Lifecycle.Event.ON_STOP)
        sharedPlayerViewModel.saveState(sessionStore)
    }
    lifecycleOwner.lifecycle.addObserver(observer)
    onDispose {
      lifecycleOwner.lifecycle.removeObserver(observer)
    }
  }

  val appViewModel: AppViewModel =
    viewModel(
      factory = AppViewModelFactory(sessionStore),
    )
  val appState by appViewModel.state.collectAsState()

  // Track dynamic color preference changes
  var useDynamicColor by remember { mutableStateOf(sessionStore.getUseDynamicColor()) }

  // Set up auth interceptor for automatic logout on 401 errors
  DisposableEffect(Unit) {
    AuthInterceptor.setLogoutCallback {
      sessionStore.clear()
      appViewModel.checkSession()
    }
    onDispose {
      AuthInterceptor.clearLogoutCallback()
    }
  }

  LaunchedEffect(Unit) {
    appViewModel.checkSession()
  }

  AureliaTheme(useDynamicColor = useDynamicColor) {
    if (appState.isLoggedIn) {
      MainScreen(
        sessionStore = sessionStore,
        playerController = playerController,
        onLogout = {
          sessionStore.clear()
          appViewModel.checkSession()
        },
      )
    } else {
      LoginScreen(sessionStore) { appViewModel.checkSession() }
    }
  }
}
