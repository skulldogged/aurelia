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
import uniffi.aurelia_core.Song
import uniffi.aurelia_core.buildMobileStreamUrl

class PlayerController(
  private val context: Context,
) {
  private val sessionToken = SessionToken(context, ComponentName(context, PlaybackService::class.java))
  private var controllerFuture: ListenableFuture<MediaController>? = null
  private var mediaController: MediaController? = null
  private var playbackEndedCallback: (() -> Unit)? = null
  private val songByMediaId: MutableMap<String, Song> = mutableMapOf()
  private var lastServerUrl: String = ""
  private var lastToken: String = ""

  // Offset tracking for non-seekable container seeking.
  // When we reload a stream with startTimeTicks, position 0 of the new stream
  // corresponds to this offset in the actual song.
  private var seekOffsetMs: Long = 0L

  companion object {
    private const val TAG = "PlayerController"
    private const val EXTRA_ALBUM_ID = "album_id"
    private const val EXTRA_ARTIST_ID = "artist_id"
    private const val EXTRA_ALBUM_NAME = "album_name"

    private val SEEKABLE_CONTAINERS = setOf("flac", "mp3", "aac", "ogg")

    private fun isContainerSeekable(container: String?): Boolean =
      container != null && container.lowercase() in SEEKABLE_CONTAINERS
  }

  // Connection state tracking
  private val _isConnected = MutableStateFlow(false)
  val isConnected: StateFlow<Boolean> = _isConnected.asStateFlow()
  private val _snapshots = MutableStateFlow(PlayerSnapshot())
  val snapshots: StateFlow<PlayerSnapshot> = _snapshots.asStateFlow()

  // Pending actions to run once connected
  private val pendingActions = mutableListOf<(MediaController) -> Unit>()

  // Track if we're in the process of connecting
  private var isConnecting = false
  private var reconnectJob: kotlinx.coroutines.Job? = null

  private val controllerListener =
    object : Player.Listener {
      override fun onEvents(player: Player, events: Player.Events) {
        val controller = mediaController ?: return
        val wasConnected = _isConnected.value
        val nowConnected = controller.isConnected

        if (nowConnected && !wasConnected) {
          onControllerConnected(controller)
        } else if (!nowConnected && wasConnected) {
          Log.d(TAG, "MediaController disconnected - will reconnect")
          _isConnected.value = false
          isConnecting = false
          scheduleReconnect()
        }

        publishSnapshot(controller)
      }

      override fun onMediaItemTransition(mediaItem: MediaItem?, reason: Int) {
        if (reason != Player.MEDIA_ITEM_TRANSITION_REASON_PLAYLIST_CHANGED) {
          seekOffsetMs = 0L
        }
      }
    }

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
          controller.removeListener(controllerListener)
          controller.addListener(controllerListener)

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

  private fun scheduleReconnect() {
    reconnectJob?.cancel()
    reconnectJob =
      CoroutineScope(Dispatchers.Main).launch {
        delay(100)
        connectToService()
      }
  }

  private fun onControllerConnected(controller: MediaController) {
    Log.d(TAG, "MediaController connected")
    _isConnected.value = true
    isConnecting = false
    publishSnapshot(controller)
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
    songs: List<Song>,
    serverUrl: String,
    token: String,
    startIndex: Int = 0,
    startPositionMs: Long = 0L,
    autoPlay: Boolean = true,
  ) {
    songByMediaId.clear()
    songs.forEach { song -> songByMediaId[song.id] = song }
    lastServerUrl = serverUrl
    lastToken = token
    seekOffsetMs = 0L

    withController { controller ->
      val mediaItems = songs.map { song -> buildMediaItem(song, serverUrl, token) }
      controller.setMediaItems(mediaItems, startIndex, startPositionMs)
      controller.prepare()
      controller.playWhenReady = autoPlay
    }
  }

  fun getQueue(): List<Song> {
    val controller = mediaController ?: return emptyList()
    val songs = mutableListOf<Song>()
    for (i in 0 until controller.mediaItemCount) {
      val mediaItem = controller.getMediaItemAt(i)
      songByMediaId[mediaItem.mediaId]?.let { songs.add(it) }
    }
    return songs
  }

  fun getCurrentQueueIndex(): Int = mediaController?.currentMediaItemIndex ?: -1

  fun playQueueItem(index: Int) {
    seekOffsetMs = 0L
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

  fun addToQueue(song: Song, serverUrl: String, token: String) {
    songByMediaId[song.id] = song
    withController { controller ->
      controller.addMediaItem(buildMediaItem(song, serverUrl, token))
    }
  }

  fun playNext(song: Song, serverUrl: String, token: String) {
    songByMediaId[song.id] = song
    withController { controller ->
      val insertIndex = controller.currentMediaItemIndex + 1
      controller.addMediaItem(insertIndex, buildMediaItem(song, serverUrl, token))
    }
  }

  fun play(song: Song, serverUrl: String, token: String) {
    songByMediaId[song.id] = song
    lastServerUrl = serverUrl
    lastToken = token
    seekOffsetMs = 0L
    withController { controller ->
      controller.setMediaItem(buildMediaItem(song, serverUrl, token))
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
      val mediaId = controller.currentMediaItem?.mediaId
      val song = mediaId?.let { songByMediaId[it] }
      val songDurationMs = song?.duration?.let { (it * 1000).toLong() } ?: 0L
      val controllerDuration = controller.duration
      val fullDuration = if (songDurationMs > 0L) songDurationMs
        else if (controllerDuration != C.TIME_UNSET && controllerDuration > 0L) controllerDuration + seekOffsetMs
        else 0L

      val targetPosition =
        if (fullDuration <= 0L) positionMs.coerceAtLeast(0)
        else positionMs.coerceIn(0, fullDuration)

      if (song != null && !isContainerSeekable(song.container) && lastServerUrl.isNotBlank()) {
        // Non-seekable container: reload the stream with startTimeTicks
        val wasPlaying = controller.isPlaying
        val currentIndex = controller.currentMediaItemIndex
        val ticks = targetPosition * 10_000 // ms to ticks (1 tick = 100ns)

        // Rebuild the current item's URL with startTimeTicks
        val baseUrl = buildMobileStreamUrl(lastServerUrl, lastToken, song.id, song.container)
        val seekUrl = "$baseUrl&startTimeTicks=$ticks"
        val newItem = buildMediaItemWithUri(song, seekUrl)

        // Rebuild queue with the updated item
        val mediaItems = mutableListOf<MediaItem>()
        for (i in 0 until controller.mediaItemCount) {
          if (i == currentIndex) mediaItems.add(newItem)
          else mediaItems.add(controller.getMediaItemAt(i))
        }

        seekOffsetMs = targetPosition
        controller.setMediaItems(mediaItems, currentIndex, 0L)
        controller.prepare()
        controller.playWhenReady = wasPlaying
      } else {
        controller.seekTo(targetPosition)
      }
    }
  }

  fun skipNext() {
    seekOffsetMs = 0L
    withController { controller ->
      if (controller.hasNextMediaItem()) {
        controller.seekToNextMediaItem()
      }
    }
  }

  fun skipPrevious() {
    seekOffsetMs = 0L
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
      snapshots.value
    } else {
      snapshotFrom(controller)
    }
  }

  fun release() {
    reconnectJob?.cancel()
    reconnectJob = null
    mediaController?.removeListener(controllerListener)
    controllerFuture?.let { MediaController.releaseFuture(it) }
    controllerFuture = null
    mediaController = null
    _isConnected.value = false
    _snapshots.value = PlayerSnapshot()
  }

  private fun buildMediaItem(song: Song, serverUrl: String, token: String): MediaItem {
    val uri = buildMobileStreamUrl(serverUrl, token, song.id, song.container)
    return buildMediaItemWithUri(song, uri)
  }

  private fun buildMediaItemWithUri(song: Song, uri: String): MediaItem {
    val artist = song.artists?.joinToString(", ") ?: ""
    val extras = Bundle().apply {
      song.albumId?.let { putString(EXTRA_ALBUM_ID, it) }
      song.artistIds?.firstOrNull()?.let { putString(EXTRA_ARTIST_ID, it) }
      song.album?.let { putString(EXTRA_ALBUM_NAME, it) }
    }
    val metadataBuilder =
      MediaMetadata
        .Builder()
        .setTitle(song.name)
        .setArtist(artist)
        .setExtras(extras)
    song.albumArtUrl?.let { metadataBuilder.setArtworkUri(it.toUri()) }

    return MediaItem
      .Builder()
      .setMediaId(song.id)
      .setUri(uri)
      .setMediaMetadata(metadataBuilder.build())
      .build()
  }

  private fun snapshotFrom(controller: MediaController): PlayerSnapshot {
    val controllerDuration = controller.duration
    val mediaId = controller.currentMediaItem?.mediaId
    val song = mediaId?.let { songByMediaId[it] }
    val fallbackDurationMs = song?.duration?.let { (it * 1000).toLong() } ?: 0L
    val duration =
      when {
        controllerDuration == C.TIME_UNSET || controllerDuration <= 0L -> fallbackDurationMs
        else -> controllerDuration
      }

    val repeatMode =
      when (controller.repeatMode) {
        Player.REPEAT_MODE_ONE -> RepeatMode.ONE
        Player.REPEAT_MODE_ALL -> RepeatMode.ALL
        else -> RepeatMode.NONE
      }

    // Prefer metadata from the current item as it's more immediate during transitions
    val currentItem = controller.currentMediaItem
    val metadata = currentItem?.mediaMetadata ?: controller.mediaMetadata

    // When playing a non-seekable container that was seeked via startTimeTicks reload,
    // player position is relative to the reload point. Add seekOffsetMs to get real position.
    val adjustedPosition = controller.currentPosition + seekOffsetMs
    val songDurationMs = song?.duration?.let { (it * 1000).toLong() } ?: 0L
    val adjustedDuration = if (seekOffsetMs > 0L && songDurationMs > 0L) songDurationMs else duration

    return PlayerSnapshot(
      title = metadata.title?.toString() ?: "",
      artist = metadata.artist?.toString() ?: "",
      albumArtUrl = metadata.artworkUri?.toString(),
      isPlaying = controller.isPlaying,
      isBuffering = controller.playbackState == Player.STATE_BUFFERING,
      positionMs = adjustedPosition,
      durationMs = adjustedDuration,
      hasPrevious = controller.hasPreviousMediaItem(),
      hasNext = controller.hasNextMediaItem(),
      isShuffled = controller.shuffleModeEnabled,
      repeatMode = repeatMode,
      currentSongId = mediaId,
      currentAlbumId = song?.albumId,
      currentArtistId = song?.artistIds?.firstOrNull(),
      currentAlbumName = song?.album,
      playbackSpeed = controller.playbackParameters.speed,
      updateTimeMs = SystemClock.elapsedRealtime(),
      codec = song?.codec,
      bitRate = song?.bitRate,
      sampleRate = song?.sampleRate,
    )
  }

  private fun publishSnapshot(controller: MediaController) {
    _snapshots.value = snapshotFrom(controller)
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
  val codec: String? = null,
  val bitRate: Int? = null,
  val sampleRate: Int? = null,
)
