package com.aurelia.app.ui.components

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import org.junit.Rule
import org.junit.Test

class AnimatedPlayPauseIconTest {
  @get:Rule
  val composeTestRule = createComposeRule()

  @Test
  fun showsPlayLabelWhenPaused() {
    composeTestRule.setContent {
      AnimatedPlayPauseIcon(isPlaying = false)
    }

    composeTestRule.onNodeWithContentDescription("Play").assertExists()
  }

  @Test
  fun showsPauseLabelWhenPlaying() {
    composeTestRule.setContent {
      AnimatedPlayPauseIcon(isPlaying = true)
    }

    composeTestRule.onNodeWithContentDescription("Pause").assertExists()
  }
}
