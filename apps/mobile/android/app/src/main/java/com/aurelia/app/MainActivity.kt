package com.aurelia.app

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
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
import androidx.core.content.ContextCompat
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlaybackService
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.AppViewModel
import com.aurelia.app.ui.AppViewModelFactory
import com.aurelia.app.ui.LoginScreen
import com.aurelia.app.ui.MainScreen
import com.aurelia.app.ui.theme.AureliaTheme

class MainActivity : ComponentActivity() {
    private val notificationPermissionLauncher =
        registerForActivityResult(
            ActivityResultContracts.RequestPermission(),
        ) { isGranted ->
            // Permission granted, notifications will work when playback starts
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        checkNotificationPermission()
        setContent {
            AureliaApp()
        }
    }

    private fun checkNotificationPermission() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val permissionStatus = checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
            if (permissionStatus != PackageManager.PERMISSION_GRANTED) {
                notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
            }
        }
    }
}

@Composable
private fun AureliaApp() {
    val context = LocalContext.current
    val sessionStore = remember { SessionStore(context) }

    if (sessionStore.getAppDataDir().isNullOrBlank()) {
        sessionStore.setAppDataDir(context.filesDir.absolutePath)
    }

    // Remember PlayerController and release it when the composable leaves composition
    val playerController = remember { PlayerController(context) }
    DisposableEffect(Unit) {
        onDispose {
            playerController.release()
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
