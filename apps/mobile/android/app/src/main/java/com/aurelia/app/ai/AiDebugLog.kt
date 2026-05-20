package com.aurelia.app.ai

import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

object AiDebugLog {
  private const val MAX_ENTRIES = 80
  private const val TAG = "OnDevicePlaylistGen"
  private val timeFormat = SimpleDateFormat("HH:mm:ss.SSS", Locale.US)
  private val mutableEntries = MutableStateFlow<List<String>>(emptyList())
  val entries: StateFlow<List<String>> = mutableEntries

  fun clear() {
    mutableEntries.value = emptyList()
  }

  fun info(message: String) {
    Log.i(TAG, message)
    append("I", message)
  }

  fun warn(
    message: String,
    error: Throwable? = null,
  ) {
    Log.w(TAG, message, error)
    append("W", listOfNotNull(message, error?.debugSummary()).joinToString(" | "))
  }

  fun text(): String = entries.value.joinToString("\n")

  private fun append(
    level: String,
    message: String,
  ) {
    val line = "${timeFormat.format(Date())} $level $message"
    mutableEntries.update { current -> (current + line).takeLast(MAX_ENTRIES) }
  }

  private fun Throwable.debugSummary(): String {
    val firstAppFrame =
      stackTrace.firstOrNull {
        it.className.startsWith("com.aurelia") ||
          it.className.startsWith("com.google.ai.edge.litertlm")
      }
    return listOfNotNull(
      javaClass.simpleName,
      message?.takeIf { it.isNotBlank() },
      firstAppFrame?.let { "${it.className}.${it.methodName}:${it.lineNumber}" },
    ).joinToString(" - ")
  }
}
