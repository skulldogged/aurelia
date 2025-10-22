package dev.pupbrained.aurelia.plugin.nowplaying

import android.app.Activity
import android.content.Intent
import android.util.Log
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class NowPlayingPayload {
    var id: String? = null
    var title: String = ""
    var artists: MutableList<String> = mutableListOf()
    var album: String? = null
    var durationSeconds: Double? = null
    var positionSeconds: Double? = null
    var isPlaying: Boolean = false
    var hasNext: Boolean = false
    var hasPrevious: Boolean = false
    var isShuffled: Boolean = false
    var repeatMode: String? = null
    var artworkUrl: String? = null
    var artworkPath: String? = null
    var artworkData: String? = null
}

@TauriPlugin
class NowPlayingPlugin(private val activity: Activity) : Plugin(activity) {

    @Command
    fun updateNowPlaying(invoke: Invoke) {
        val args = invoke.parseArgs(NowPlayingPayload::class.java)
        if (args.title.isBlank()) {
            invoke.resolve(errorResponse("Missing title"))
            return
        }

        val context = activity.applicationContext
        val intent = Intent(context, NowPlayingService::class.java).apply {
            action = NowPlayingService.ACTION_UPDATE
            putExtra(NowPlayingService.EXTRA_MEDIA_ID, args.id)
            putExtra(NowPlayingService.EXTRA_TITLE, args.title)
            putStringArrayListExtra(
                NowPlayingService.EXTRA_ARTISTS,
                ArrayList(args.artists)
            )
            putExtra(NowPlayingService.EXTRA_ALBUM, args.album)
            args.durationSeconds?.let { putExtra(NowPlayingService.EXTRA_DURATION_SECONDS, it) }
            args.positionSeconds?.let { putExtra(NowPlayingService.EXTRA_POSITION_SECONDS, it) }
            putExtra(NowPlayingService.EXTRA_IS_PLAYING, args.isPlaying)
            putExtra(NowPlayingService.EXTRA_HAS_NEXT, args.hasNext)
            putExtra(NowPlayingService.EXTRA_HAS_PREVIOUS, args.hasPrevious)
            putExtra(NowPlayingService.EXTRA_IS_SHUFFLED, args.isShuffled)
            putExtra(NowPlayingService.EXTRA_REPEAT_MODE, args.repeatMode)
            putExtra(NowPlayingService.EXTRA_ARTWORK_URL, args.artworkUrl)
            putExtra(NowPlayingService.EXTRA_ARTWORK_PATH, args.artworkPath)
            putExtra(NowPlayingService.EXTRA_ARTWORK_DATA, args.artworkData)
        }

        try {
            ContextCompat.startForegroundService(context, intent)
            invoke.resolve(successResponse())
        } catch (exception: Exception) {
            Log.e(TAG, "Failed to update now playing service", exception)
            invoke.resolve(errorResponse(exception.message ?: "Unknown error"))
        }
    }

    @Command
    fun clearNowPlaying(invoke: Invoke) {
        val context = activity.applicationContext
        val stopped = try {
            context.stopService(Intent(context, NowPlayingService::class.java))
        } catch (exception: Exception) {
            Log.w(TAG, "Failed to stop now playing service directly", exception)
            false
        }

        if (!stopped) {
            Log.d(TAG, "Now playing service was not running when attempting to clear")
        }

        invoke.resolve(successResponse())
    }

    private fun successResponse(): JSObject = JSObject().apply {
        put("success", true)
    }

    private fun errorResponse(message: String): JSObject = JSObject().apply {
        put("success", false)
        put("message", message)
    }

    companion object {
        private const val TAG = "NowPlayingPlugin"
    }
}
