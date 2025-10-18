package dev.pupbrained.aurelia

import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : TauriActivity() {
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
