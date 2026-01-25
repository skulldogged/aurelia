package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.auth.AuthInterceptor
import com.aurelia.app.data.model.Lyrics
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.PlayerSnapshot
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.utils.LyricsUtils
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.aurelia_core.getLyrics as uniffiGetLyrics
import uniffi.aurelia_core.toggleFavorite as uniffiToggleFavorite

class PlayerViewModel(
    private val playerController: PlayerController,
    private val sessionStore: SessionStore,
) : ViewModel() {
    private val mutableState = MutableStateFlow(PlayerState())
    val state: StateFlow<PlayerState> = mutableState

    private var lastFetchedSongId: String? = null
    private var favoriteCache: MutableMap<String, Boolean> = mutableMapOf()

    init {
        playerController.observe { snapshot ->
            val previousSongId = mutableState.value.currentSongId
            val newSongId = snapshot.currentSongId

            // Update favorite cache from queue items
            val queue = playerController.getQueue()
            queue.forEach { item ->
                if (!favoriteCache.containsKey(item.id)) {
                    favoriteCache[item.id] = item.isFavorite
                }
            }

            mutableState.update { current ->
                val isFavorite = newSongId?.let { favoriteCache[it] } ?: false
                current.fromSnapshot(snapshot, isFavorite)
            }

            if (newSongId != null && newSongId.isNotBlank() && newSongId != lastFetchedSongId) {
                lastFetchedSongId = newSongId
                fetchLyrics(newSongId, snapshot.artist, snapshot.title)
            }
        }
    }

    private fun fetchLyrics(
        songId: String?,
        artist: String,
        title: String,
    ) {
        viewModelScope.launch(Dispatchers.IO) {
            lastFetchedSongId = songId
            mutableState.update { it.copy(lyrics = null) }
            try {
                val serverUrl = sessionStore.getServerUrl() ?: ""
                val token = sessionStore.getToken() ?: ""
                val itemId = songId ?: ""

                val lrcContent = uniffiGetLyrics(serverUrl, token, itemId, artist, title)
                val lyrics = LyricsUtils.parseLyrics(lrcContent)

                if (lyrics.isValid()) {
                    mutableState.update { it.copy(lyrics = lyrics) }
                } else {
                    mutableState.update { it.copy(showLyrics = false) }
                }
            } catch (e: Exception) {
                mutableState.update { it.copy(showLyrics = false) }
                // Only handle auth errors, ignore others (e.g., lyrics not found)
                AuthInterceptor.handlePotentialAuthError(e)
            }
        }
    }

    fun togglePlayPause() {
        val currentState = mutableState.value
        if (currentState.isPlaying) {
            playerController.pause()
        } else {
            playerController.resume()
        }
    }

    fun seekTo(positionMs: Long) {
        playerController.seekTo(positionMs)
    }

    fun skipNext() {
        playerController.skipNext()
    }

    fun skipPrevious() {
        playerController.skipPrevious()
    }

    private fun PlayerState.fromSnapshot(
        snapshot: PlayerSnapshot,
        isFavorite: Boolean,
    ): PlayerState =
        copy(
            title = snapshot.title,
            artist = snapshot.artist,
            albumArtUrl = snapshot.albumArtUrl,
            isPlaying = snapshot.isPlaying,
            isBuffering = snapshot.isBuffering,
            positionMs = snapshot.positionMs,
            durationMs = snapshot.durationMs,
            queue = playerController.getQueue(),
            currentQueueIndex = playerController.getCurrentQueueIndex(),
            isShuffled = snapshot.isShuffled,
            repeatMode = snapshot.repeatMode,
            currentSongId = snapshot.currentSongId,
            isFavorite = isFavorite,
            playbackSpeed = snapshot.playbackSpeed,
            updateTimeMs = snapshot.updateTimeMs,
        )

    fun playQueueItem(index: Int) {
        playerController.playQueueItem(index)
    }

    fun toggleLyrics() {
        val currentState = mutableState.value
        if (currentState.lyrics == null && currentState.title.isNotBlank()) {
            fetchLyrics(currentState.currentSongId, currentState.artist, currentState.title)
        }
        mutableState.update { it.copy(showLyrics = !it.showLyrics) }
    }

    fun setLyrics(lyrics: Lyrics?) {
        mutableState.update { it.copy(lyrics = lyrics) }
    }

    fun clearLyrics() {
        mutableState.update { it.copy(lyrics = null, showLyrics = false) }
    }

    fun toggleShuffle() {
        playerController.toggleShuffle()
    }

    fun cycleRepeatMode() {
        playerController.cycleRepeatMode()
    }

    fun toggleFavorite() {
        val currentState = mutableState.value
        val songId = currentState.currentSongId ?: return

        val serverUrl = sessionStore.getServerUrl() ?: return
        val token = sessionStore.getToken() ?: return
        val userId = sessionStore.getUserId() ?: return

        mutableState.update { it.copy(isFavoriteLoading = true) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                val newFavoriteState =
                    uniffiToggleFavorite(
                        serverUrl = serverUrl,
                        token = token,
                        userId = userId,
                        itemId = songId,
                        isFavorite = currentState.isFavorite,
                    )
                favoriteCache[songId] = newFavoriteState
                mutableState.update {
                    it.copy(isFavorite = newFavoriteState, isFavoriteLoading = false)
                }
            } catch (e: Exception) {
                if (!AuthInterceptor.handlePotentialAuthError(e)) {
                    e.printStackTrace()
                    mutableState.update { it.copy(isFavoriteLoading = false) }
                }
            }
        }
    }

    fun setFavoriteFromSong(
        songId: String,
        isFavorite: Boolean,
    ) {
        favoriteCache[songId] = isFavorite
        if (mutableState.value.currentSongId == songId) {
            mutableState.update { it.copy(isFavorite = isFavorite) }
        }
    }
}
