package com.aurelia.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.PlayerSnapshot
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class PlayerViewModel(private val playerController: PlayerController) : ViewModel() {
  private val mutableState = MutableStateFlow(PlayerState())
  val state: StateFlow<PlayerState> = mutableState

  init {
    playerController.observe { snapshot ->
      mutableState.update { it.fromSnapshot(snapshot) }
    }
    startProgressUpdates()
  }

  private fun startProgressUpdates() {
    viewModelScope.launch {
      while (isActive) {
        mutableState.update { current ->
          current.copy(positionMs = playerController.getCurrentState().positionMs)
        }
        delay(500L)
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

  private fun PlayerState.fromSnapshot(snapshot: PlayerSnapshot): PlayerState {
    return copy(
      title = snapshot.title,
      artist = snapshot.artist,
      albumArtUrl = snapshot.albumArtUrl,
      isPlaying = snapshot.isPlaying,
      positionMs = snapshot.positionMs,
      durationMs = snapshot.durationMs,
      queue = playerController.getQueue(),
      currentQueueIndex = playerController.getCurrentQueueIndex()
    )
  }

  fun playQueueItem(index: Int) {
    playerController.playQueueItem(index)
  }
}
