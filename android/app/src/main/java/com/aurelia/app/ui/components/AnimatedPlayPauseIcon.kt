package com.aurelia.app.ui.components

import androidx.compose.animation.graphics.ExperimentalAnimationGraphicsApi
import androidx.compose.animation.graphics.res.animatedVectorResource
import androidx.compose.animation.graphics.res.rememberAnimatedVectorPainter
import androidx.compose.animation.graphics.vector.AnimatedImageVector
import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import com.aurelia.app.R

/**
 * A play/pause icon that morphs between states with animation,
 * similar to YouTube's play/pause button.
 */
@OptIn(ExperimentalAnimationGraphicsApi::class)
@Composable
fun AnimatedPlayPauseIcon(
  isPlaying: Boolean,
  modifier: Modifier = Modifier,
  tint: Color = Color.White,
  contentDescription: String? = if (isPlaying) "Pause" else "Play"
) {
  val animatedVector = AnimatedImageVector.animatedVectorResource(R.drawable.avd_play_to_pause)

  Image(
    painter = rememberAnimatedVectorPainter(animatedVector, atEnd = isPlaying),
    contentDescription = contentDescription,
    modifier = modifier,
    colorFilter = ColorFilter.tint(tint)
  )
}
