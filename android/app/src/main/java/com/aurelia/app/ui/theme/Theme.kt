package com.aurelia.app.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

private val DarkColors = darkColorScheme(
  primary = AureliaPurplePrimary,
  onPrimary = Color.White,
  primaryContainer = AureliaPrimaryContainer,
  onPrimaryContainer = AureliaOnPrimaryContainer,
  secondary = AureliaPink,
  onSecondary = Color.White,
  secondaryContainer = AureliaPink.copy(alpha = 0.2f),
  onSecondaryContainer = AureliaPink,
  tertiary = AureliaOrange,
  onTertiary = Color.Black,
  tertiaryContainer = AureliaOrange.copy(alpha = 0.2f),
  onTertiaryContainer = AureliaOrange,
  background = AureliaPurpleDark,
  onBackground = AureliaOnSurface,
  surface = AureliaSurface,
  onSurface = AureliaOnSurface,
  surfaceVariant = AureliaSurfaceVariant,
  onSurfaceVariant = AureliaOnSurfaceMuted,
  surfaceContainerLowest = AureliaPurpleDark,
  surfaceContainerLow = AureliaSurfaceContainerLow,
  surfaceContainer = AureliaSurface,
  surfaceContainerHigh = AureliaSurfaceContainerHigh,
  surfaceContainerHighest = AureliaSurfaceVariant,
  outline = AureliaOutline,
  outlineVariant = AureliaOutline.copy(alpha = 0.5f),
  error = Color(0xFFFF6B6B),
  onError = Color.White
)

private val LightColors = lightColorScheme(
  primary = AureliaLightPrimary,
  onPrimary = Color.White,
  primaryContainer = AureliaLightPrimaryContainer,
  onPrimaryContainer = AureliaLightOnPrimaryContainer,
  secondary = AureliaPink,
  onSecondary = Color.White,
  tertiary = AureliaOrange,
  onTertiary = Color.Black,
  background = AureliaLightBackground,
  onBackground = AureliaLightOnSurface,
  surface = AureliaLightSurface,
  onSurface = AureliaLightOnSurface,
  surfaceVariant = AureliaLightSurfaceVariant,
  onSurfaceVariant = AureliaLightOnSurfaceMuted,
  surfaceContainerLowest = AureliaLightBackground,
  surfaceContainerLow = AureliaLightSurfaceContainerLow,
  surfaceContainer = AureliaLightSurface,
  surfaceContainerHigh = AureliaLightSurfaceContainerHigh,
  surfaceContainerHighest = AureliaLightSurfaceVariant,
  outline = AureliaLightOutline,
  outlineVariant = AureliaLightOutline.copy(alpha = 0.6f),
  error = Color(0xFFD32F2F),
  onError = Color.White
)

@Composable
fun AureliaTheme(
  darkTheme: Boolean = isSystemInDarkTheme(),
  useDynamicColor: Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.S,
  content: @Composable () -> Unit,
) {
  val context = LocalContext.current
  val colorScheme = when {
    useDynamicColor && darkTheme -> dynamicDarkColorScheme(context)
    useDynamicColor -> dynamicLightColorScheme(context)
    darkTheme -> DarkColors
    else -> LightColors
  }

  val view = LocalView.current
  if (!view.isInEditMode) {
    SideEffect {
      val window = (view.context as Activity).window
      // Let enableEdgeToEdge handle the system bar colors (transparent)
      // Just set the icon appearance based on theme
      val insetsController = WindowCompat.getInsetsController(window, view)
      insetsController.isAppearanceLightStatusBars = !darkTheme
      insetsController.isAppearanceLightNavigationBars = !darkTheme
    }
  }

  MaterialTheme(
    colorScheme = colorScheme,
    typography = AureliaTypography,
    shapes = AureliaShapes,
    content = content
  )
}
