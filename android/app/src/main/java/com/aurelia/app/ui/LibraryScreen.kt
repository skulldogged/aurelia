package com.aurelia.app.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore

@Composable
fun LibraryScreen(sessionStore: SessionStore, playerController: PlayerController) {
  val viewModel: LibraryViewModel = viewModel(
    factory = LibraryViewModelFactory(sessionStore, playerController)
  )
  val state by viewModel.state.collectAsState()

  LaunchedEffect(Unit) {
    viewModel.loadLibrary()
  }

  Column(
    modifier = Modifier
      .fillMaxSize()
      .padding(PaddingValues(24.dp)),
    verticalArrangement = Arrangement.spacedBy(12.dp)
  ) {
    Text(text = "Library", style = MaterialTheme.typography.headlineMedium)

    when {
      state.isLoading -> CircularProgressIndicator()
      state.error != null -> Text(text = state.error ?: "", color = MaterialTheme.colorScheme.error)
      else -> {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
          items(state.songs) { song ->
            Column(
              modifier = Modifier
                .fillMaxWidth()
                .clickable { viewModel.play(song.id, song.container) }
                .padding(vertical = 8.dp)
            ) {
              Text(text = song.name, style = MaterialTheme.typography.bodyLarge)
              Text(
                text = song.artists?.joinToString(", ") ?: "Unknown Artist",
                style = MaterialTheme.typography.bodySmall
              )
            }
          }
        }
      }
    }
  }
}
