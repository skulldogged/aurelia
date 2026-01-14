package dev.pupbrained.aurelia

import android.content.res.Configuration
import android.os.Bundle
import android.util.Log
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import dev.pupbrained.aurelia.plugin.audio.AudioPlayerPlugin

class MainActivity : TauriActivity() {
  companion object {
    private const val TAG = "MainActivity"
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    // Allow content to extend under the system bars
    WindowCompat.setDecorFitsSystemWindows(window, false)

    // Make status bar and navigation bar transparent
    window.statusBarColor = android.graphics.Color.TRANSPARENT
    window.navigationBarColor = android.graphics.Color.TRANSPARENT

    // Set initial immersive mode based on current orientation
    updateImmersiveMode(resources.configuration.orientation)
  }

  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
    updateImmersiveMode(newConfig.orientation)
  }

  override fun onRequestPermissionsResult(
    requestCode: Int,
    permissions: Array<out String>,
    grantResults: IntArray
  ) {
    Log.d(TAG, "onRequestPermissionsResult: requestCode=$requestCode, permissions=${permissions.contentToString()}, grantResults=${grantResults.contentToString()}")
    super.onRequestPermissionsResult(requestCode, permissions, grantResults)
    // Forward to AudioPlayerPlugin for visualizer permission handling
    AudioPlayerPlugin.handlePermissionResult(requestCode, permissions, grantResults)
  }

  private fun updateImmersiveMode(orientation: Int) {
    val controller = WindowCompat.getInsetsController(window, window.decorView)

    if (orientation == Configuration.ORIENTATION_LANDSCAPE) {
      // Enable immersive mode in landscape (hide system bars)
      controller?.hide(WindowInsetsCompat.Type.systemBars())
      controller?.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    } else {
      // Show system bars in portrait
      controller?.show(WindowInsetsCompat.Type.systemBars())
    }
  }
}
