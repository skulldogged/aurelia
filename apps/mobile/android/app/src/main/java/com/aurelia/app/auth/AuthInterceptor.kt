package com.aurelia.app.auth

import android.util.Log

object AuthInterceptor {
    private const val TAG = "AuthInterceptor"
    private var logoutCallback: (() -> Unit)? = null

    fun setLogoutCallback(callback: () -> Unit) {
        logoutCallback = callback
    }

    fun clearLogoutCallback() {
        logoutCallback = null
    }

    fun isUnauthorizedError(error: Throwable): Boolean {
        val message = error.message?.lowercase() ?: return false
        return message.contains("unauthorized") ||
            message.contains("401") ||
            message.contains("authentication") ||
            message.contains("not authenticated")
    }

    fun isUnauthorizedError(errorMessage: String?): Boolean {
        val message = errorMessage?.lowercase() ?: return false
        return message.contains("unauthorized") ||
            message.contains("401") ||
            message.contains("authentication") ||
            message.contains("not authenticated")
    }

    fun handlePotentialAuthError(error: Throwable): Boolean {
        if (isUnauthorizedError(error)) {
            Log.w(TAG, "Unauthorized error detected, triggering logout")
            triggerLogout()
            return true
        }
        return false
    }

    fun handlePotentialAuthError(errorMessage: String?): Boolean {
        if (isUnauthorizedError(errorMessage)) {
            Log.w(TAG, "Unauthorized error detected, triggering logout")
            triggerLogout()
            return true
        }
        return false
    }

    private fun triggerLogout() {
        logoutCallback?.invoke() ?: Log.w(TAG, "Logout callback not set")
    }
}
