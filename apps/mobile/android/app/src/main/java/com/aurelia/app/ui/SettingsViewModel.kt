package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.clearCache
import uniffi.aurelia_core.fetchSongs

data class SettingsState(
    val isSyncing: Boolean = false,
    val isClearing: Boolean = false,
    val syncSuccess: Boolean? = null,
    val clearSuccess: Boolean? = null,
    val error: String? = null,
)

class SettingsViewModel(
    private val sessionStore: SessionStore,
) : ViewModel() {
    private val mutableState = MutableStateFlow(SettingsState())
    val state: StateFlow<SettingsState> = mutableState

    fun syncLibrary() {
        val serverUrl = sessionStore.getServerUrl()
        val userId = sessionStore.getUserId()
        val token = sessionStore.getToken()
        val appDataDir = sessionStore.getAppDataDir()

        if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
            mutableState.update { it.copy(error = "Missing session data") }
            return
        }

        mutableState.update { it.copy(isSyncing = true, error = null, syncSuccess = null) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                fetchSongs(serverUrl, token, userId, appDataDir ?: "")
                mutableState.update { it.copy(isSyncing = false, syncSuccess = true) }
            } catch (e: Exception) {
                if (!AuthInterceptor.handlePotentialAuthError(e)) {
                    mutableState.update {
                        it.copy(isSyncing = false, syncSuccess = false, error = e.message ?: "Sync failed")
                    }
                }
            }
        }
    }

    fun clearLocalCache() {
        val appDataDir = sessionStore.getAppDataDir()

        if (appDataDir.isNullOrBlank()) {
            mutableState.update { it.copy(error = "No cache directory") }
            return
        }

        mutableState.update { it.copy(isClearing = true, error = null, clearSuccess = null) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                clearCache(appDataDir)
                mutableState.update { it.copy(isClearing = false, clearSuccess = true) }
            } catch (e: Exception) {
                mutableState.update {
                    it.copy(isClearing = false, clearSuccess = false, error = e.message ?: "Clear failed")
                }
            }
        }
    }

    fun clearMessages() {
        mutableState.update { it.copy(syncSuccess = null, clearSuccess = null, error = null) }
    }
}
