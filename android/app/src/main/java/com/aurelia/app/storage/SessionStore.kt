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

  fun clear() {
    prefs.edit().clear().apply()
  }

  fun getServerUrl(): String? = prefs.getString("serverUrl", null)

  fun getUserId(): String? = prefs.getString("userId", null)

  fun getToken(): String? = prefs.getString("token", null)
}
