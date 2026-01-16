package com.aurelia.app.player

import android.content.Context
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer

class PlayerController(context: Context) {
  private val player = ExoPlayer.Builder(context).build()

  fun setOnPlaybackEnded(onEnded: () -> Unit) {
    player.addListener(object : Player.Listener {
      override fun onPlaybackStateChanged(playbackState: Int) {
        if (playbackState == Player.STATE_ENDED) {
          onEnded()
        }
      }
    })
  }

  fun play(url: String) {
    val mediaItem = MediaItem.fromUri(url)
    player.setMediaItem(mediaItem)
    player.prepare()
    player.playWhenReady = true
  }

  fun stop() {
    player.stop()
  }

  fun release() {
    player.release()
  }
}
