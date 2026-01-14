package dev.pupbrained.aurelia.plugin.audio

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.content.pm.ServiceInfo
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.audiofx.Equalizer
import android.media.audiofx.Visualizer
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.net.Uri
import android.util.Log
import androidx.annotation.OptIn
import androidx.core.app.NotificationCompat
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.ForwardingPlayer
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.session.DefaultMediaNotificationProvider
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import dev.pupbrained.aurelia.R
import okhttp3.OkHttpClient
import java.net.URL
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit

@OptIn(UnstableApi::class)
class AudioPlayerService : MediaSessionService() {
    private var mediaSession: MediaSession? = null
    private var player: ExoPlayer? = null
    private var equalizer: Equalizer? = null
    private var visualizer: Visualizer? = null
    private var spectrumEmitCount = 0  // Debug counter for spectrum emissions

    private val mainHandler = Handler(Looper.getMainLooper())
    private val executor = Executors.newSingleThreadExecutor()
    private var positionUpdateRunnable: Runnable? = null
    private var isVisualizerEnabled = false
    private var visualizerFailed = false  // Prevent retry after permission failure

    // EQ band mapping (our conceptual bands to Android bands)
    private var eqBandMapping: IntArray? = null
    private val targetFrequencies = intArrayOf(60, 250, 1000, 4000, 16000)

    // Prepared next track info
    private var preparedNextUrl: String? = null
    private var preparedNextToken: String? = null

    // Current auth token for HTTP requests
    private var currentAuthToken: String? = null

    // Artwork cache
    private var currentArtwork: Bitmap? = null
    private var currentArtworkUrl: String? = null

    companion object {
        private const val TAG = "AudioPlayerService"
        private const val CHANNEL_ID = "aurelia_audio_playback"
        private const val NOTIFICATION_ID = 2108
        private const val POSITION_UPDATE_INTERVAL_MS = 250L

        // Singleton instance for plugin access
        @Volatile
        private var instance: AudioPlayerService? = null

        fun getInstance(): AudioPlayerService? = instance
    }

    override fun onCreate() {
        super.onCreate()
        instance = this
        createNotificationChannel()
        initializePlayer()
        startPositionUpdates()
        
        // Set up a custom notification provider with our icon
        setMediaNotificationProvider(
            DefaultMediaNotificationProvider.Builder(this)
                .setChannelId(CHANNEL_ID)
                .setChannelName(R.string.notification_channel_playback)
                .build()
                .apply { setSmallIcon(R.drawable.ic_notification) }
        )
        
        Log.d(TAG, "AudioPlayerService created")
    }

    // Track if we've started foreground
    private var foregroundStarted = false

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // MUST call startForeground() immediately when started via startForegroundService()
        // to prevent ANR/crash. Use a placeholder notification that will be replaced
        // by MediaSessionService once playback starts.
        if (!foregroundStarted) {
            foregroundStarted = true
            val notification = buildPlaceholderNotification()
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK)
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
            Log.d(TAG, "Started foreground with placeholder notification")
        }
        // Let MediaSessionService handle the rest (it will update the notification)
        return super.onStartCommand(intent, flags, startId)
    }
    
    private fun buildPlaceholderNotification(): Notification {
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle(getString(R.string.app_name))
            .setContentText(getString(R.string.notification_loading))
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .setSilent(true)
            .apply { 
                createContentIntent()?.let { setContentIntent(it) }
            }
            .build()
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? {
        return mediaSession
    }

    override fun onDestroy() {
        stopPositionUpdates()
        releaseVisualizer()
        releaseEqualizer()
        mediaSession?.run {
            removeSession(this)
            player.release()
            release()
            mediaSession = null
        }
        player = null
        instance = null
        executor.shutdownNow()
        Log.d(TAG, "AudioPlayerService destroyed")
        super.onDestroy()
    }
    
    override fun onTaskRemoved(rootIntent: Intent?) {
        val currentPlayer = player
        // Keep playing if music is active, otherwise stop the service
        if (currentPlayer == null || !currentPlayer.playWhenReady || 
            currentPlayer.playbackState == Player.STATE_ENDED ||
            currentPlayer.playbackState == Player.STATE_IDLE) {
            Log.d(TAG, "Task removed - stopping service (not playing)")
            stopSelf()
        } else {
            Log.d(TAG, "Task removed - continuing playback in background")
        }
        super.onTaskRemoved(rootIntent)
    }

    private fun initializePlayer() {
        val okHttpClient = OkHttpClient.Builder()
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .build()

        val dataSourceFactory = OkHttpDataSource.Factory(okHttpClient)
            .setDefaultRequestProperties(mapOf())

        player = ExoPlayer.Builder(this)
            .setMediaSourceFactory(DefaultMediaSourceFactory(dataSourceFactory))
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(C.USAGE_MEDIA)
                    .setContentType(C.AUDIO_CONTENT_TYPE_MUSIC)
                    .build(),
                true // handleAudioFocus
            )
            .setHandleAudioBecomingNoisy(true)
            .build()
            .also { exoPlayer ->
                exoPlayer.addListener(playerListener)

                // Create a ForwardingPlayer to intercept media button commands
                // and forward them to the WebView for queue management
                val forwardingPlayer = object : ForwardingPlayer(exoPlayer) {
                    override fun seekToNext() {
                        // Don't let ExoPlayer handle it - emit to WebView instead
                        // The frontend manages the queue and will call play() with next track
                        Log.d(TAG, "Media button: Next - forwarding to WebView")
                        AudioPlayerBridge.emitMediaNext()
                    }

                    override fun seekToPrevious() {
                        // Don't let ExoPlayer handle it - emit to WebView instead
                        Log.d(TAG, "Media button: Previous - forwarding to WebView")
                        AudioPlayerBridge.emitMediaPrevious()
                    }

                    override fun stop() {
                        // Let ExoPlayer handle stop, but also notify WebView
                        super.stop()
                        Log.d(TAG, "Media button: Stop - forwarding to WebView")
                        AudioPlayerBridge.emitMediaStop()
                    }

                    // Always report that we have next/previous available
                    // since the frontend manages the queue
                    override fun getAvailableCommands(): Player.Commands {
                        return super.getAvailableCommands().buildUpon()
                            .add(Player.COMMAND_SEEK_TO_NEXT)
                            .add(Player.COMMAND_SEEK_TO_PREVIOUS)
                            .build()
                    }
                }

                val sessionBuilder = MediaSession.Builder(this, forwardingPlayer)
                createContentIntent()?.let { sessionBuilder.setSessionActivity(it) }
                
                // Add callback for media button handling
                sessionBuilder.setCallback(object : MediaSession.Callback {
                    override fun onConnect(
                        session: MediaSession,
                        controller: MediaSession.ControllerInfo
                    ): MediaSession.ConnectionResult {
                        // Accept all connections (system media controls, etc.)
                        return MediaSession.ConnectionResult.AcceptedResultBuilder(session)
                            .setAvailablePlayerCommands(
                                MediaSession.ConnectionResult.DEFAULT_PLAYER_COMMANDS
                            )
                            .setAvailableSessionCommands(
                                MediaSession.ConnectionResult.DEFAULT_SESSION_COMMANDS
                            )
                            .build()
                    }
                })
                
                mediaSession = sessionBuilder.build()
                
                // Register the session with MediaSessionService so it manages notifications
                addSession(mediaSession!!)
            }

        Log.d(TAG, "ExoPlayer initialized with ForwardingPlayer for media button interception")
    }

    private val playerListener = object : Player.Listener {
        override fun onPlaybackStateChanged(playbackState: Int) {
            val isFinished = playbackState == Player.STATE_ENDED
            if (isFinished) {
                AudioPlayerBridge.emitPosition(
                    player?.currentPosition?.toDouble()?.div(1000.0) ?: 0.0,
                    true
                )
            }
            updateNotification()
        }

        override fun onIsPlayingChanged(isPlaying: Boolean) {
            AudioPlayerBridge.emitPlaybackState(isPlaying)
            updateNotification()

            // Update visualizer state based on playback
            if (isPlaying && isVisualizerEnabled) {
                startVisualizer()
            } else {
                stopVisualizer()
            }
        }

        override fun onPlayerError(error: PlaybackException) {
            Log.e(TAG, "Player error: ${error.message}", error)
            AudioPlayerBridge.emitError(error.message ?: "Unknown playback error")
        }

        override fun onMediaItemTransition(mediaItem: MediaItem?, reason: Int) {
            // Gapless transition happened
            if (reason == Player.MEDIA_ITEM_TRANSITION_REASON_AUTO) {
                Log.d(TAG, "Gapless transition to next track")
            }
            updateNotification()
        }
    }

    // === Public API for Plugin ===

    fun play(url: String, token: String, title: String? = null, artist: String? = null, album: String? = null, artworkUrl: String? = null) {
        currentAuthToken = token
        
        val headers = mapOf(
            "Authorization" to "MediaBrowser Token=\"$token\"",
            "X-Emby-Token" to token
        )

        // Recreate data source factory with new headers
        val okHttpClient = OkHttpClient.Builder()
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .addInterceptor { chain ->
                val request = chain.request().newBuilder()
                headers.forEach { (key, value) -> request.addHeader(key, value) }
                chain.proceed(request.build())
            }
            .build()

        val dataSourceFactory = OkHttpDataSource.Factory(okHttpClient)

        // Build media item with metadata
        val metadataBuilder = MediaMetadata.Builder()
            .setTitle(title)
            .setArtist(artist)
            .setAlbumTitle(album)
        
        // Set artwork URI for notification (requires auth header, so we also load manually)
        if (artworkUrl != null) {
            metadataBuilder.setArtworkUri(Uri.parse(artworkUrl))
        }
        
        val mediaMetadata = metadataBuilder.build()

        val mediaItem = MediaItem.Builder()
            .setUri(url)
            .setMediaMetadata(mediaMetadata)
            .build()

        player?.let { exoPlayer ->
            // Update the media source factory
            val newSourceFactory = DefaultMediaSourceFactory(dataSourceFactory)

            exoPlayer.stop()
            exoPlayer.clearMediaItems()
            exoPlayer.setMediaSource(newSourceFactory.createMediaSource(mediaItem))
            exoPlayer.prepare()
            exoPlayer.play()

            // Load artwork in background
            if (artworkUrl != null && artworkUrl != currentArtworkUrl) {
                loadArtwork(artworkUrl, token)
            }

            // Initialize EQ if not already done
            initializeEqualizer()
        }

        Log.d(TAG, "Playing: $url")
    }

    fun pause() {
        player?.pause()
        Log.d(TAG, "Paused")
    }

    fun resume() {
        player?.play()
        Log.d(TAG, "Resumed")
    }

    fun stop() {
        player?.stop()
        player?.clearMediaItems()
        Log.d(TAG, "Stopped")
    }

    fun setVolume(volume: Float) {
        player?.volume = volume.coerceIn(0f, 1f)
        Log.d(TAG, "Volume set to $volume")
    }

    fun getVolume(): Float {
        return player?.volume ?: 1f
    }

    fun isPlaying(): Boolean {
        return player?.isPlaying ?: false
    }

    fun isFinished(): Boolean {
        return player?.playbackState == Player.STATE_ENDED
    }

    fun getPosition(): Double {
        return (player?.currentPosition?.toDouble() ?: 0.0) / 1000.0
    }

    fun seek(positionSecs: Double) {
        player?.seekTo((positionSecs * 1000).toLong())
        Log.d(TAG, "Seeked to $positionSecs seconds")
    }

    fun prepareNext(url: String, token: String) {
        preparedNextUrl = url
        preparedNextToken = token

        val headers = mapOf(
            "Authorization" to "MediaBrowser Token=\"$token\"",
            "X-Emby-Token" to token
        )

        player?.let { exoPlayer ->
            if (exoPlayer.mediaItemCount > 1) {
                // Remove any previously queued items (keep current)
                while (exoPlayer.mediaItemCount > 1) {
                    exoPlayer.removeMediaItem(1)
                }
            }

            val mediaItem = MediaItem.Builder()
                .setUri(url)
                .build()

            exoPlayer.addMediaItem(mediaItem)
            Log.d(TAG, "Prepared next track: $url")
        }
    }

    fun advanceGapless(): Boolean {
        player?.let { exoPlayer ->
            if (exoPlayer.hasNextMediaItem()) {
                exoPlayer.seekToNextMediaItem()
                Log.d(TAG, "Advanced to next track (gapless)")
                return true
            }
        }
        return false
    }

    // === Equalizer ===

    private fun initializeEqualizer() {
        if (equalizer != null) return

        val audioSessionId = player?.audioSessionId ?: return
        if (audioSessionId == C.AUDIO_SESSION_ID_UNSET) return

        try {
            equalizer = Equalizer(0, audioSessionId).also { eq ->
                // Build band mapping
                val numBands = eq.numberOfBands.toInt()
                eqBandMapping = IntArray(5) { conceptualBand ->
                    val targetFreq = targetFrequencies[conceptualBand]
                    var closestBand = 0
                    var closestDiff = Int.MAX_VALUE

                    for (band in 0 until numBands) {
                        val centerFreq = eq.getCenterFreq(band.toShort()) / 1000 // Hz
                        val diff = kotlin.math.abs(centerFreq - targetFreq)
                        if (diff < closestDiff) {
                            closestDiff = diff
                            closestBand = band
                        }
                    }
                    closestBand
                }

                Log.d(TAG, "Equalizer initialized with ${numBands} bands, mapping: ${eqBandMapping?.contentToString()}")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize equalizer", e)
        }
    }

    private fun releaseEqualizer() {
        equalizer?.release()
        equalizer = null
    }

    fun setEqEnabled(enabled: Boolean) {
        equalizer?.enabled = enabled
        Log.d(TAG, "EQ enabled: $enabled")
    }

    fun isEqEnabled(): Boolean {
        return equalizer?.enabled ?: false
    }

    fun setEqBand(band: Int, gainDb: Float) {
        val eq = equalizer ?: return
        val mapping = eqBandMapping ?: return
        if (band < 0 || band >= 5) return

        val androidBand = mapping[band].toShort()
        // Convert dB to millibels (-20dB to +20dB -> -2000 to +2000 millibels)
        val minLevel = eq.bandLevelRange[0]
        val maxLevel = eq.bandLevelRange[1]
        val millibels = (gainDb * 100).toInt().toShort().coerceIn(minLevel, maxLevel)

        eq.setBandLevel(androidBand, millibels)
        Log.d(TAG, "EQ band $band (Android band $androidBand) set to $gainDb dB ($millibels mB)")
    }

    fun getEqBand(band: Int): Float {
        val eq = equalizer ?: return 0f
        val mapping = eqBandMapping ?: return 0f
        if (band < 0 || band >= 5) return 0f

        val androidBand = mapping[band].toShort()
        val millibels = eq.getBandLevel(androidBand)
        return millibels / 100f
    }

    fun getAllEqBands(): FloatArray {
        return FloatArray(5) { band -> getEqBand(band) }
    }

    fun resetEq() {
        for (band in 0 until 5) {
            setEqBand(band, 0f)
        }
        Log.d(TAG, "EQ reset to flat")
    }

    // === Visualizer ===

    fun setAnalyzerEnabled(enabled: Boolean) {
        isVisualizerEnabled = enabled
        Log.d(TAG, "setAnalyzerEnabled: enabled=$enabled, player?.isPlaying=${player?.isPlaying}, visualizerFailed=$visualizerFailed")
        if (enabled) {
            // Reset failure flag when explicitly enabling (user may have granted permission)
            visualizerFailed = false
            if (player?.isPlaying == true) {
                Log.d(TAG, "setAnalyzerEnabled: calling startVisualizer()")
                startVisualizer()
            } else {
                Log.d(TAG, "setAnalyzerEnabled: deferred - player not playing yet")
            }
        } else {
            stopVisualizer()
        }
    }

    fun isAnalyzerEnabled(): Boolean {
        return isVisualizerEnabled
    }

    private fun startVisualizer() {
        Log.d(TAG, "startVisualizer: visualizer=${visualizer != null}, visualizerFailed=$visualizerFailed")
        if (visualizer != null) {
            Log.d(TAG, "startVisualizer: already running")
            return
        }
        if (visualizerFailed) {
            Log.d(TAG, "startVisualizer: previously failed, not retrying")
            return
        }

        val exoPlayer = player
        if (exoPlayer == null) {
            Log.d(TAG, "startVisualizer: player is null")
            return
        }
        val audioSessionId = exoPlayer.audioSessionId
        Log.d(TAG, "startVisualizer: audioSessionId=$audioSessionId, playbackState=${exoPlayer.playbackState}, isPlaying=${exoPlayer.isPlaying}")
        
        // Validate audio session
        if (audioSessionId == C.AUDIO_SESSION_ID_UNSET) {
            Log.d(TAG, "Visualizer: Audio session not ready yet")
            return
        }
        
        // Ensure player is actually ready and playing
        if (exoPlayer.playbackState != Player.STATE_READY || !exoPlayer.isPlaying) {
            Log.d(TAG, "Visualizer: Player not ready/playing, deferring")
            return
        }

        // Delay visualizer creation slightly to ensure audio output is fully initialized
        Log.d(TAG, "startVisualizer: scheduling delayed visualizer creation")
        mainHandler.postDelayed({
            Log.d(TAG, "startVisualizer delayed: visualizer=${visualizer != null}, visualizerFailed=$visualizerFailed, isVisualizerEnabled=$isVisualizerEnabled")
            if (visualizer != null || visualizerFailed) {
                Log.d(TAG, "startVisualizer delayed: bailing - already exists or failed")
                return@postDelayed
            }
            if (!isVisualizerEnabled) {
                Log.d(TAG, "startVisualizer delayed: bailing - not enabled")
                return@postDelayed
            }
            
            val currentSessionId = player?.audioSessionId
            Log.d(TAG, "startVisualizer delayed: currentSessionId=$currentSessionId")
            if (currentSessionId == null || currentSessionId == C.AUDIO_SESSION_ID_UNSET) {
                Log.d(TAG, "startVisualizer delayed: bailing - no valid session ID")
                return@postDelayed
            }
            
            try {
                visualizer = Visualizer(currentSessionId).also { viz ->
                    viz.captureSize = Visualizer.getCaptureSizeRange()[1].coerceAtMost(256)
                    viz.setDataCaptureListener(
                        object : Visualizer.OnDataCaptureListener {
                            override fun onWaveFormDataCapture(
                                visualizer: Visualizer?,
                                waveform: ByteArray?,
                                samplingRate: Int
                            ) {
                                // We'll emit both in onFftDataCapture
                            }

                            override fun onFftDataCapture(
                                visualizer: Visualizer?,
                                fft: ByteArray?,
                                samplingRate: Int
                            ) {
                                // Use the callback-provided visualizer with safety checks
                                // to avoid race conditions when visualizer is released
                                if (fft == null || !isVisualizerEnabled || visualizer == null) return
                                
                                try {
                                    // Check visualizer state before accessing properties
                                    // State 1 = STATE_INITIALIZED, State 2 = STATE_ENABLED
                                    if (!visualizer.enabled) return
                                    
                                    val captureSize = visualizer.captureSize
                                    val waveform = ByteArray(captureSize)
                                    visualizer.getWaveForm(waveform)

                                    // Convert FFT to frequency magnitudes
                                    val frequencyData = processFFT(fft)
                                    spectrumEmitCount++
                                    if (spectrumEmitCount <= 3 || spectrumEmitCount % 100 == 0) {
                                        Log.d(TAG, "Emitting spectrum data #$spectrumEmitCount, freqData size=${frequencyData.size}")
                                    }
                                    AudioPlayerBridge.emitSpectrum(frequencyData, waveform)
                                } catch (e: IllegalStateException) {
                                    // Visualizer was released between check and use - ignore
                                    Log.d(TAG, "Visualizer callback ignored - visualizer released")
                                }
                            }
                        },
                        Visualizer.getMaxCaptureRate(), // Max rate - JS polls at its own frame rate
                        true,  // waveform
                        true   // fft
                    )
                    viz.enabled = true
                    Log.d(TAG, "Visualizer started with session ID: $currentSessionId")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start visualizer: ${e.message}", e)
                visualizerFailed = true  // Don't retry
            }
        }, 200)  // 200ms delay for audio output to stabilize
    }

    private fun stopVisualizer() {
        visualizer?.enabled = false
        visualizer?.release()
        visualizer = null
        spectrumEmitCount = 0
        Log.d(TAG, "Visualizer stopped")
    }

    private fun releaseVisualizer() {
        stopVisualizer()
    }

    private fun processFFT(fft: ByteArray): ByteArray {
        // FFT data is in pairs: [real0, imag0, real1, imag1, ...]
        // Convert to magnitude spectrum
        val n = fft.size / 2
        val magnitudes = ByteArray(n)

        for (i in 0 until n) {
            val real = fft[i * 2].toFloat()
            val imag = fft[i * 2 + 1].toFloat()
            val magnitude = kotlin.math.sqrt(real * real + imag * imag)
            // Normalize to 0-255 range
            magnitudes[i] = (magnitude.coerceIn(0f, 255f)).toInt().toByte()
        }

        return magnitudes
    }

    // === Position Updates ===

    private fun startPositionUpdates() {
        positionUpdateRunnable = object : Runnable {
            override fun run() {
                player?.let { exoPlayer ->
                    if (exoPlayer.isPlaying) {
                        val position = exoPlayer.currentPosition.toDouble() / 1000.0
                        val isFinished = exoPlayer.playbackState == Player.STATE_ENDED
                        AudioPlayerBridge.emitPosition(position, isFinished)
                    }
                }
                mainHandler.postDelayed(this, POSITION_UPDATE_INTERVAL_MS)
            }
        }
        mainHandler.post(positionUpdateRunnable!!)
    }

    private fun stopPositionUpdates() {
        positionUpdateRunnable?.let { mainHandler.removeCallbacks(it) }
        positionUpdateRunnable = null
    }

    // === Notification ===

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                getString(R.string.notification_channel_playback),
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = getString(R.string.notification_channel_playback_description)
                setShowBadge(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager?.createNotificationChannel(channel)
        }
    }

    private fun createContentIntent(): PendingIntent? {
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName) ?: return null
        launchIntent.flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
        val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        return PendingIntent.getActivity(this, 0, launchIntent, flags)
    }

    private fun updateNotification() {
        // MediaSessionService handles notification automatically via MediaSession
        // This is called for manual updates if needed
    }

    private fun loadArtwork(url: String, token: String) {
        currentArtworkUrl = url
        executor.execute {
            try {
                val connection = URL(url).openConnection().apply {
                    setRequestProperty("Authorization", "MediaBrowser Token=\"$token\"")
                    setRequestProperty("X-Emby-Token", token)
                    connectTimeout = 5000
                    readTimeout = 5000
                }
                val bitmap = connection.getInputStream().use { BitmapFactory.decodeStream(it) }
                currentArtwork = bitmap
                
                // Update MediaItem metadata with artwork on main thread
                mainHandler.post {
                    updateMediaMetadataWithArtwork(bitmap)
                }
            } catch (e: Exception) {
                Log.w(TAG, "Failed to load artwork", e)
            }
        }
    }
    
    private fun updateMediaMetadataWithArtwork(bitmap: Bitmap) {
        player?.let { exoPlayer ->
            val currentItem = exoPlayer.currentMediaItem ?: return
            val currentMetadata = currentItem.mediaMetadata
            
            // Build new metadata with artwork
            val newMetadata = currentMetadata.buildUpon()
                .setArtworkData(
                    bitmapToByteArray(bitmap),
                    MediaMetadata.PICTURE_TYPE_FRONT_COVER
                )
                .build()
            
            // Create new MediaItem with updated metadata
            val newItem = currentItem.buildUpon()
                .setMediaMetadata(newMetadata)
                .build()
            
            // Replace current item (this triggers notification update)
            val currentIndex = exoPlayer.currentMediaItemIndex
            val currentPosition = exoPlayer.currentPosition
            val wasPlaying = exoPlayer.isPlaying
            
            exoPlayer.replaceMediaItem(currentIndex, newItem)
            exoPlayer.seekTo(currentIndex, currentPosition)
            if (wasPlaying) {
                exoPlayer.play()
            }
            
            Log.d(TAG, "Updated media metadata with artwork")
        }
    }
    
    private fun bitmapToByteArray(bitmap: Bitmap): ByteArray {
        val stream = java.io.ByteArrayOutputStream()
        bitmap.compress(Bitmap.CompressFormat.JPEG, 90, stream)
        return stream.toByteArray()
    }
}
