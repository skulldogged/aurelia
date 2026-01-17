package com.aurelia.app.storage

import android.content.Context

class SessionStore(context: Context) {
  private val prefs = context.getSharedPreferences("aurelia_session", Context.MODE_PRIVATE)

  fun save(serverUrl: String, userId: String, token: String) {
    prefs.edit()
      .putString("serverUrl", serverUrl)
      .putString("userId", userId)
      .putString("token", token)
      .apply()
  }

  fun setAppDataDir(path: String) {
    prefs.edit().putString("appDataDir", path).apply()
  }

  fun getAppDataDir(): String? = prefs.getString("appDataDir", null)

  fun setUseDynamicColor(enabled: Boolean) {
    prefs.edit().putBoolean("useDynamicColor", enabled).apply()
  }

  fun getUseDynamicColor(): Boolean = prefs.getBoolean("useDynamicColor", true)

  fun clear() {
    prefs.edit().clear().apply()
  }

  fun getServerUrl(): String? = prefs.getString("serverUrl", null)

  fun getUserId(): String? = prefs.getString("userId", null)

  fun getToken(): String? = prefs.getString("token", null)
}
