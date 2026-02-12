package com.aurelia.app.storage

import android.content.Context
import android.util.Log
import androidx.core.content.edit
import uniffi.aurelia_core.Credentials
import uniffi.aurelia_core.clearCredentials
import uniffi.aurelia_core.loadCredentials
import uniffi.aurelia_core.saveCredentials

class SessionStore(
  context: Context,
) {
  private val prefs = context.getSharedPreferences("aurelia_session", Context.MODE_PRIVATE)
  private var migrationAttempted = false

  fun save(
    serverUrl: String,
    userId: String,
    token: String,
    username: String = "",
  ) {
    val appDataDir = getAppDataDir()
    if (appDataDir.isNullOrEmpty()) {
      Log.w(TAG, "Cannot save credentials: appDataDir not set")
      return
    }
    try {
      val credentials = Credentials(
        serverUrl = serverUrl,
        username = username,
        token = token,
        userId = userId,
      )
      saveCredentials(appDataDir, credentials)
    } catch (e: Exception) {
      Log.e(TAG, "Failed to save credentials to redb", e)
    }
  }

  fun setAppDataDir(path: String) {
    prefs.edit { putString("appDataDir", path) }
  }

  fun getAppDataDir(): String? = prefs.getString("appDataDir", null)

  fun setUseDynamicColor(enabled: Boolean) {
    prefs.edit { putBoolean("useDynamicColor", enabled) }
  }

  fun getUseDynamicColor(): Boolean = prefs.getBoolean("useDynamicColor", true)

  fun clear() {
    val appDataDir = getAppDataDir()
    if (!appDataDir.isNullOrEmpty()) {
      try {
        clearCredentials(appDataDir)
      } catch (e: Exception) {
        Log.e(TAG, "Failed to clear credentials from redb", e)
      }
    }
    // Also clear legacy SharedPreferences credentials (but keep settings)
    prefs.edit {
      remove("serverUrl")
        .remove("userId")
        .remove("token")
    }
  }

  fun getCredentials(): Credentials? {
    val appDataDir = getAppDataDir() ?: return null

    // Attempt migration from SharedPreferences on first access
    if (!migrationAttempted) {
      migrationAttempted = true
      migrateFromSharedPreferences(appDataDir)
    }

    return try {
      loadCredentials(appDataDir)
    } catch (e: Exception) {
      Log.e(TAG, "Failed to load credentials from redb", e)
      null
    }
  }

  fun getServerUrl(): String? = getCredentials()?.serverUrl

  fun getUserId(): String? = getCredentials()?.userId

  fun getToken(): String? = getCredentials()?.token

  fun setLyricsServerUrl(url: String?) {
    prefs.edit {
      if (url.isNullOrBlank()) {
        remove("lyricsServerUrl")
      } else {
        putString("lyricsServerUrl", url)
      }
    }
  }

  fun getLyricsServerUrl(): String? = prefs.getString("lyricsServerUrl", null)

  // EQ Settings
  fun setEQEnabled(enabled: Boolean) {
    prefs.edit { putBoolean("eq_enabled", enabled) }
  }

  fun getEQEnabled(): Boolean = prefs.getBoolean("eq_enabled", false)

  fun setEQBands(bands: List<Float>) {
    prefs.edit { putString("eq_bands", bands.joinToString(",")) }
  }

  fun getEQBands(): List<Float> {
    val stored = prefs.getString("eq_bands", null) ?: return listOf(0f, 0f, 0f, 0f, 0f)
    return stored.split(",").mapNotNull { it.toFloatOrNull() }.takeIf { it.size == 5 }
      ?: listOf(0f, 0f, 0f, 0f, 0f)
  }

  fun setEQPreset(presetName: String?) {
    prefs.edit {
      if (presetName.isNullOrBlank()) remove("eq_preset") else putString("eq_preset", presetName)
    }
  }

  fun getEQPreset(): String? = prefs.getString("eq_preset", null)

  // Visualizer Settings
  fun setVisualizerEnabled(enabled: Boolean) {
    prefs.edit { putBoolean("visualizer_enabled", enabled) }
  }

  fun getVisualizerEnabled(): Boolean = prefs.getBoolean("visualizer_enabled", false)

  fun setVisualizerStyle(style: String) {
    prefs.edit { putString("visualizer_style", style) }
  }

  fun getVisualizerStyle(): String = prefs.getString("visualizer_style", "BARS") ?: "BARS"

  // Debug performance toggles (debug builds only UI)
  fun setDebugDisablePlayerBackdropBlur(disabled: Boolean) {
    prefs.edit { putBoolean("debug_disable_player_backdrop_blur", disabled) }
  }

  fun getDebugDisablePlayerBackdropBlur(): Boolean =
    prefs.getBoolean("debug_disable_player_backdrop_blur", false)

  fun setDebugDisablePlayerBackdropImageLayer(disabled: Boolean) {
    prefs.edit { putBoolean("debug_disable_player_backdrop_image_layer", disabled) }
  }

  fun getDebugDisablePlayerBackdropImageLayer(): Boolean =
    prefs.getBoolean("debug_disable_player_backdrop_image_layer", false)

  fun setDebugDisablePlayerTransitions(disabled: Boolean) {
    prefs.edit { putBoolean("debug_disable_player_transitions", disabled) }
  }

  fun getDebugDisablePlayerTransitions(): Boolean =
    prefs.getBoolean("debug_disable_player_transitions", false)

  fun getDeviceId(): String {
    val savedId = prefs.getString("device_id", null)
    if (savedId != null) return savedId

    val newId = java.util.UUID.randomUUID().toString()
    prefs.edit { putString("device_id", newId) }
    return newId
  }

  private fun migrateFromSharedPreferences(appDataDir: String) {
    val oldServerUrl = prefs.getString("serverUrl", null)
    val oldUserId = prefs.getString("userId", null)
    val oldToken = prefs.getString("token", null)

    // Only migrate if old credentials exist
    if (oldServerUrl != null && oldUserId != null && oldToken != null) {
      // Check if redb already has credentials
      try {
        val existingCreds = loadCredentials(appDataDir)
        if (existingCreds != null) {
          // Already have credentials in redb, just clear old ones
          clearOldSharedPreferencesCredentials()
          Log.d(TAG, "Credentials already in redb, cleared legacy SharedPreferences")
          return
        }
      } catch (e: Exception) {
        Log.w(TAG, "Could not check existing redb credentials", e)
      }

      // Migrate to redb
      try {
        val credentials = Credentials(
          serverUrl = oldServerUrl,
          username = "",
          token = oldToken,
          userId = oldUserId,
        )
        saveCredentials(appDataDir, credentials)
        clearOldSharedPreferencesCredentials()
        Log.i(TAG, "Successfully migrated credentials from SharedPreferences to redb")
      } catch (e: Exception) {
        Log.e(TAG, "Failed to migrate credentials to redb", e)
      }
    }
  }

  private fun clearOldSharedPreferencesCredentials() {
    prefs.edit {
      remove("serverUrl")
        .remove("userId")
        .remove("token")
    }
  }

  companion object {
    private const val TAG = "SessionStore"
  }
}
