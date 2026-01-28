package com.aurelia.app.utils

import com.aurelia.app.storage.SessionStore

data class SessionData(
    val serverUrl: String,
    val userId: String,
    val token: String,
    val appDataDir: String?,
)

/**
 * Validates that required session credentials are present.
 * Returns null if any required field is missing.
 */
fun validateSession(
    sessionStore: SessionStore,
    requireAppDataDir: Boolean = false,
): SessionData? {
    val serverUrl = sessionStore.getServerUrl()
    val userId = sessionStore.getUserId()
    val token = sessionStore.getToken()
    val appDataDir = sessionStore.getAppDataDir()

    if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
        return null
    }
    if (requireAppDataDir && appDataDir.isNullOrBlank()) {
        return null
    }

    return SessionData(serverUrl, userId, token, appDataDir)
}
