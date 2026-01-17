package com.aurelia.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AlbumArtStyle
import com.aurelia.app.ui.components.BottomBarDimensions


@Composable
fun LibraryScreen(
  sessionStore: SessionStore,
  playerController: PlayerController,
  onOpenPlayer: () -> Unit,
  hasPlayerBar: Boolean = false
) {
  val viewModel: LibraryViewModel = viewModel(
    factory = LibraryViewModelFactory(sessionStore, playerController)
  )
  val state by viewModel.state.collectAsState()
  val colors = MaterialTheme.colorScheme
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  LaunchedEffect(Unit) {
    viewModel.loadLibrary()
  }

  Column(
    modifier = Modifier
      .fillMaxSize()
      .statusBarsPadding()
  ) {
    // Header
    Column(
      modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 24.dp, vertical = 16.dp)
    ) {
      Text(
        text = "Songs",
        style = MaterialTheme.typography.displayLarge,
        fontWeight = FontWeight.Bold,
        color = colors.onBackground
      )
      Spacer(modifier = Modifier.height(4.dp))
      Text(
        text = "${state.songs.size} songs",
        style = MaterialTheme.typography.bodyMedium,
        color = colors.onSurfaceVariant
      )
    }

    // Song list
    when {
      state.isLoading -> {
        Box(
          modifier = Modifier
            .fillMaxWidth()
            .weight(1f),
          contentAlignment = Alignment.Center
        ) {
          CircularProgressIndicator(color = colors.primary)
        }
      }
      state.error != null -> {
        Box(
          modifier = Modifier
            .fillMaxWidth()
            .weight(1f)
            .padding(24.dp),
          contentAlignment = Alignment.Center
        ) {
          Text(
            text = state.error ?: "",
            color = colors.error,
            style = MaterialTheme.typography.bodyLarge
          )
        }
      }
      else -> {
        val currentSongId = state.currentSongId
        val isPlaying = state.nowPlaying?.isPlaying == true

        LazyColumn(
          modifier = Modifier
            .fillMaxWidth()
            .weight(1f),
          contentPadding = PaddingValues(
            start = 16.dp,
            end = 16.dp,
            top = 8.dp,
            bottom = bottomPadding
          ),
          verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
          items(
            items = state.songs,
            key = { song -> song.id }
          ) { song ->
            val isCurrentSong = song.id == currentSongId
            
            EnhancedSongItem(
              title = song.name,
              artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
              albumArtUrl = song.albumArtUrl,
              isPlaying = isPlaying && isCurrentSong,
              isCurrentSong = isCurrentSong,
              onClick = {
                viewModel.playFromList(song.id)
                onOpenPlayer()
              }
            )
          }
        }
      }
    }
  }
}

@Composable
private fun EnhancedSongItem(
  title: String,
  artist: String,
  albumArtUrl: String?,
  isPlaying: Boolean,
  isCurrentSong: Boolean,
  onClick: () -> Unit,
  modifier: Modifier = Modifier
) {
  val colors = MaterialTheme.colorScheme
  
  val cornerRadius = if (isCurrentSong) 50.dp else 22.dp
  val albumCornerRadius = if (isCurrentSong) 50.dp else 12.dp

  val containerColor = if (isCurrentSong) colors.primaryContainer else colors.surfaceContainerLow
  val contentColor = if (isCurrentSong) colors.onPrimaryContainer else colors.onSurface
  val secondaryContentColor = if (isCurrentSong) colors.onPrimaryContainer.copy(alpha = 0.7f) else colors.onSurfaceVariant

  Surface(
    modifier = modifier.fillMaxWidth(),
    shape = RoundedCornerShape(cornerRadius),
    color = containerColor,
    onClick = onClick
  ) {
    Row(
      modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 13.dp, vertical = 12.dp),
      verticalAlignment = Alignment.CenterVertically
    ) {
      // Album art with play indicator overlay
      Box(modifier = Modifier.size(56.dp)) {
        AlbumArt(
          imageUrl = albumArtUrl,
          size = 56.dp,
          cornerRadius = albumCornerRadius,
          style = AlbumArtStyle.Song,
          containerColor = if (isCurrentSong) colors.primary.copy(alpha = 0.2f) else colors.surfaceVariant
        )
        if (isPlaying) {
          Surface(
            modifier = Modifier.size(56.dp),
            shape = RoundedCornerShape(albumCornerRadius),
            color = colors.primary.copy(alpha = 0.7f)
          ) {
            Box(contentAlignment = Alignment.Center) {
              Icon(
                imageVector = Icons.Filled.PlayArrow,
                contentDescription = null,
                tint = colors.onPrimary,
                modifier = Modifier.size(24.dp)
              )
            }
          }
        }
      }

      Spacer(modifier = Modifier.width(12.dp))

      // Song info
      Column(modifier = Modifier.weight(1f)) {
        Text(
          text = title,
          style = MaterialTheme.typography.titleMedium,
          fontWeight = if (isCurrentSong) FontWeight.Bold else FontWeight.Medium,
          color = contentColor,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis
        )
        Spacer(modifier = Modifier.height(2.dp))
        Text(
          text = artist,
          style = MaterialTheme.typography.bodyMedium,
          color = secondaryContentColor,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis
        )
      }

      // More options button
      IconButton(onClick = { /* TODO: Show options */ }) {
        Icon(
          imageVector = Icons.Filled.MoreVert,
          contentDescription = "More options",
          tint = secondaryContentColor
        )
      }
    }
  }
}
