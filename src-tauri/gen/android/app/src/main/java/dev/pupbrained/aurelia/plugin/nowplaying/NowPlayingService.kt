package dev.pupbrained.aurelia.plugin.nowplaying

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.support.v4.media.MediaMetadataCompat
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import android.util.Base64
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.media.app.NotificationCompat.MediaStyle
import androidx.media.session.MediaButtonReceiver
import dev.pupbrained.aurelia.R
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.Executors

class NowPlayingService : Service() {
    private val notificationManager by lazy { NotificationManagerCompat.from(this) }
    private val mainHandler = Handler(Looper.getMainLooper())
    private val executor = Executors.newSingleThreadExecutor()
    private lateinit var mediaSession: MediaSessionCompat

    private var foregroundStarted = false
    private var currentArtworkKey: String? = null
    private var currentArtwork: Bitmap? = null
    private var currentInfo: NowPlayingInfo? = null

    private val mediaSessionCallback = object : MediaSessionCompat.Callback() {
        override fun onPlay() {
            dispatchControlAction("play")
        }

        override fun onPause() {
            dispatchControlAction("pause")
        }

        override fun onStop() {
            handleClear()
        }

        override fun onSkipToNext() {
            dispatchControlAction("next")
        }

        override fun onSkipToPrevious() {
            dispatchControlAction("previous")
        }

        override fun onSeekTo(pos: Long) {
            val positionSeconds = pos / 1000.0
            dispatchControlAction("seek:$positionSeconds")
        }
    }

    override fun onCreate() {
        super.onCreate()
        mediaSession = MediaSessionCompat(this, SESSION_TAG).apply {
            setFlags(MediaSessionCompat.FLAG_HANDLES_MEDIA_BUTTONS or MediaSessionCompat.FLAG_HANDLES_TRANSPORT_CONTROLS)
            setCallback(mediaSessionCallback, mainHandler)
            isActive = true
        }
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_CLEAR -> handleClear()
            ACTION_UPDATE -> {
                val info = NowPlayingInfo.fromIntent(intent)
                handleUpdate(info)
            }
            else -> {
                intent?.let {
                    val info = NowPlayingInfo.fromIntent(it)
                    handleUpdate(info)
                }
            }
        }
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        stopForegroundInternal()
        mediaSession.isActive = false
        mediaSession.release()
        executor.shutdownNow()
        currentArtwork = null
        currentArtworkKey = null
        currentInfo = null
    }

    private fun handleUpdate(info: NowPlayingInfo) {
        mainHandler.post {
            currentInfo = info
            updateSession(info, currentArtwork)
            loadArtworkIfNecessary(info)
        }
    }

    private fun dispatchControlAction(action: String) {
        Log.d(TAG, "Dispatching control action: $action")
        NowPlayingBridge.emit(action)
    }

    private fun handleClear() {
        mainHandler.post {
            currentInfo = null
            currentArtwork = null
            currentArtworkKey = null

            dispatchControlAction("stop")

            val playbackState = PlaybackStateCompat.Builder()
                .setState(PlaybackStateCompat.STATE_STOPPED, 0, 0f)
                .setActions(0)
                .build()
            mediaSession.setPlaybackState(playbackState)
            mediaSession.setMetadata(MediaMetadataCompat.Builder().build())
            mediaSession.isActive = false

            notificationManager.cancel(NOTIFICATION_ID)
            stopForegroundInternal()
            stopSelf()
        }
    }

    private fun loadArtworkIfNecessary(info: NowPlayingInfo) {
        val sourceKey = when {
            !info.artworkPath.isNullOrBlank() -> "path:${info.artworkPath}"
            !info.artworkData.isNullOrBlank() -> "data:${info.artworkData.hashCode()}"
            !info.artworkUrl.isNullOrBlank() -> "url:${info.artworkUrl}"
            else -> null
        }

        if (sourceKey == null) {
            if (currentArtwork != null) {
                currentArtwork = null
                currentArtworkKey = null
                updateSession(info, null)
            }
            return
        }

        if (sourceKey == currentArtworkKey) {
            return
        }

        executor.execute {
            val bitmap = loadArtworkBitmap(info)
            currentArtworkKey = sourceKey
            currentArtwork = bitmap
            mainHandler.post {
                currentInfo?.let { active -> updateSession(active, bitmap) }
            }
        }
    }

    private fun loadArtworkBitmap(info: NowPlayingInfo): Bitmap? {
        val path = info.artworkPath?.takeIf { it.isNotBlank() }
        if (!path.isNullOrBlank()) {
            val file = File(path)
            if (file.exists()) {
                BitmapFactory.decodeFile(file.absolutePath)?.let { return it }
            }
        }

        val base64Data = info.artworkData?.takeIf { it.isNotBlank() }
        if (!base64Data.isNullOrBlank()) {
            val encoded = if (base64Data.startsWith("data")) {
                base64Data.substringAfter(',')
            } else {
                base64Data
            }
            try {
                val bytes = Base64.decode(encoded, Base64.DEFAULT)
                BitmapFactory.decodeByteArray(bytes, 0, bytes.size)?.let { return it }
            } catch (exception: Exception) {
                Log.w(TAG, "Failed to decode artwork from base64", exception)
            }
        }

        val artworkUrl = info.artworkUrl?.takeIf { it.isNotBlank() } ?: return null
        var connection: HttpURLConnection? = null
        return try {
            connection = (URL(artworkUrl).openConnection() as HttpURLConnection).apply {
                connectTimeout = 5000
                readTimeout = 5000
                instanceFollowRedirects = true
                doInput = true
            }
            connection.connect()
            connection.inputStream.use { BitmapFactory.decodeStream(it) }
        } catch (exception: Exception) {
            Log.w(TAG, "Failed to download artwork", exception)
            null
        } finally {
            connection?.disconnect()
        }
    }

    private fun updateSession(info: NowPlayingInfo, artwork: Bitmap?) {
        val artistsText = if (info.artists.isEmpty()) "" else info.artists.joinToString(", ")
        val durationMs = info.durationSeconds?.let { (it * 1000L).toLong().coerceAtLeast(0L) } ?: 0L
        val positionMs = info.positionSeconds?.let { (it * 1000L).toLong().coerceAtLeast(0L) } ?: 0L

        val metadataBuilder = MediaMetadataCompat.Builder()
            .putString(MediaMetadataCompat.METADATA_KEY_TITLE, info.title)
            .putString(MediaMetadataCompat.METADATA_KEY_ARTIST, artistsText)
            .putString(MediaMetadataCompat.METADATA_KEY_ALBUM, info.album ?: "")
            .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, durationMs)
            .putString(MediaMetadataCompat.METADATA_KEY_MEDIA_ID, info.mediaId ?: "")

        artwork?.let {
            metadataBuilder.putBitmap(MediaMetadataCompat.METADATA_KEY_ALBUM_ART, it)
            metadataBuilder.putBitmap(MediaMetadataCompat.METADATA_KEY_ART, it)
        }

        mediaSession.setMetadata(metadataBuilder.build())

        val playbackStateBuilder = PlaybackStateCompat.Builder()
            .setActions(resolvePlaybackActions(info))
            .setState(
                if (info.isPlaying) PlaybackStateCompat.STATE_PLAYING else PlaybackStateCompat.STATE_PAUSED,
                positionMs,
                if (info.isPlaying) 1f else 0f
            )

        mediaSession.setPlaybackState(playbackStateBuilder.build())
        mediaSession.isActive = true

        val notification = buildNotification(info, artistsText, artwork)

        if (!foregroundStarted) {
            startForeground(NOTIFICATION_ID, notification)
            foregroundStarted = true
        } else {
            notificationManager.notify(NOTIFICATION_ID, notification)
        }
    }

    private fun resolvePlaybackActions(info: NowPlayingInfo): Long {
        var actions = PlaybackStateCompat.ACTION_PLAY or
                      PlaybackStateCompat.ACTION_PAUSE or
                      PlaybackStateCompat.ACTION_PLAY_PAUSE or
                      PlaybackStateCompat.ACTION_STOP or
                      PlaybackStateCompat.ACTION_SEEK_TO

        if (info.hasNext) actions = actions or PlaybackStateCompat.ACTION_SKIP_TO_NEXT
        if (info.hasPrevious) actions = actions or PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS
        return actions
    }

    private fun createMediaAction(iconRes: Int, titleRes: Int, action: Long): NotificationCompat.Action {
        val intent = MediaButtonReceiver.buildMediaButtonPendingIntent(this, action)
        return NotificationCompat.Action(iconRes, getString(titleRes), intent)
    }

    private fun buildNotification(info: NowPlayingInfo, artistsText: String, artwork: Bitmap?): Notification {
        val contentIntent = createContentIntent()
        val secondaryText = when {
            artistsText.isNotBlank() && !info.album.isNullOrBlank() -> info.album
            artistsText.isBlank() -> info.album
            else -> artistsText
        }

        val builder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(info.title.ifBlank { getString(R.string.app_name) })
            .setContentText(artistsText.ifBlank { info.album ?: "" })
            .setSubText(secondaryText)
            .setSmallIcon(R.drawable.ic_notification)
            .setLargeIcon(artwork)
            .setOnlyAlertOnce(true)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .setOngoing(info.isPlaying)
            .setSilent(true)
            .setCategory(Notification.CATEGORY_TRANSPORT)
            .setPriority(NotificationCompat.PRIORITY_LOW)

        contentIntent?.let { pending ->
            builder.setContentIntent(pending)
            mediaSession.setSessionActivity(pending)
        }

        val compactActionIndices = mutableListOf<Int>()
        var actionIndex = 0

        if (info.hasPrevious) {
            builder.addAction(
                createMediaAction(
                    android.R.drawable.ic_media_previous,
                    R.string.notification_action_previous,
                    PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS,
                ),
            )
            compactActionIndices.add(actionIndex)
            actionIndex += 1
        }

        val playPauseAction = if (info.isPlaying) {
            createMediaAction(
                android.R.drawable.ic_media_pause,
                R.string.notification_action_pause,
                PlaybackStateCompat.ACTION_PAUSE,
            )
        } else {
            createMediaAction(
                android.R.drawable.ic_media_play,
                R.string.notification_action_play,
                PlaybackStateCompat.ACTION_PLAY,
            )
        }

        builder.addAction(playPauseAction)
        compactActionIndices.add(actionIndex)
        actionIndex += 1

        if (info.hasNext) {
            builder.addAction(
                createMediaAction(
                    android.R.drawable.ic_media_next,
                    R.string.notification_action_next,
                    PlaybackStateCompat.ACTION_SKIP_TO_NEXT,
                ),
            )
            compactActionIndices.add(actionIndex)
        }

        val mediaStyle = MediaStyle().setMediaSession(mediaSession.sessionToken)
        if (compactActionIndices.isNotEmpty()) {
            mediaStyle.setShowActionsInCompactView(*compactActionIndices.toIntArray())
        }

        builder.setStyle(mediaStyle)

        return builder.build()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = getSystemService(NotificationManager::class.java)
            val existing = manager?.getNotificationChannel(CHANNEL_ID)
            if (existing == null) {
                val channel = NotificationChannel(
                    CHANNEL_ID,
                    getString(R.string.notification_channel_playback),
                    NotificationManager.IMPORTANCE_LOW
                ).apply {
                    description = getString(R.string.notification_channel_playback_description)
                    setShowBadge(false)
                }
                manager?.createNotificationChannel(channel)
            }
        }
    }

    private fun createContentIntent(): PendingIntent? {
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName) ?: return null
        launchIntent.flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
        val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        return PendingIntent.getActivity(this, 0, launchIntent, flags)
    }

    private fun stopForegroundInternal() {
        if (!foregroundStarted) return
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            stopForeground(STOP_FOREGROUND_REMOVE)
        } else {
            @Suppress("DEPRECATION")
            stopForeground(true)
        }
        foregroundStarted = false
    }

    data class NowPlayingInfo(
        val mediaId: String?,
        val title: String,
        val artists: List<String>,
        val album: String?,
        val durationSeconds: Double?,
        val positionSeconds: Double?,
        val isPlaying: Boolean,
        val hasNext: Boolean,
        val hasPrevious: Boolean,
        val isShuffled: Boolean,
        val repeatMode: String?,
        val artworkUrl: String?,
        val artworkPath: String?,
        val artworkData: String?,
    ) {
        companion object {
            fun fromIntent(intent: Intent): NowPlayingInfo {
                val artists = intent.getStringArrayListExtra(EXTRA_ARTISTS)?.toList() ?: emptyList()
                val duration = intentExtrasNullableDouble(intent, EXTRA_DURATION_SECONDS)
                val position = intentExtrasNullableDouble(intent, EXTRA_POSITION_SECONDS)
                return NowPlayingInfo(
                    mediaId = intent.getStringExtra(EXTRA_MEDIA_ID),
                    title = intent.getStringExtra(EXTRA_TITLE) ?: "",
                    artists = artists,
                    album = intent.getStringExtra(EXTRA_ALBUM),
                    durationSeconds = duration,
                    positionSeconds = position,
                    isPlaying = intent.getBooleanExtra(EXTRA_IS_PLAYING, false),
                    hasNext = intent.getBooleanExtra(EXTRA_HAS_NEXT, false),
                    hasPrevious = intent.getBooleanExtra(EXTRA_HAS_PREVIOUS, false),
                    isShuffled = intent.getBooleanExtra(EXTRA_IS_SHUFFLED, false),
                    repeatMode = intent.getStringExtra(EXTRA_REPEAT_MODE),
                    artworkUrl = intent.getStringExtra(EXTRA_ARTWORK_URL),
                    artworkPath = intent.getStringExtra(EXTRA_ARTWORK_PATH),
                    artworkData = intent.getStringExtra(EXTRA_ARTWORK_DATA),
                )
            }

            private fun intentExtrasNullableDouble(intent: Intent, key: String): Double? =
                if (intent.hasExtra(key)) intent.getDoubleExtra(key, 0.0) else null
        }
    }

    companion object {
        const val ACTION_UPDATE = "dev.pupbrained.aurelia.plugin.nowplaying.UPDATE"
        const val ACTION_CLEAR = "dev.pupbrained.aurelia.plugin.nowplaying.CLEAR"

        const val EXTRA_MEDIA_ID = "extra_media_id"
        const val EXTRA_TITLE = "extra_title"
        const val EXTRA_ARTISTS = "extra_artists"
        const val EXTRA_ALBUM = "extra_album"
        const val EXTRA_DURATION_SECONDS = "extra_duration_seconds"
        const val EXTRA_POSITION_SECONDS = "extra_position_seconds"
        const val EXTRA_IS_PLAYING = "extra_is_playing"
        const val EXTRA_HAS_NEXT = "extra_has_next"
        const val EXTRA_HAS_PREVIOUS = "extra_has_previous"
        const val EXTRA_IS_SHUFFLED = "extra_is_shuffled"
        const val EXTRA_REPEAT_MODE = "extra_repeat_mode"
        const val EXTRA_ARTWORK_URL = "extra_artwork_url"
        const val EXTRA_ARTWORK_PATH = "extra_artwork_path"
        const val EXTRA_ARTWORK_DATA = "extra_artwork_data"

        private const val CHANNEL_ID = "aurelia_now_playing"
        private const val SESSION_TAG = "AureliaNowPlaying"
        private const val NOTIFICATION_ID = 2107
        private const val TAG = "NowPlayingService"
    }
}
