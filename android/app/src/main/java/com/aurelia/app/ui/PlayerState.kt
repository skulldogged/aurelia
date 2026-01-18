package com.aurelia.app.ui

import com.aurelia.app.data.model.Lyrics
import com.aurelia.app.player.QueueItem

data class PlayerState(
  val title: String = "",
  val artist: String = "",
  val albumArtUrl: String? = null,
  val isPlaying: Boolean = false,
  val positionMs: Long = 0L,
  val durationMs: Long = 0L,
  val queue: List<QueueItem> = emptyList(),
  val currentQueueIndex: Int = -1,
  val lyrics: Lyrics? = null,
  val showLyrics: Boolean = false,
) {
  val hasPrevious: Boolean get() = currentQueueIndex > 0
  val hasNext: Boolean get() = currentQueueIndex >= 0 && currentQueueIndex < queue.size - 1
}
