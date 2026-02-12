package com.aurelia.app.audio

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.audiofx.Visualizer
import android.os.Handler
import android.os.Looper
import android.util.Log
import androidx.core.content.ContextCompat
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

data class VisualizerState(
    val enabled: Boolean = false,
    val available: Boolean = false,
    val hasPermission: Boolean = false,
    val frequencyData: List<Float> = emptyList(),
    val waveform: List<Float> = emptyList(),
)

enum class VisualizerStyle {
    BARS,
    CURVE,
    WAVE,
}

class VisualizerManager(
    private val context: Context,
    private val audioSessionId: Int,
) {
    private var visualizer: Visualizer? = null
    private val _state = MutableStateFlow(VisualizerState())
    val state: StateFlow<VisualizerState> = _state.asStateFlow()

    private val mainHandler = Handler(Looper.getMainLooper())
    private var captureRunnable: Runnable? = null

    private var smoothedFrequency = FloatArray(0)
    private var smoothedWaveform = FloatArray(0)
    private var waveBuffer = ByteArray(0)
    private var freqBuffer = ByteArray(0)

    private val attackSmoothing = 0.8f
    private val decaySmoothing = 0.15f
    private var isReleased = false

    // Propagates every state change to AudioManager so its StateFlow stays in sync
    var stateCallback: ((VisualizerState) -> Unit)? = null

    private fun setState(newState: VisualizerState) {
        _state.value = newState
        stateCallback?.invoke(newState)
    }

    init {
        val hasPermission = ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.RECORD_AUDIO
        ) == PackageManager.PERMISSION_GRANTED

        setState(_state.value.copy(hasPermission = hasPermission))
        Log.d(TAG, "VisualizerManager initialized for session $audioSessionId, permission=$hasPermission")
    }

    fun setEnabled(enabled: Boolean) {
        if (isReleased) {
            Log.w(TAG, "Cannot enable visualizer - already released")
            return
        }

        if (enabled && !_state.value.hasPermission) {
            Log.w(TAG, "Cannot enable visualizer - missing RECORD_AUDIO permission")
            return
        }

        // All visualizer operations must happen on main thread
        if (Looper.myLooper() != Looper.getMainLooper()) {
            mainHandler.post { setEnabled(enabled) }
            return
        }

        if (enabled) {
            // Delay visualizer creation to ensure audio session is stable
            // This prevents crashes from race conditions with Media3/ExoPlayer
            mainHandler.postDelayed({
                if (isReleased) return@postDelayed
                
                // Create visualizer lazily only when enabled
                if (visualizer == null) {
                    if (!createVisualizer()) {
                        Log.w(TAG, "Cannot enable visualizer - creation failed")
                        setState(_state.value.copy(enabled = false))
                        return@postDelayed
                    }
                }

                val viz = visualizer
                if (viz == null) {
                    Log.w(TAG, "Cannot enable visualizer - visualizer is null after creation")
                    setState(_state.value.copy(enabled = false))
                    return@postDelayed
                }

                try {
                    viz.enabled = true
                    setState(_state.value.copy(enabled = true))
                    startCapture()
                    Log.d(TAG, "Visualizer enabled successfully")
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to enable visualizer", e)
                    setState(_state.value.copy(enabled = false))
                }
            }, 500) // 500ms delay to let player stabilize
        } else {
            stopCapture()
            try {
                visualizer?.enabled = false
            } catch (e: Exception) {
                Log.e(TAG, "Error disabling visualizer", e)
            }
            setState(_state.value.copy(enabled = false))
            Log.d(TAG, "Visualizer disabled")
        }
    }

    private fun createVisualizer(): Boolean {
        if (isReleased) {
            Log.w(TAG, "Cannot create visualizer - already released")
            return false
        }
        
        // Check if audio session is ready (0 means not ready yet)
        if (audioSessionId == 0) {
            Log.w(TAG, "Cannot create visualizer - audio session ID is 0 (not ready yet)")
            return false
        }
        
        return try {
            Log.d(TAG, "Creating visualizer with audio session $audioSessionId")
            
            visualizer = Visualizer(audioSessionId).apply {
                // Use a smaller capture size for better stability
                val range = Visualizer.getCaptureSizeRange()
                captureSize = range[0] // Use minimum size for stability
                enabled = false
            }
            
            val captureSize = visualizer?.captureSize ?: 0
            if (captureSize <= 0) {
                Log.e(TAG, "Failed to create visualizer - invalid capture size: $captureSize")
                visualizer?.release()
                visualizer = null
                return false
            }
            
            // Initialize buffers based on actual capture size
            initializeBuffers(captureSize)
            
            setState(_state.value.copy(available = true))
            Log.d(TAG, "Visualizer created successfully with capture size $captureSize")
            Log.d(TAG, "Buffers initialized - waveBuffer: ${waveBuffer.size}, freqBuffer: ${freqBuffer.size}")
            Log.d(TAG, "Smoothed arrays - waveform: ${smoothedWaveform.size}, frequency: ${smoothedFrequency.size}")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to create visualizer for session $audioSessionId", e)
            setState(_state.value.copy(available = false))
            false
        }
    }
    
    private fun initializeBuffers(captureSize: Int) {
        // waveBuffer needs captureSize bytes
        waveBuffer = ByteArray(captureSize)

        // getFft() writes captureSize bytes (not captureSize/2+1).
        // Using a smaller buffer causes a heap overflow in native code, which
        // manifests as an MTE tag mismatch SIGSEGV on Android 14+ with MTE enabled.
        freqBuffer = ByteArray(captureSize)

        // smoothedWaveform matches waveBuffer size
        smoothedWaveform = FloatArray(captureSize)

        // smoothedFrequency matches freqBuffer size
        smoothedFrequency = FloatArray(captureSize)
        
        Log.d(TAG, "Initialized buffers for capture size $captureSize:")
        Log.d(TAG, "  waveBuffer size: ${waveBuffer.size}")
        Log.d(TAG, "  freqBuffer size: ${freqBuffer.size}")
        Log.d(TAG, "  smoothedWaveform size: ${smoothedWaveform.size}")
        Log.d(TAG, "  smoothedFrequency size: ${smoothedFrequency.size}")
    }

    private fun startCapture() {
        if (isReleased) {
            Log.w(TAG, "Cannot start capture - already released")
            return
        }
        
        stopCapture()
        
        val viz = visualizer
        if (viz == null) {
            Log.w(TAG, "Cannot start capture - visualizer is null")
            return
        }
        
        // Check if visualizer is actually enabled before starting
        if (!viz.enabled) {
            Log.w(TAG, "Cannot start capture - visualizer not enabled")
            return
        }
        
        // Verify buffers are initialized
        if (waveBuffer.isEmpty() || freqBuffer.isEmpty()) {
            Log.w(TAG, "Cannot start capture - buffers not initialized")
            return
        }
        
        Log.d(TAG, "Starting capture with buffer sizes - wave: ${waveBuffer.size}, freq: ${freqBuffer.size}")
        
        captureRunnable = object : Runnable {
            override fun run() {
                if (isReleased) {
                    Log.d(TAG, "Capture stopped - VisualizerManager released")
                    return
                }
                
                val currentViz = visualizer
                if (currentViz == null) {
                    Log.d(TAG, "Capture stopped - visualizer is null")
                    return
                }
                
                if (!currentViz.enabled) {
                    Log.d(TAG, "Capture stopped - visualizer disabled")
                    return
                }
                
                try {
                    // Get FFT data (frequency spectrum)
                    val fftResult = currentViz.getFft(freqBuffer)
                    if (fftResult != Visualizer.SUCCESS) {
                        // Don't crash, just skip this frame
                        mainHandler.postDelayed(this, 16)
                        return
                    }
                    
                    // Get waveform data
                    val waveResult = currentViz.getWaveForm(waveBuffer)
                    if (waveResult != Visualizer.SUCCESS) {
                        // Don't crash, just skip this frame
                        mainHandler.postDelayed(this, 16)
                        return
                    }
                    
                    applySmoothing(freqBuffer, waveBuffer)

                    setState(_state.value.copy(
                        frequencyData = smoothedFrequency.toList(),
                        waveform = smoothedWaveform.toList()
                    ))
                } catch (e: IllegalStateException) {
                    Log.e(TAG, "Visualizer not initialized", e)
                    return
                } catch (e: Exception) {
                    Log.e(TAG, "Capture error", e)
                    return
                }
                
                // Schedule next frame (60fps)
                mainHandler.postDelayed(this, 16)
            }
        }
        
        mainHandler.post(captureRunnable!!)
        Log.d(TAG, "Capture started")
    }

    private fun applySmoothing(rawFreq: ByteArray, rawWave: ByteArray) {
        // Ensure we don't go out of bounds
        val freqSize = minOf(rawFreq.size, smoothedFrequency.size)
        val waveSize = minOf(rawWave.size, smoothedWaveform.size)
        
        for (i in 0 until freqSize) {
            val raw = rawFreq[i].toInt() and 0xFF
            val current = smoothedFrequency[i]
            val rate = if (raw > current) attackSmoothing else decaySmoothing
            smoothedFrequency[i] = current + (raw - current) * rate
        }

        for (i in 0 until waveSize) {
            val raw = rawWave[i].toInt() and 0xFF
            val current = smoothedWaveform[i]
            val rate = if (raw > current) attackSmoothing else decaySmoothing
            smoothedWaveform[i] = current + (raw - current) * rate
        }
    }

    private fun stopCapture() {
        captureRunnable?.let { 
            mainHandler.removeCallbacks(it) 
        }
        captureRunnable = null
        smoothedFrequency.fill(0f)
        smoothedWaveform.fill(128f)
        setState(_state.value.copy(
            frequencyData = emptyList(),
            waveform = emptyList()
        ))
        Log.d(TAG, "Capture stopped")
    }

    fun updatePermissionStatus(hasPermission: Boolean) {
        setState(_state.value.copy(hasPermission = hasPermission))
    }

    fun release() {
        if (Looper.myLooper() != Looper.getMainLooper()) {
            mainHandler.post { release() }
            return
        }
        
        Log.d(TAG, "Releasing VisualizerManager for session $audioSessionId")
        isReleased = true
        stopCapture()
        
        try {
            visualizer?.enabled = false
            visualizer?.release()
            visualizer = null
            Log.d(TAG, "Visualizer released successfully")
        } catch (e: Exception) {
            Log.e(TAG, "Error releasing visualizer", e)
        }
    }

    companion object {
        private const val TAG = "VisualizerManager"
    }
}
