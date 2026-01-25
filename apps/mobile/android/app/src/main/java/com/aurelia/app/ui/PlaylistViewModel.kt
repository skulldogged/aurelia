package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.AppException
import uniffi.aurelia_core.Playlist
import uniffi.aurelia_core.PlaylistCreateData
import uniffi.aurelia_core.addPlaylistItems
import uniffi.aurelia_core.buildStreamUrl
import uniffi.aurelia_core.createPlaylist
import uniffi.aurelia_core.deletePlaylist
import uniffi.aurelia_core.getPlaylistItems
import uniffi.aurelia_core.getPlaylists

class PlaylistViewModel(
    private val sessionStore: SessionStore,
    private val playerController: PlayerController,
) : ViewModel() {
    private val mutableState = MutableStateFlow(PlaylistsState())
    val state: StateFlow<PlaylistsState> = mutableState

    private val mutableDetailState = MutableStateFlow(PlaylistDetailState())
    val detailState: StateFlow<PlaylistDetailState> = mutableDetailState

    fun loadPlaylists() {
        val serverUrl = sessionStore.getServerUrl()
        val userId = sessionStore.getUserId()
        val token = sessionStore.getToken()

        if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
            mutableState.update { it.copy(error = "Missing session data", isLoading = false) }
            return
        }

        mutableState.update { it.copy(isLoading = true, error = null) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                val playlists = getPlaylists(serverUrl, token, userId)
                mutableState.update { it.copy(isLoading = false, playlists = playlists) }
            } catch (error: AppException) {
                if (!AuthInterceptor.handlePotentialAuthError(error.message)) {
                    mutableState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load playlists") }
                }
            } catch (error: Exception) {
                if (!AuthInterceptor.handlePotentialAuthError(error)) {
                    mutableState.update { it.copy(isLoading = false, error = "Failed to load playlists") }
                }
            }
        }
    }

    fun createPlaylist(
        name: String,
        songIds: List<String>? = null,
    ) {
        val serverUrl = sessionStore.getServerUrl()
        val userId = sessionStore.getUserId()
        val token = sessionStore.getToken()

        if (serverUrl.isNullOrBlank() || userId.isNullOrBlank() || token.isNullOrBlank()) {
            return
        }

        mutableState.update { it.copy(isCreating = true) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                val data =
                    PlaylistCreateData(
                        name = name,
                        ids = songIds,
                        userId = userId,
                        isPublic = false,
                    )
                val newPlaylist = createPlaylist(serverUrl, token, data)
                mutableState.update { current ->
                    current.copy(
                        isCreating = false,
                        playlists = current.playlists + newPlaylist,
                    )
                }
            } catch (error: Exception) {
                mutableState.update { it.copy(isCreating = false, error = "Failed to create playlist") }
            }
        }
    }

    fun deletePlaylist(playlistId: String) {
        val serverUrl = sessionStore.getServerUrl()
        val token = sessionStore.getToken()

        if (serverUrl.isNullOrBlank() || token.isNullOrBlank()) {
            return
        }

        mutableState.update { it.copy(isDeleting = true) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                deletePlaylist(serverUrl, token, playlistId)
                mutableState.update { current ->
                    current.copy(
                        isDeleting = false,
                        playlists = current.playlists.filter { it.id != playlistId },
                    )
                }
            } catch (error: Exception) {
                mutableState.update { it.copy(isDeleting = false, error = "Failed to delete playlist") }
            }
        }
    }

    fun loadPlaylistDetail(
        playlistId: String,
        playlistName: String,
    ) {
        val serverUrl = sessionStore.getServerUrl()
        val token = sessionStore.getToken()

        if (serverUrl.isNullOrBlank() || token.isNullOrBlank()) {
            mutableDetailState.update { it.copy(error = "Missing session data", isLoading = false) }
            return
        }

        mutableDetailState.update { it.copy(isLoading = true, error = null) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                val songs = getPlaylistItems(serverUrl, token, playlistId)
                // Find the playlist from the main state if available
                val playlist = mutableState.value.playlists.find { it.id == playlistId }
                mutableDetailState.update {
                    it.copy(
                        isLoading = false,
                        playlist = playlist,
                        songs = songs,
                    )
                }
            } catch (error: AppException) {
                if (!AuthInterceptor.handlePotentialAuthError(error.message)) {
                    mutableDetailState.update { it.copy(isLoading = false, error = error.message ?: "Failed to load playlist") }
                }
            } catch (error: Exception) {
                if (!AuthInterceptor.handlePotentialAuthError(error)) {
                    mutableDetailState.update { it.copy(isLoading = false, error = "Failed to load playlist") }
                }
            }
        }
    }

    fun addSongsToPlaylist(
        playlistId: String,
        songIds: List<String>,
    ) {
        val serverUrl = sessionStore.getServerUrl()
        val token = sessionStore.getToken()

        if (serverUrl.isNullOrBlank() || token.isNullOrBlank()) {
            return
        }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                addPlaylistItems(serverUrl, token, playlistId, songIds)
                // Reload playlist detail to reflect changes
                loadPlaylistDetail(playlistId, "")
                // Also reload playlists to update child count
                loadPlaylists()
            } catch (_: Exception) {
                // Silent fail for now
            }
        }
    }

    fun playPlaylist(startIndex: Int = 0) {
        val serverUrl = sessionStore.getServerUrl() ?: return
        val token = sessionStore.getToken() ?: return
        val songs = mutableDetailState.value.songs

        if (songs.isEmpty()) return

        val queueItems =
            songs.map { song ->
                QueueItem(
                    id = song.id,
                    uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                    title = song.name,
                    artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                    albumArtUrl = song.albumArtUrl,
                    durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                    isFavorite = song.isFavorite ?: false,
                )
            }

        playerController.setQueue(queueItems, startIndex)
    }

    fun shufflePlaylist() {
        val serverUrl = sessionStore.getServerUrl() ?: return
        val token = sessionStore.getToken() ?: return
        val songs = mutableDetailState.value.songs.shuffled()

        if (songs.isEmpty()) return

        val queueItems =
            songs.map { song ->
                QueueItem(
                    id = song.id,
                    uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                    title = song.name,
                    artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                    albumArtUrl = song.albumArtUrl,
                    durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                    isFavorite = song.isFavorite ?: false,
                )
            }

        playerController.setQueue(queueItems, 0)
    }

    fun clearError() {
        mutableState.update { it.copy(error = null) }
        mutableDetailState.update { it.copy(error = null) }
    }
}
