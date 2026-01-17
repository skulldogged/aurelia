package com.aurelia.app.player

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import androidx.core.app.NotificationCompat
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.MediaStyleNotificationHelper
import com.aurelia.app.MainActivity
import com.aurelia.app.R

class PlaybackService : MediaSessionService() {
  private var mediaSession: MediaSession? = null
  private lateinit var notificationManager: NotificationManager

  override fun onCreate() {
    super.onCreate()
    notificationManager = getSystemService(NotificationManager::class.java)
    ensureNotificationChannel()
    
    // Start foreground immediately to avoid ANR
    // Android requires startForeground() within 5 seconds of startForegroundService()
    val initialNotification = buildInitialNotification()
    startForeground(PlaybackNotificationIds.service, initialNotification)
    
    val player = ExoPlayer.Builder(this).build()
    mediaSession = MediaSession.Builder(this, player).build()
  }

  override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? {
    return mediaSession
  }

  override fun onTaskRemoved(rootIntent: Intent?) {
    val player = mediaSession?.player
    if (player == null || !player.playWhenReady) {
      stopSelf()
    }
  }

  override fun onDestroy() {
    mediaSession?.run {
      player.release()
      release()
    }
    mediaSession = null
    super.onDestroy()
  }

  override fun onUpdateNotification(session: MediaSession, startInForegroundRequired: Boolean) {
    val notification = buildNotification(session)
    if (startInForegroundRequired) {
      startForeground(PlaybackNotificationIds.service, notification)
    } else {
      notificationManager.notify(PlaybackNotificationIds.service, notification)
    }
  }

  private fun buildNotification(session: MediaSession): Notification {
    val intent = Intent(this, MainActivity::class.java)
    val contentIntent = PendingIntent.getActivity(
      this,
      0,
      intent,
      PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
    )

    val metadata = session.player.mediaMetadata
    val title = metadata.title?.toString() ?: getString(R.string.playback_notification_title)
    val artist = metadata.artist?.toString() ?: getString(R.string.playback_notification_artist)

    return NotificationCompat.Builder(this, PlaybackNotificationIds.channel)
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
    val channel = NotificationChannel(
      PlaybackNotificationIds.channel,
      getString(R.string.playback_notification_channel_name),
      NotificationManager.IMPORTANCE_LOW
    )
    notificationManager.createNotificationChannel(channel)
  }

  private fun buildInitialNotification(): Notification {
    val intent = Intent(this, MainActivity::class.java)
    val contentIntent = PendingIntent.getActivity(
      this,
      0,
      intent,
      PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
    )

    return NotificationCompat.Builder(this, PlaybackNotificationIds.channel)
      .setContentTitle(getString(R.string.playback_notification_title))
      .setContentText(getString(R.string.playback_notification_artist))
      .setSmallIcon(R.drawable.ic_stat_music_note)
      .setContentIntent(contentIntent)
      .setOngoing(false)
      .setOnlyAlertOnce(true)
      .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
      .build()
  }
}

private object PlaybackNotificationIds {
  const val channel = "aurelia_playback"
  const val service = 2001
}
