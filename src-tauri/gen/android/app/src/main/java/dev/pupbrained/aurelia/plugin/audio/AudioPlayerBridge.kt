package dev.pupbrained.aurelia.plugin.audio

import android.os.Handler
import android.os.Looper
import java.util.concurrent.atomic.AtomicReference

/**
 * Thread-safe bridge for emitting audio events to the WebView.
 * Similar pattern to NowPlayingBridge but for audio playback events.
 */
object AudioPlayerBridge {
    private val mainHandler = Handler(Looper.getMainLooper())
    private val listenerRef = AtomicReference<AudioEventListener?>(null)

    interface AudioEventListener {
        fun onPositionUpdate(position: Double, isFinished: Boolean)
        fun onSpectrumData(frequencyData: ByteArray, timeDomainData: ByteArray)
        fun onPlaybackStateChanged(isPlaying: Boolean)
        fun onError(message: String)
        // Media control events from notification/quick settings
        fun onMediaNext()
        fun onMediaPrevious()
        fun onMediaStop()
    }

    fun setListener(listener: AudioEventListener?) {
        listenerRef.set(listener)
    }

    fun emitPosition(position: Double, isFinished: Boolean) {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onPositionUpdate(position, isFinished)
        }
    }

    fun emitSpectrum(frequencyData: ByteArray, timeDomainData: ByteArray) {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onSpectrumData(frequencyData, timeDomainData)
        }
    }

    fun emitPlaybackState(isPlaying: Boolean) {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onPlaybackStateChanged(isPlaying)
        }
    }

    fun emitError(message: String) {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onError(message)
        }
    }

    fun emitMediaNext() {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onMediaNext()
        }
    }

    fun emitMediaPrevious() {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onMediaPrevious()
        }
    }

    fun emitMediaStop() {
        val listener = listenerRef.get() ?: return
        mainHandler.post {
            listener.onMediaStop()
        }
    }
}
