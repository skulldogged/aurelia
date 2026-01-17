package com.aurelia.app.ui

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
)
