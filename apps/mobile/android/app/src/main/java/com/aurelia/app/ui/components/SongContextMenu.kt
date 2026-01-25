package com.aurelia.app.ui.components

import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.PlaylistAdd
import androidx.compose.material.icons.automirrored.filled.PlaylistPlay
import androidx.compose.material.icons.automirrored.filled.QueueMusic
import androidx.compose.material.icons.filled.Album
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.aurelia_core.Song

@Composable
fun SongContextMenu(
  song: Song,
  expanded: Boolean,
  onDismiss: () -> Unit,
  onAddToQueue: () -> Unit,
  onPlayNext: () -> Unit,
  onAddToPlaylist: () -> Unit,
  onGoToAlbum: (() -> Unit)? = null,
  onGoToArtist: (() -> Unit)? = null,
  onToggleFavorite: (() -> Unit)? = null,
) {
  val isFavorite = song.isFavorite ?: false

  DropdownMenu(
    expanded = expanded,
    onDismissRequest = onDismiss,
    modifier = Modifier.width(220.dp),
  ) {
    DropdownMenuItem(
      text = { Text("Play Next") },
      onClick = {
        onPlayNext()
        onDismiss()
      },
      leadingIcon = {
        Icon(
          imageVector = Icons.AutoMirrored.Filled.PlaylistPlay,
          contentDescription = null,
          tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
      },
    )

    DropdownMenuItem(
      text = { Text("Add to Queue") },
      onClick = {
        onAddToQueue()
        onDismiss()
      },
      leadingIcon = {
        Icon(
          imageVector = Icons.AutoMirrored.Filled.QueueMusic,
          contentDescription = null,
          tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
      },
    )

    DropdownMenuItem(
      text = { Text("Add to Playlist") },
      onClick = {
        onAddToPlaylist()
        onDismiss()
      },
      leadingIcon = {
        Icon(
          imageVector = Icons.AutoMirrored.Filled.PlaylistAdd,
          contentDescription = null,
          tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
      },
    )

    HorizontalDivider()

    if (onGoToAlbum != null && song.albumId != null) {
      DropdownMenuItem(
        text = { Text("Go to Album") },
        onClick = {
          onGoToAlbum()
          onDismiss()
        },
        leadingIcon = {
          Icon(
            imageVector = Icons.Default.Album,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
          )
        },
      )
    }

    if (onGoToArtist != null && song.artistIds?.isNotEmpty() == true) {
      DropdownMenuItem(
        text = { Text("Go to Artist") },
        onClick = {
          onGoToArtist()
          onDismiss()
        },
        leadingIcon = {
          Icon(
            imageVector = Icons.Default.Person,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
          )
        },
      )
    }

    if (onToggleFavorite != null) {
      DropdownMenuItem(
        text = { Text(if (isFavorite) "Remove from Favorites" else "Add to Favorites") },
        onClick = {
          onToggleFavorite()
          onDismiss()
        },
        leadingIcon = {
          Icon(
            imageVector = if (isFavorite) Icons.Default.Favorite else Icons.Default.FavoriteBorder,
            contentDescription = null,
            tint =
              if (isFavorite) {
                MaterialTheme.colorScheme.primary
              } else {
                MaterialTheme.colorScheme.onSurfaceVariant
              },
          )
        },
      )
    }
  }
}
