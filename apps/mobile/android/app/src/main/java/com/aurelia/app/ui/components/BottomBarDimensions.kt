package com.aurelia.app.ui.components

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * Shared dimensions for the floating bottom bars (mini player + nav bar).
 * Use these to calculate consistent bottom padding across all screens.
 */
object BottomBarDimensions {
  val MiniPlayerHeight = 64.dp
  val NavBarContentHeight = 90.dp
  val NavBarVerticalPadding = 16.dp // top + bottom padding around nav bar
  val MiniPlayerTopPadding = 8.dp
  val NavBarSpacing = 4.dp // spacing between player and nav when both visible

  /**
   * Calculate the total bottom padding needed for scrollable content.
   * This accounts for: nav bar + optional mini player + system nav bar.
   */
  @Composable
  fun calculateBottomPadding(hasPlayerBar: Boolean): Dp {
    val systemNavBarHeight = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()

    val navBarTotal = NavBarContentHeight + NavBarVerticalPadding + 12.dp // extra buffer
    val playerBarTotal =
      if (hasPlayerBar) {
        MiniPlayerHeight + MiniPlayerTopPadding + NavBarSpacing
      } else {
        0.dp
      }

    return navBarTotal + playerBarTotal + systemNavBarHeight
  }
}
