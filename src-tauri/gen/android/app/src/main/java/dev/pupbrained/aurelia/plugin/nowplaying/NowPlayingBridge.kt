package dev.pupbrained.aurelia.plugin.nowplaying

import android.util.Log

object NowPlayingBridge {
    @Volatile
    private var listener: ((String) -> Unit)? = null

    fun setListener(newListener: ((String) -> Unit)?) {
        listener = newListener
    }

    fun emit(action: String) {
        val target = listener
        if (target == null) {
            Log.d(TAG, "No listener registered for now playing control action: $action")
        } else {
            target.invoke(action)
        }
    }

    private const val TAG = "NowPlayingBridge"
}
