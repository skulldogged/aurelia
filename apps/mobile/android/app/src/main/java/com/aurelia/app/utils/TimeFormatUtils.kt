package com.aurelia.app.utils

/**
 * Formats a duration in milliseconds to "m:ss" format.
 * Used by album, artist, and playlist detail screens.
 */
fun formatDuration(durationMs: Long): String {
    val totalSeconds = durationMs / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "%d:%02d".format(minutes, seconds)
}

/**
 * Formats a time in milliseconds to "m:ss" format, clamping negative values to 0.
 * Used by the player screen for current position / duration display.
 */
fun formatTime(timeMs: Long): String {
    val totalSeconds = timeMs.coerceAtLeast(0L) / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "%d:%02d".format(minutes, seconds)
}
