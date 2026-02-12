package com.aurelia.app.player

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import androidx.annotation.OptIn
import androidx.core.app.NotificationCompat
import androidx.media3.common.ForwardingPlayer
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.MediaStyleNotificationHelper
import androidx.media3.session.SessionCommand
import androidx.media3.session.SessionResult
import com.aurelia.app.MainActivity
import com.aurelia.app.R
import com.aurelia.app.audio.AudioManager
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture

class PlaybackService : MediaSessionService() {
  private var mediaSession: MediaSession? = null
  private lateinit var notificationManager: NotificationManager

  override fun onCreate() {
    super.onCreate()
    notificationManager = getSystemService(NotificationManager::class.java)
    ensureNotificationChannel()

    val exoPlayer = ExoPlayer.Builder(this).build()

    val sessionStore = com.aurelia.app.storage.SessionStore(this)

    // Initialize audio effects when audio session ID becomes available
    exoPlayer.addListener(object : Player.Listener {
      override fun onAudioSessionIdChanged(audioSessionId: Int) {
        if (audioSessionId != 0) {
          AudioManager.initialize(this@PlaybackService, audioSessionId, sessionStore)
        }
      }

      override fun onIsPlayingChanged(isPlaying: Boolean) {
        if (isPlaying) {
          AudioManager.onPlaybackStarted(sessionStore)
        } else {
          AudioManager.onPlaybackStopped(sessionStore)
        }
      }
    })

    // Wrap in ForwardingPlayer that always reports seek as available.
    // ExoPlayer removes COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM for non-seekable streams
    // (e.g. transcoded ALAC via /universal), but we handle seeking by rebuilding
    // the stream URL with startTimeTicks in PlayerController.
    @OptIn(UnstableApi::class)
    val player = object : ForwardingPlayer(exoPlayer) {
      override fun getAvailableCommands(): Player.Commands {
        return super.getAvailableCommands().buildUpon()
          .add(Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM)
          .add(Player.COMMAND_SEEK_TO_MEDIA_ITEM)
          .build()
      }

      override fun isCommandAvailable(command: Int): Boolean {
        if (command == Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM ||
            command == Player.COMMAND_SEEK_TO_MEDIA_ITEM) {
          return true
        }
        return super.isCommandAvailable(command)
      }
    }

    mediaSession =
      MediaSession
        .Builder(this, player)
        .setCallback(
          object : MediaSession.Callback {
            override fun onConnect(
              session: MediaSession,
              controller: MediaSession.ControllerInfo,
            ): MediaSession.ConnectionResult {
              return MediaSession.ConnectionResult.AcceptedResultBuilder(session)
                .setAvailablePlayerCommands(Player.Commands.Builder().addAllCommands().build())
                .setAvailableSessionCommands(MediaSession.ConnectionResult.DEFAULT_SESSION_COMMANDS)
                .build()
            }

            override fun onCustomCommand(
              session: MediaSession,
              controller: MediaSession.ControllerInfo,
              customCommand: SessionCommand,
              args: android.os.Bundle,
            ): ListenableFuture<SessionResult> {
              return Futures.immediateFuture(SessionResult(SessionResult.RESULT_ERROR_NOT_SUPPORTED))
            }
          },
        )
        .build()
  }

  override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = mediaSession

  override fun onTaskRemoved(rootIntent: Intent?) {
    val player = mediaSession?.player
    // Only stop if not playing AND no media loaded
    // This keeps the service alive when paused with a queue
    if (player == null || (!player.playWhenReady && player.mediaItemCount == 0)) {
      stopSelf()
    }
  }

  override fun onDestroy() {
    AudioManager.release()
    mediaSession?.run {
      player.release()
      release()
    }
    mediaSession = null
    super.onDestroy()
  }

  override fun onUpdateNotification(
    session: MediaSession,
    startInForegroundRequired: Boolean,
  ) {
    val notification = buildNotification(session)
    if (startInForegroundRequired) {
      startForeground(PlaybackNotificationIds.SERVICE, notification)
    } else {
      notificationManager.notify(PlaybackNotificationIds.SERVICE, notification)
    }
  }

  @OptIn(UnstableApi::class)
  private fun buildNotification(session: MediaSession): Notification {
    val intent = Intent(this, MainActivity::class.java)
    val contentIntent =
      PendingIntent.getActivity(
        this,
        0,
        intent,
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )

    val metadata = session.player.mediaMetadata
    val title = metadata.title?.toString() ?: getString(R.string.playback_notification_title)
    val artist = metadata.artist?.toString() ?: getString(R.string.playback_notification_artist)

    return NotificationCompat
      .Builder(this, PlaybackNotificationIds.CHANNEL)
      .setContentTitle(title)
      .setContentText(artist)
      .setSmallIcon(R.drawable.ic_stat_music_note)
      .setContentIntent(contentIntent)
      .setOngoing(session.player.playWhenReady)
      .setOnlyAlertOnce(true)
      .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
      .setStyle(MediaStyleNotificationHelper.MediaStyle(session))
      .build()
  }

  private fun ensureNotificationChannel() {
    val channel =
      NotificationChannel(
        PlaybackNotificationIds.CHANNEL,
        getString(R.string.playback_notification_channel_name),
        NotificationManager.IMPORTANCE_LOW,
      )
    notificationManager.createNotificationChannel(channel)
  }
}

private object PlaybackNotificationIds {
  const val CHANNEL = "aurelia_playback"
  const val SERVICE = 2001
}
