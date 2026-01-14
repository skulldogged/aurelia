package dev.pupbrained.aurelia.plugin.audio

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.util.Log
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONArray

@InvokeArg
class AudioPlayPayload {
    var url: String = ""
    var token: String = ""
    var title: String? = null
    var artist: String? = null
    var album: String? = null
    var artworkUrl: String? = null
}

@InvokeArg
class AudioPrepareNextPayload {
    var url: String = ""
    var token: String = ""
}

@InvokeArg
class AudioSeekPayload {
    var positionSecs: Double = 0.0
}

@InvokeArg
class AudioVolumePayload {
    var volume: Float = 1.0f
}

@InvokeArg
class AudioEqBandPayload {
    var band: Int = 0
    var gainDb: Float = 0.0f
}

@InvokeArg
class AudioEqBandGetPayload {
    var band: Int = 0
}

@InvokeArg
class AudioBoolPayload {
    var enabled: Boolean = false
}

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.RECORD_AUDIO], alias = "recordAudio")
    ]
)
class AudioPlayerPlugin(private val activity: Activity) : Plugin(activity) {
    private var webView: WebView? = null
    private var serviceStarted = false
    private var pendingPermissionInvoke: Invoke? = null
    
    // Spectrum data buffer for high-performance JS polling
    // Using volatile for thread-safe reads without synchronization overhead
    @Volatile private var latestFrequencyData: ByteArray? = null
    @Volatile private var latestTimeDomainData: ByteArray? = null
    @Volatile private var spectrumDataVersion: Long = 0

    companion object {
        private const val TAG = "AudioPlayerPlugin"
        private const val RECORD_AUDIO_REQUEST_CODE = 1001
        
        // Static reference for permission callback
        private var pluginInstance: AudioPlayerPlugin? = null
        
        fun handlePermissionResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
            pluginInstance?.onRequestPermissionsResult(requestCode, permissions, grantResults)
        }
    }
    
    init {
        pluginInstance = this
    }

    override fun load(webView: WebView) {
        super.load(webView)
        this.webView = webView
        
        // Add JavaScript interface for high-performance spectrum polling
        webView.addJavascriptInterface(SpectrumInterface(), "AureliaSpectrum")
        
        setupBridgeListener()
    }
    
    /**
     * JavaScript interface for high-performance spectrum data polling.
     * JS calls this at requestAnimationFrame rate for smooth visualizations.
     */
    inner class SpectrumInterface {
        /**
         * Get latest spectrum data as comma-separated values.
         * Format: "version,freq0,freq1,...,freqN|time0,time1,...,timeN"
         * Returns empty string if no data available.
         */
        @JavascriptInterface
        fun getData(): String {
            val freq = latestFrequencyData ?: return ""
            val time = latestTimeDomainData ?: return ""
            
            // Pre-allocate StringBuilder for performance
            // Format: version,freq_values|time_values
            val sb = StringBuilder(freq.size * 4 + time.size * 4 + 20)
            
            sb.append(spectrumDataVersion).append(',')
            
            // Frequency data
            for (i in freq.indices) {
                if (i > 0) sb.append(',')
                sb.append(freq[i].toInt() and 0xFF)
            }
            
            sb.append('|')
            
            // Time domain data
            for (i in time.indices) {
                if (i > 0) sb.append(',')
                sb.append(time[i].toInt() and 0xFF)
            }
            
            return sb.toString()
        }
        
        /**
         * Get current data version for change detection.
         * JS can skip parsing if version hasn't changed.
         */
        @JavascriptInterface
        fun getVersion(): Long {
            return spectrumDataVersion
        }
    }

    private fun setupBridgeListener() {
        AudioPlayerBridge.setListener(object : AudioPlayerBridge.AudioEventListener {
            override fun onPositionUpdate(position: Double, isFinished: Boolean) {
                emitEvent("audio:position", JSObject().apply {
                    put("position", position)
                    put("isFinished", isFinished)
                })
            }

            override fun onSpectrumData(frequencyData: ByteArray, timeDomainData: ByteArray) {
                // Store in buffer for JS polling - no JS calls needed!
                latestFrequencyData = frequencyData
                latestTimeDomainData = timeDomainData
                spectrumDataVersion++
            }

            override fun onPlaybackStateChanged(isPlaying: Boolean) {
                emitEvent("audio:playback-state", JSObject().apply {
                    put("isPlaying", isPlaying)
                })
            }

            override fun onError(message: String) {
                emitEvent("audio:error", JSObject().apply {
                    put("message", message)
                })
            }

            override fun onMediaNext() {
                // Emit same event name as desktop for frontend compatibility
                emitEvent("media:next", JSObject())
            }

            override fun onMediaPrevious() {
                // Emit same event name as desktop for frontend compatibility
                emitEvent("media:previous", JSObject())
            }

            override fun onMediaStop() {
                // Emit same event name as desktop for frontend compatibility
                emitEvent("media:stop", JSObject())
            }
        })
    }

    private fun emitEvent(eventName: String, data: JSObject) {
        val jsonData = data.toString().replace("'", "\\'").replace("\n", "")
        
        // Use Tauri's internal event system to emit events to the frontend
        val script = """
            (function() {
                try {
                    if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                        window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {
                            event: '$eventName',
                            payload: $jsonData
                        }).catch(function(e) { console.error('Event emit failed:', e); });
                    }
                } catch(e) {
                    console.error('Failed to emit event $eventName:', e);
                }
            })();
        """.trimIndent()

        activity.runOnUiThread {
            webView?.evaluateJavascript(script, null)
        }
    }

    private fun ensureServiceStarted() {
        if (!serviceStarted) {
            val intent = Intent(activity, AudioPlayerService::class.java)
            ContextCompat.startForegroundService(activity, intent)
            serviceStarted = true
            Log.d(TAG, "AudioPlayerService started")
        }
    }

    private fun getService(): AudioPlayerService? {
        return AudioPlayerService.getInstance()
    }

    @Command
    fun audioInit(invoke: Invoke) {
        ensureServiceStarted()
        // Give service time to initialize
        activity.window.decorView.postDelayed({
            invoke.resolve(JSObject())
        }, 100)
    }

    @Command
    fun audioPlay(invoke: Invoke) {
        ensureServiceStarted()
        val args = invoke.parseArgs(AudioPlayPayload::class.java)

        if (args.url.isBlank()) {
            invoke.reject("URL is required")
            return
        }

        // Wait for service to be available
        activity.window.decorView.postDelayed({
            getService()?.play(
                url = args.url,
                token = args.token,
                title = args.title,
                artist = args.artist,
                album = args.album,
                artworkUrl = args.artworkUrl
            )
            invoke.resolve(JSObject())
        }, if (serviceStarted) 0 else 200)
    }

    @Command
    fun audioPause(invoke: Invoke) {
        getService()?.pause()
        invoke.resolve(JSObject())
    }

    @Command
    fun audioResume(invoke: Invoke) {
        getService()?.resume()
        invoke.resolve(JSObject())
    }

    @Command
    fun audioStop(invoke: Invoke) {
        getService()?.stop()
        invoke.resolve(JSObject())
    }

    @Command
    fun audioSetVolume(invoke: Invoke) {
        val args = invoke.parseArgs(AudioVolumePayload::class.java)
        getService()?.setVolume(args.volume)
        invoke.resolve(JSObject())
    }

    @Command
    fun audioGetVolume(invoke: Invoke) {
        val volume = getService()?.getVolume() ?: 1.0f
        invoke.resolve(JSObject().apply {
            put("value", volume)
        })
    }

    @Command
    fun audioIsPlaying(invoke: Invoke) {
        val isPlaying = getService()?.isPlaying() ?: false
        invoke.resolve(JSObject().apply {
            put("value", isPlaying)
        })
    }

    @Command
    fun audioIsFinished(invoke: Invoke) {
        val isFinished = getService()?.isFinished() ?: true
        invoke.resolve(JSObject().apply {
            put("value", isFinished)
        })
    }

    @Command
    fun audioGetPosition(invoke: Invoke) {
        val position = getService()?.getPosition() ?: 0.0
        invoke.resolve(JSObject().apply {
            put("value", position)
        })
    }

    @Command
    fun audioSeek(invoke: Invoke) {
        val args = invoke.parseArgs(AudioSeekPayload::class.java)
        getService()?.seek(args.positionSecs)
        invoke.resolve(JSObject())
    }

    @Command
    fun audioPrepareNext(invoke: Invoke) {
        val args = invoke.parseArgs(AudioPrepareNextPayload::class.java)
        getService()?.prepareNext(args.url, args.token)
        invoke.resolve(JSObject())
    }

    @Command
    fun audioAdvanceGapless(invoke: Invoke) {
        val success = getService()?.advanceGapless() ?: false
        invoke.resolve(JSObject().apply {
            put("success", success)
        })
    }

    @Command
    fun audioSetEqEnabled(invoke: Invoke) {
        val args = invoke.parseArgs(AudioBoolPayload::class.java)
        getService()?.setEqEnabled(args.enabled)
        invoke.resolve(JSObject())
    }

    @Command
    fun audioIsEqEnabled(invoke: Invoke) {
        val enabled = getService()?.isEqEnabled() ?: false
        invoke.resolve(JSObject().apply {
            put("value", enabled)
        })
    }

    @Command
    fun audioSetEqBand(invoke: Invoke) {
        val args = invoke.parseArgs(AudioEqBandPayload::class.java)
        getService()?.setEqBand(args.band, args.gainDb)
        invoke.resolve(JSObject())
    }

    @Command
    fun audioGetEqBand(invoke: Invoke) {
        val args = invoke.parseArgs(AudioEqBandGetPayload::class.java)
        val gain = getService()?.getEqBand(args.band) ?: 0f
        invoke.resolve(JSObject().apply {
            put("value", gain)
        })
    }

    @Command
    fun audioGetAllEqBands(invoke: Invoke) {
        val bands = getService()?.getAllEqBands() ?: FloatArray(5) { 0f }
        invoke.resolve(JSObject().apply {
            put("value", bands.toList())
        })
    }

    @Command
    fun audioResetEq(invoke: Invoke) {
        getService()?.resetEq()
        invoke.resolve(JSObject())
    }

    @Command
    fun audioSetAnalyzerEnabled(invoke: Invoke) {
        val args = invoke.parseArgs(AudioBoolPayload::class.java)
        Log.d(TAG, "audioSetAnalyzerEnabled: enabled=${args.enabled}")
        
        val service = getService()
        if (service == null) {
            Log.w(TAG, "audioSetAnalyzerEnabled: service is null!")
            invoke.resolve(JSObject().apply {
                put("serviceUnavailable", true)
            })
            return
        }
        
        if (args.enabled) {
            // Check if we have RECORD_AUDIO permission (required for Visualizer)
            val hasPermission = ContextCompat.checkSelfPermission(
                activity, 
                Manifest.permission.RECORD_AUDIO
            ) == PackageManager.PERMISSION_GRANTED
            
            Log.d(TAG, "audioSetAnalyzerEnabled: hasPermission=$hasPermission")
            
            if (hasPermission) {
                service.setAnalyzerEnabled(true)
                invoke.resolve(JSObject())
            } else {
                // Permission not granted - resolve with flag so frontend knows
                Log.d(TAG, "RECORD_AUDIO permission not granted, cannot enable visualizer")
                invoke.resolve(JSObject().apply {
                    put("permissionDenied", true)
                })
            }
        } else {
            service.setAnalyzerEnabled(false)
            invoke.resolve(JSObject())
        }
    }

    @Command
    fun audioIsAnalyzerEnabled(invoke: Invoke) {
        val enabled = getService()?.isAnalyzerEnabled() ?: false
        invoke.resolve(JSObject().apply {
            put("value", enabled)
        })
    }

    @Command
    fun audioReinit(invoke: Invoke) {
        // Not needed on Android - ExoPlayer handles device changes automatically
        invoke.resolve(JSObject())
    }
    
    @Command
    fun audioCheckRecordPermission(invoke: Invoke) {
        try {
            val hasPermission = ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.RECORD_AUDIO
            ) == PackageManager.PERMISSION_GRANTED
            
            Log.d(TAG, "audioCheckRecordPermission: hasPermission=$hasPermission")
            
            val result = JSObject()
            result.put("granted", hasPermission)
            invoke.resolve(result)
        } catch (e: Exception) {
            Log.e(TAG, "audioCheckRecordPermission error", e)
            invoke.reject("Permission check failed: ${e.message}")
        }
    }
    
    @Command
    fun audioRequestRecordPermission(invoke: Invoke) {
        val hasPermission = ContextCompat.checkSelfPermission(
            activity,
            Manifest.permission.RECORD_AUDIO
        ) == PackageManager.PERMISSION_GRANTED
        
        if (hasPermission) {
            invoke.resolve(JSObject().apply {
                put("granted", true)
                put("status", "already_granted")
            })
            return
        }
        
        // Check if we should show rationale (i.e., user denied before but didn't check "Don't ask again")
        val shouldShowRationale = ActivityCompat.shouldShowRequestPermissionRationale(
            activity,
            Manifest.permission.RECORD_AUDIO
        )
        
        Log.d(TAG, "Requesting RECORD_AUDIO permission (shouldShowRationale: $shouldShowRationale)")
        
        // Store invoke to resolve after permission result
        pendingPermissionInvoke = invoke
        
        // Request permission using ActivityCompat
        ActivityCompat.requestPermissions(
            activity,
            arrayOf(Manifest.permission.RECORD_AUDIO),
            RECORD_AUDIO_REQUEST_CODE
        )
        
        // IMPORTANT: Do NOT resolve here!
        // The invoke will be resolved in onRequestPermissionsResult callback
        // Tauri mobile plugin system will wait for invoke.resolve() to be called
    }
    
    // Handle permission result callback
    fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        Log.d(TAG, "onRequestPermissionsResult called: requestCode=$requestCode, permissions=${permissions.contentToString()}, grantResults=${grantResults.contentToString()}")
        
        if (requestCode == RECORD_AUDIO_REQUEST_CODE) {
            val granted = grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
            Log.d(TAG, "RECORD_AUDIO permission result: granted=$granted, pendingInvoke=${pendingPermissionInvoke != null}")
            
            pendingPermissionInvoke?.resolve(JSObject().apply {
                put("granted", granted)
                put("status", if (granted) "granted" else "denied")
            })
            pendingPermissionInvoke = null
        }
    }
}
