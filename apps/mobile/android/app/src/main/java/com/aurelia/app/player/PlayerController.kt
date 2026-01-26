package com.aurelia.app.player

import android.content.ComponentName
import android.content.Context
import android.os.Bundle
import android.os.SystemClock
import android.util.Log
import androidx.core.net.toUri
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.Player
import androidx.media3.session.MediaController
import androidx.media3.session.SessionToken
import com.google.common.util.concurrent.ListenableFuture
import com.google.common.util.concurrent.MoreExecutors
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.aurelia_core.PlayerStateData
import uniffi.aurelia_core.QueueItemData
import uniffi.aurelia_core.loadPlayerState
import uniffi.aurelia_core.savePlayerState

data class QueueItem(
  val id: String,
  val uri: String,
  val title: String,
  val artist: String,
  val albumArtUrl: String?,
  val durationMs: Long = 0L,
  val isFavorite: Boolean = false,
  val albumId: String? = null,
  val artistId: String? = null,
  val albumName: String? = null,
)

class PlayerController(
  private val context: Context,
) {
  private val sessionToken = SessionToken(context, ComponentName(context, PlaybackService::class.java))
  private var controllerFuture: ListenableFuture<MediaController>? = null
  private var mediaController: MediaController? = null
  private var playbackEndedCallback: (() -> Unit)? = null
  private val durationByMediaId: MutableMap<String, Long> = mutableMapOf()
  private val artistIdByMediaId: MutableMap<String, String> = mutableMapOf()
  private val albumIdByMediaId: MutableMap<String, String> = mutableMapOf()
  private val albumNameByMediaId: MutableMap<String, String> = mutableMapOf()

  // Connection state tracking
  private val _isConnected = MutableStateFlow(false)
  val isConnected: StateFlow<Boolean> = _isConnected.asStateFlow()

  // Pending actions to run once connected
  private val pendingActions = mutableListOf<(MediaController) -> Unit>()

  // Track if we're in the process of connecting
  private var isConnecting = false

  init {
    connectToService()
  }

  private fun connectToService() {
    synchronized(this) {
      if (isConnecting) return
      isConnecting = true
    }

    Log.d(TAG, "Connecting to PlaybackService...")
    val future = MediaController.Builder(context, sessionToken).buildAsync()
    controllerFuture = future

    future.addListener(
      {
        try {
          val controller = future.get()
          mediaController = controller

          // Add listener to track connection state changes
          controller.addListener(object : Player.Listener {
            override fun onEvents(player: Player, events: Player.Events) {
              val wasConnected = _isConnected.value
              val nowConnected = controller.isConnected

              if (nowConnected && !wasConnected) {
                onControllerConnected(controller)
              } else if (!nowConnected && wasConnected) {
                Log.d(TAG, "MediaController disconnected - will reconnect")
                _isConnected.value = false
                isConnecting = false
                // Reconnect after a short delay
                CoroutineScope(Dispatchers.Main).launch {
                  delay(100)
                  connectToService()
                }
              }
            }
          })

          // Check if already connected
          if (controller.isConnected) {
            onControllerConnected(controller)
          }

          playbackEndedCallback?.let { registerPlaybackEndedListener(it) }
        } catch (e: Exception) {
          Log.e(TAG, "Failed to connect to PlaybackService", e)
          isConnecting = false
        }
      },
      MoreExecutors.directExecutor(),
    )
  }

  private fun onControllerConnected(controller: MediaController) {
    Log.d(TAG, "MediaController connected")
    _isConnected.value = true
    isConnecting = false
    // Execute any pending actions
    synchronized(pendingActions) {
      pendingActions.forEach { action ->
        try {
          action(controller)
        } catch (e: Exception) {
          Log.e(TAG, "Error executing pending action", e)
        }
      }
      pendingActions.clear()
    }
  }

  /**
   * Suspends until the MediaController is connected to the session.
   */
  suspend fun awaitConnection() {
    isConnected.first { it }
  }

  fun setQueue(
    items: List<QueueItem>,
    startIndex: Int = 0,
  ) {
    durationByMediaId.clear()
    artistIdByMediaId.clear()
    albumIdByMediaId.clear()
    albumNameByMediaId.clear()

    items.forEach { item ->
      if (item.durationMs > 0L) {
        durationByMediaId[item.id] = item.durationMs
      }
      item.artistId?.let { artistIdByMediaId[item.id] = it }
      item.albumId?.let { albumIdByMediaId[item.id] = it }
      item.albumName?.let { albumNameByMediaId[item.id] = it }
    }

    withController { controller ->
      val mediaItems =
        items.map { item ->
          val extras = Bundle().apply {
            item.albumId?.let { putString(EXTRA_ALBUM_ID, it) }
            item.artistId?.let { putString(EXTRA_ARTIST_ID, it) }
          }
          val metadataBuilder =
            MediaMetadata
              .Builder()
              .setTitle(item.title)
              .setArtist(item.artist)
              .setExtras(extras)
          item.albumArtUrl?.let { metadataBuilder.setArtworkUri(it.toUri()) }

          MediaItem
            .Builder()
            .setMediaId(item.id)
            .setUri(item.uri)
            .setMediaMetadata(metadataBuilder.build())
            .build()
        }
      controller.setMediaItems(mediaItems, startIndex, 0L)
      controller.prepare()
      controller.playWhenReady = true
    }
  }

  fun getQueue(): List<QueueItem> {
    val controller = mediaController ?: return emptyList()
    val items = mutableListOf<QueueItem>()
    for (i in 0 until controller.mediaItemCount) {
      val mediaItem = controller.getMediaItemAt(i)
      val extras = mediaItem.mediaMetadata.extras
      items.add(
        QueueItem(
          id = mediaItem.mediaId,
          uri = mediaItem.localConfiguration?.uri?.toString() ?: "",
          title = mediaItem.mediaMetadata.title?.toString() ?: "",
          artist = mediaItem.mediaMetadata.artist?.toString() ?: "",
          albumArtUrl = mediaItem.mediaMetadata.artworkUri?.toString(),
          durationMs = durationByMediaId[mediaItem.mediaId] ?: 0L,
          albumId = albumIdByMediaId[mediaItem.mediaId] ?: extras?.getString(EXTRA_ALBUM_ID),
          artistId = artistIdByMediaId[mediaItem.mediaId] ?: extras?.getString(EXTRA_ARTIST_ID),
          albumName = albumNameByMediaId[mediaItem.mediaId] ?: extras?.getString(EXTRA_ALBUM_NAME),
        ),
      )
    }
    return items
  }

  fun getCurrentQueueIndex(): Int = mediaController?.currentMediaItemIndex ?: -1

  fun playQueueItem(index: Int) {
    withController { controller ->
      if (index >= 0 && index < controller.mediaItemCount) {
        controller.seekToDefaultPosition(index)
        controller.playWhenReady = true
        if (controller.playbackState == Player.STATE_IDLE) {
          controller.prepare()
        }
      }
    }
  }

  fun addToQueue(item: QueueItem) {
    if (item.durationMs > 0L) {
      durationByMediaId[item.id] = item.durationMs
    }
    item.artistId?.let { artistIdByMediaId[item.id] = it }
    item.albumId?.let { albumIdByMediaId[item.id] = it }
    item.albumName?.let { albumNameByMediaId[item.id] = it }

    withController { controller ->
      val extras = Bundle().apply {
        item.albumId?.let { putString(EXTRA_ALBUM_ID, it) }
        item.artistId?.let { putString(EXTRA_ARTIST_ID, it) }
        item.albumName?.let { putString(EXTRA_ALBUM_NAME, it) }
      }
      val metadataBuilder =
        MediaMetadata
          .Builder()
          .setTitle(item.title)
          .setArtist(item.artist)
          .setExtras(extras)
      item.albumArtUrl?.let { metadataBuilder.setArtworkUri(it.toUri()) }

      val mediaItem =
        MediaItem
          .Builder()
          .setMediaId(item.id)
          .setUri(item.uri)
          .setMediaMetadata(metadataBuilder.build())
          .build()

      controller.addMediaItem(mediaItem)
    }
  }

  fun playNext(item: QueueItem) {
    if (item.durationMs > 0L) {
      durationByMediaId[item.id] = item.durationMs
    }
    item.artistId?.let { artistIdByMediaId[item.id] = it }
    item.albumId?.let { albumIdByMediaId[item.id] = it }
    item.albumName?.let { albumNameByMediaId[item.id] = it }

    withController { controller ->
      val extras = Bundle().apply {
        item.albumId?.let { putString(EXTRA_ALBUM_ID, it) }
        item.artistId?.let { putString(EXTRA_ARTIST_ID, it) }
        item.albumName?.let { putString(EXTRA_ALBUM_NAME, it) }
      }
      val metadataBuilder =
        MediaMetadata
          .Builder()
          .setTitle(item.title)
          .setArtist(item.artist)
          .setExtras(extras)
      item.albumArtUrl?.let { metadataBuilder.setArtworkUri(it.toUri()) }

      val mediaItem =
        MediaItem
          .Builder()
          .setMediaId(item.id)
          .setUri(item.uri)
          .setMediaMetadata(metadataBuilder.build())
          .build()

      val insertIndex = controller.currentMediaItemIndex + 1
      controller.addMediaItem(insertIndex, mediaItem)
    }
  }

  fun play(
    url: String,
    mediaId: String,
    title: String,
    artist: String?,
    albumArtUrl: String? = null,
    durationMs: Long = 0L,
    albumId: String? = null,
    artistId: String? = null,
    albumName: String? = null,
  ) {
    if (durationMs > 0L) {
      durationByMediaId[mediaId] = durationMs
    }
    artistId?.let { artistIdByMediaId[mediaId] = it }
    albumId?.let { albumIdByMediaId[mediaId] = it }
    albumName?.let { albumNameByMediaId[mediaId] = it }

    withController { controller ->
      val extras = Bundle().apply {
        albumId?.let { putString(EXTRA_ALBUM_ID, it) }
        artistId?.let { putString(EXTRA_ARTIST_ID, it) }
        albumName?.let { putString(EXTRA_ALBUM_NAME, it) }
      }
      val metadataBuilder =
        MediaMetadata
          .Builder()
          .setTitle(title)
          .setArtist(artist)
          .setExtras(extras)
      albumArtUrl?.let { metadataBuilder.setArtworkUri(it.toUri()) }

      val mediaItem =
        MediaItem
          .Builder()
          .setMediaId(mediaId)
          .setUri(url)
          .setMediaMetadata(metadataBuilder.build())
          .build()
      controller.setMediaItem(mediaItem)
      controller.prepare()
      controller.playWhenReady = true
    }
  }

  fun pause() {
    withController { controller ->
      controller.pause()
    }
  }

  fun resume() {
    withController { controller ->
      controller.playWhenReady = true
      if (controller.playbackState == Player.STATE_IDLE) {
        controller.prepare()
      }
    }
  }

  fun stop() {
    withController { controller ->
      controller.stop()
    }
  }

  fun seekTo(positionMs: Long) {
    withController { controller ->
      val duration = controller.duration
      val clampedPosition =
        if (duration == C.TIME_UNSET) {
          positionMs.coerceAtLeast(0)
        } else {
          positionMs.coerceIn(0, duration)
        }
      controller.seekTo(clampedPosition)
    }
  }

  fun skipNext() {
    withController { controller ->
      if (controller.hasNextMediaItem()) {
        controller.seekToNextMediaItem()
      }
    }
  }

  fun skipPrevious() {
    withController { controller ->
      if (controller.hasPreviousMediaItem()) {
        controller.seekToPreviousMediaItem()
      }
    }
  }

  fun toggleShuffle() {
    withController { controller ->
      controller.shuffleModeEnabled = !controller.shuffleModeEnabled
    }
  }

  fun cycleRepeatMode() {
    withController { controller ->
      controller.repeatMode =
        when (controller.repeatMode) {
          Player.REPEAT_MODE_OFF -> Player.REPEAT_MODE_ONE
          Player.REPEAT_MODE_ONE -> Player.REPEAT_MODE_ALL
          else -> Player.REPEAT_MODE_OFF
        }
    }
  }

  fun getShuffleEnabled(): Boolean = mediaController?.shuffleModeEnabled ?: false

  fun getRepeatMode(): RepeatMode =
    when (mediaController?.repeatMode) {
      Player.REPEAT_MODE_ONE -> RepeatMode.ONE
      Player.REPEAT_MODE_ALL -> RepeatMode.ALL
      else -> RepeatMode.NONE
    }

  fun getCurrentState(): PlayerSnapshot {
    val controller = mediaController
    return if (controller == null) {
      PlayerSnapshot()
    } else {
      snapshotFrom(controller)
    }
  }

  fun observe(onUpdate: (PlayerSnapshot) -> Unit) {
    withController { controller ->
      val listener =
        object : Player.Listener {
          override fun onIsPlayingChanged(isPlaying: Boolean) {
            onUpdate(snapshotFrom(controller))
          }

          override fun onPlaybackStateChanged(playbackState: Int) {
            onUpdate(snapshotFrom(controller))
          }

          override fun onMediaMetadataChanged(mediaMetadata: MediaMetadata) {
            onUpdate(snapshotFrom(controller))
          }

          override fun onPositionDiscontinuity(
            oldPosition: Player.PositionInfo,
            newPosition: Player.PositionInfo,
            reason: Int,
          ) {
            onUpdate(snapshotFrom(controller))
          }

          override fun onShuffleModeEnabledChanged(shuffleModeEnabled: Boolean) {
            onUpdate(snapshotFrom(controller))
          }

          override fun onRepeatModeChanged(repeatMode: Int) {
            onUpdate(snapshotFrom(controller))
          }
        }
      controller.addListener(listener)
      onUpdate(snapshotFrom(controller))
    }
  }

  fun release() {
    controllerFuture?.let { MediaController.releaseFuture(it) }
    controllerFuture = null
    mediaController = null
    _isConnected.value = false
  }

  fun saveState(appDataDir: String) {
    if (appDataDir.isBlank()) return

    val controller = mediaController ?: return
    if (controller.mediaItemCount == 0) return

    val queueItems = mutableListOf<QueueItemData>()
    for (i in 0 until controller.mediaItemCount) {
      val mediaItem = controller.getMediaItemAt(i)
      queueItems.add(
        QueueItemData(
          id = mediaItem.mediaId,
          title = mediaItem.mediaMetadata.title?.toString() ?: "",
          artist = mediaItem.mediaMetadata.artist?.toString() ?: "",
          albumArtUrl = mediaItem.mediaMetadata.artworkUri?.toString(),
          durationMs = durationByMediaId[mediaItem.mediaId] ?: 0L,
          container = null,
          isFavorite = false,
          artistId = artistIdByMediaId[mediaItem.mediaId],
          albumId = albumIdByMediaId[mediaItem.mediaId],
          albumName = albumNameByMediaId[mediaItem.mediaId],
        ),
      )
    }

    val repeatModeStr =
      when (controller.repeatMode) {
        Player.REPEAT_MODE_ONE -> "ONE"
        Player.REPEAT_MODE_ALL -> "ALL"
        else -> "OFF"
      }

    val state =
      PlayerStateData(
        queue = queueItems,
        currentIndex = controller.currentMediaItemIndex,
        positionMs = controller.currentPosition,
        shuffleEnabled = controller.shuffleModeEnabled,
        repeatMode = repeatModeStr,
      )

    val logPosition = controller.currentPosition

    CoroutineScope(Dispatchers.IO).launch {
      try {
        savePlayerState(appDataDir, state)
        Log.d(TAG, "Player state saved: ${queueItems.size} items, position ${logPosition}ms")
      } catch (e: Exception) {
        Log.e(TAG, "Failed to save player state", e)
      }
    }
  }

  fun restoreState(
    appDataDir: String,
    buildStreamUrl: (String, String?) -> String,
  ) {
    if (appDataDir.isBlank()) return

    CoroutineScope(Dispatchers.IO).launch {
      try {
        val state = loadPlayerState(appDataDir) ?: return@launch

        if (state.queue.isEmpty()) return@launch

        val queueItems =
          state.queue.map { item ->
            QueueItem(
              id = item.id,
              uri = buildStreamUrl(item.id, item.container),
              title = item.title,
              artist = item.artist,
              albumArtUrl = item.albumArtUrl,
              durationMs = item.durationMs,
              isFavorite = item.isFavorite,
              artistId = item.artistId,
              albumId = item.albumId,
              albumName = item.albumName,
            )
          }

        withContext(Dispatchers.Main) {
          // Store durations in our map
          queueItems.forEach { item ->
            if (item.durationMs > 0L) {
              durationByMediaId[item.id] = item.durationMs
            }
            item.artistId?.let { artistIdByMediaId[item.id] = it }
            item.albumId?.let { albumIdByMediaId[item.id] = it }
            item.albumName?.let { albumNameByMediaId[item.id] = it }
          }

          withController { controller ->
            val mediaItems =
              queueItems.map { item ->
                val metadataBuilder =
                  MediaMetadata
                    .Builder()
                    .setTitle(item.title)
                    .setArtist(item.artist)
                item.albumArtUrl?.let { metadataBuilder.setArtworkUri(it.toUri()) }

                MediaItem
                  .Builder()
                  .setMediaId(item.id)
                  .setUri(item.uri)
                  .setMediaMetadata(metadataBuilder.build())
                  .build()
              }

            controller.setMediaItems(
              mediaItems,
              state.currentIndex.coerceIn(0, mediaItems.size - 1),
              state.positionMs.coerceAtLeast(0),
            )

            controller.shuffleModeEnabled = state.shuffleEnabled

            controller.repeatMode =
              when (state.repeatMode) {
                "ONE" -> Player.REPEAT_MODE_ONE
                "ALL" -> Player.REPEAT_MODE_ALL
                else -> Player.REPEAT_MODE_OFF
              }

            // Don't auto-play or prepare on restore - wait for user interaction
            controller.playWhenReady = false
          }
        }

        Log.d(TAG, "Player state restored: ${queueItems.size} items at position ${state.positionMs}ms")
      } catch (e: Exception) {
        Log.e(TAG, "Failed to restore player state", e)
      }
    }
  }

  companion object {
    private const val TAG = "PlayerController"
    private const val EXTRA_ALBUM_ID = "album_id"
    private const val EXTRA_ARTIST_ID = "artist_id"
    private const val EXTRA_ALBUM_NAME = "album_name"
  }

  private fun snapshotFrom(controller: MediaController): PlayerSnapshot {
    val controllerDuration = controller.duration
    val mediaId = controller.currentMediaItem?.mediaId
    val extras = controller.currentMediaItem?.mediaMetadata?.extras
    val fallbackDuration = mediaId?.let { durationByMediaId[it] } ?: 0L
    val duration =
      when {
        controllerDuration == C.TIME_UNSET || controllerDuration <= 0L -> fallbackDuration
        else -> controllerDuration
      }

    val currentArtistId = mediaId?.let { artistIdByMediaId[it] } ?: extras?.getString(EXTRA_ARTIST_ID)
    val currentAlbumId = mediaId?.let { albumIdByMediaId[it] } ?: extras?.getString(EXTRA_ALBUM_ID)
    val currentAlbumName = mediaId?.let { albumNameByMediaId[it] } ?: extras?.getString(EXTRA_ALBUM_NAME)

    val repeatMode =
      when (controller.repeatMode) {
        Player.REPEAT_MODE_ONE -> RepeatMode.ONE
        Player.REPEAT_MODE_ALL -> RepeatMode.ALL
        else -> RepeatMode.NONE
      }

    return PlayerSnapshot(
      title = controller.mediaMetadata.title?.toString() ?: "",
      artist = controller.mediaMetadata.artist?.toString() ?: "",
      albumArtUrl = controller.mediaMetadata.artworkUri?.toString(),
      isPlaying = controller.isPlaying,
      isBuffering = controller.playbackState == Player.STATE_BUFFERING,
      positionMs = controller.currentPosition,
      durationMs = duration,
      hasPrevious = controller.hasPreviousMediaItem(),
      hasNext = controller.hasNextMediaItem(),
      isShuffled = controller.shuffleModeEnabled,
      repeatMode = repeatMode,
      currentSongId = mediaId,
      currentAlbumId = currentAlbumId,
      currentArtistId = currentArtistId,
      currentAlbumName = currentAlbumName,
      playbackSpeed = controller.playbackParameters.speed,
      updateTimeMs = SystemClock.elapsedRealtime(),
    )
  }

  private fun withController(action: (MediaController) -> Unit) {
    val currentController = mediaController
    // Check actual connection state from the controller itself
    if (currentController != null && currentController.isConnected) {
      action(currentController)
      return
    }
    // Queue action until connected
    synchronized(pendingActions) {
      val controller = mediaController
      if (controller != null && controller.isConnected) {
        // Double-check after acquiring lock
        action(controller)
      } else {
        Log.d(TAG, "Queueing action - controller not connected")
        pendingActions.add(action)
      }
    }
  }

  private fun registerPlaybackEndedListener(onEnded: () -> Unit) {
    mediaController?.addListener(
      object : Player.Listener {
        override fun onPlaybackStateChanged(playbackState: Int) {
          if (playbackState == Player.STATE_ENDED) {
            onEnded()
          }
        }
      },
    )
  }
}

enum class RepeatMode { NONE, ONE, ALL }

data class PlayerSnapshot(
  val title: String = "",
  val artist: String = "",
  val albumArtUrl: String? = null,
  val isPlaying: Boolean = false,
  val isBuffering: Boolean = false,
  val positionMs: Long = 0L,
  val durationMs: Long = 0L,
  val hasPrevious: Boolean = false,
  val hasNext: Boolean = false,
  val isShuffled: Boolean = false,
  val repeatMode: RepeatMode = RepeatMode.NONE,
  val currentSongId: String? = null,
  val currentAlbumId: String? = null,
  val currentArtistId: String? = null,
  val currentAlbumName: String? = null,
  val playbackSpeed: Float = 1f,
  val updateTimeMs: Long = 0L,
)
