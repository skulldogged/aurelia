package com.aurelia.app.player

import android.content.ComponentName
import android.content.Context
import android.net.Uri
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.Player
import androidx.media3.session.MediaController
import androidx.media3.session.SessionToken
import com.google.common.util.concurrent.ListenableFuture
import com.google.common.util.concurrent.MoreExecutors

data class QueueItem(
    val id: String,
    val uri: String,
    val title: String,
    val artist: String,
    val albumArtUrl: String?,
    val durationMs: Long = 0L,
    val isFavorite: Boolean = false,
)

class PlayerController(
    private val context: Context,
) {
    private val sessionToken = SessionToken(context, ComponentName(context, PlaybackService::class.java))
    private val controllerFuture: ListenableFuture<MediaController>
    private var mediaController: MediaController? = null
    private var playbackEndedCallback: (() -> Unit)? = null
    private val durationByMediaId: MutableMap<String, Long> = mutableMapOf()

    init {
        controllerFuture = MediaController.Builder(context, sessionToken).buildAsync()
        controllerFuture.addListener(
            {
                mediaController = controllerFuture.get()
                playbackEndedCallback?.let { registerPlaybackEndedListener(it) }
            },
            MoreExecutors.directExecutor(),
        )
    }

    fun setOnPlaybackEnded(onEnded: () -> Unit) {
        playbackEndedCallback = onEnded
        mediaController?.let { registerPlaybackEndedListener(onEnded) }
    }

    fun setQueue(
        items: List<QueueItem>,
        startIndex: Int = 0,
    ) {
        durationByMediaId.clear()
        items.forEach { item ->
            if (item.durationMs > 0L) {
                durationByMediaId[item.id] = item.durationMs
            }
        }

        withController { controller ->
            val mediaItems =
                items.map { item ->
                    val metadataBuilder =
                        MediaMetadata
                            .Builder()
                            .setTitle(item.title)
                            .setArtist(item.artist)
                    item.albumArtUrl?.let { metadataBuilder.setArtworkUri(Uri.parse(it)) }

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
            items.add(
                QueueItem(
                    id = mediaItem.mediaId,
                    uri = mediaItem.localConfiguration?.uri?.toString() ?: "",
                    title = mediaItem.mediaMetadata.title?.toString() ?: "",
                    artist = mediaItem.mediaMetadata.artist?.toString() ?: "",
                    albumArtUrl = mediaItem.mediaMetadata.artworkUri?.toString(),
                    durationMs = durationByMediaId[mediaItem.mediaId] ?: 0L,
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
            }
        }
    }

    fun play(
        url: String,
        mediaId: String,
        title: String,
        artist: String?,
        albumArtUrl: String? = null,
        durationMs: Long = 0L,
    ) {
        if (durationMs > 0L) {
            durationByMediaId[mediaId] = durationMs
        }

        withController { controller ->
            val metadataBuilder =
                MediaMetadata
                    .Builder()
                    .setTitle(title)
                    .setArtist(artist ?: "Unknown Artist")

            albumArtUrl?.let { artUrl ->
                metadataBuilder.setArtworkUri(android.net.Uri.parse(artUrl))
            }

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
            val controllerDuration = controller.duration
            val mediaId = controller.currentMediaItem?.mediaId
            val fallbackDuration = mediaId?.let { durationByMediaId[it] } ?: 0L
            val duration =
                when {
                    controllerDuration == C.TIME_UNSET || controllerDuration <= 0L -> fallbackDuration
                    else -> controllerDuration
                }

            PlayerSnapshot(
                title = controller.mediaMetadata.title?.toString() ?: "",
                artist = controller.mediaMetadata.artist?.toString() ?: "",
                isPlaying = controller.isPlaying,
                positionMs = controller.currentPosition,
                durationMs = duration,
            )
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
        MediaController.releaseFuture(controllerFuture)
    }

    private fun snapshotFrom(controller: MediaController): PlayerSnapshot {
        val controllerDuration = controller.duration
        val mediaId = controller.currentMediaItem?.mediaId
        val fallbackDuration = mediaId?.let { durationByMediaId[it] } ?: 0L
        val duration =
            when {
                controllerDuration == C.TIME_UNSET || controllerDuration <= 0L -> fallbackDuration
                else -> controllerDuration
            }

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
        )
    }

    private fun withController(action: (MediaController) -> Unit) {
        val currentController = mediaController
        if (currentController != null) {
            action(currentController)
            return
        }
        controllerFuture.addListener(
            { action(controllerFuture.get()) },
            MoreExecutors.directExecutor(),
        )
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
)
