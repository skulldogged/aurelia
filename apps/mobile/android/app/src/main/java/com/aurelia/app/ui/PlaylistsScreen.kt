package com.aurelia.app.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.PlaylistPlay
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AlbumArtStyle
import com.aurelia.app.ui.components.LibraryMessageState
import com.aurelia.app.ui.components.LibraryScreenHeader
import com.aurelia.app.ui.components.MediaListItem
import com.aurelia.app.ui.navigation.Screen
import uniffi.aurelia_core.Playlist

private val MiniPlayerHeight = 64.dp
private val NavBarContentHeight = 90.dp

@Composable
fun PlaylistsScreen(
  viewModel: PlaylistViewModel,
  onOpenPlayer: () -> Unit,
  onNavigateToPlaylist: (Screen.PlaylistDetail) -> Unit,
  hasPlayerBar: Boolean = false,
) {
  val state by viewModel.state.collectAsStateWithLifecycle()
  val colors = MaterialTheme.colorScheme

  var showCreateDialog by remember { mutableStateOf(false) }
  var playlistToDelete by remember { mutableStateOf<Playlist?>(null) }

  LaunchedEffect(Unit) {
    viewModel.ensureLoaded()
  }

  val systemNavBarInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
  val fabBottomPadding =
    NavBarContentHeight + systemNavBarInset + 24.dp +
      (if (hasPlayerBar) MiniPlayerHeight + 12.dp else 0.dp)

  Box(
    modifier =
      Modifier
        .fillMaxSize()
        .statusBarsPadding(),
  ) {
    Column(modifier = Modifier.fillMaxSize()) {
      when {
        state.isLoading -> {
          Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
          ) {
            CircularProgressIndicator()
          }
        }

        state.error != null -> {
          Box(
            modifier =
              Modifier
                .fillMaxSize()
                .padding(32.dp),
            contentAlignment = Alignment.Center,
          ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
              Text(
                text = state.error ?: "An error occurred",
                style = MaterialTheme.typography.bodyLarge,
                color = colors.error,
                textAlign = TextAlign.Center,
              )
              Spacer(modifier = Modifier.height(16.dp))
              TextButton(onClick = { viewModel.ensureLoaded(force = true) }) {
                Text("Retry")
              }
            }
          }
        }

        state.playlists.isEmpty() -> {
          LibraryMessageState(
            icon = Icons.AutoMirrored.Filled.PlaylistPlay,
            title = "No playlists yet",
            subtitle = "Create a playlist to organize your favorite music.",
            modifier = Modifier.fillMaxSize(),
          )
        }

        else -> {
          val bottomPadding =
            NavBarContentHeight + systemNavBarInset +
              (if (hasPlayerBar) MiniPlayerHeight + 12.dp else 0.dp)

          LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding =
              PaddingValues(
                start = 16.dp,
                end = 16.dp,
                bottom = bottomPadding + 80.dp,
              ),
            verticalArrangement = Arrangement.spacedBy(8.dp),
          ) {
            item(key = "header") {
              LibraryScreenHeader(
                title = "Playlists",
                subtitle = "Your music collections",
                modifier = Modifier.padding(horizontal = 8.dp),
              )
            }

            items(state.playlists, key = { it.id }) { playlist ->
              MediaListItem(
                title = playlist.name,
                subtitle = "${playlist.childCount ?: 0} songs",
                imageUrl = null,
                artworkStyle = AlbumArtStyle.Playlist,
                showMore = playlist.canDelete == true,
                onClick = {
                  onNavigateToPlaylist(
                    Screen.PlaylistDetail(
                      playlistId = playlist.id,
                      playlistName = playlist.name,
                    ),
                  )
                },
                onMoreClick = { playlistToDelete = playlist },
              )
            }
          }
        }
      }
    }

    // FAB
    FloatingActionButton(
      onClick = { showCreateDialog = true },
      modifier =
        Modifier
          .align(Alignment.BottomEnd)
          .padding(end = 16.dp, bottom = fabBottomPadding),
      containerColor = colors.primary,
      contentColor = colors.onPrimary,
    ) {
      Icon(
        imageVector = Icons.Filled.Add,
        contentDescription = "Create playlist",
      )
    }
  }

  // Create playlist dialog
  if (showCreateDialog) {
    CreatePlaylistDialog(
      isCreating = state.isCreating,
      onDismiss = { showCreateDialog = false },
      onCreate = { name ->
        viewModel.createPlaylist(name)
        showCreateDialog = false
      },
    )
  }

  // Delete confirmation dialog
  playlistToDelete?.let { playlist ->
    AlertDialog(
      onDismissRequest = { playlistToDelete = null },
      title = { Text("Delete Playlist") },
      text = { Text("Are you sure you want to delete \"${playlist.name}\"?") },
      confirmButton = {
        TextButton(
          onClick = {
            viewModel.deletePlaylist(playlist.id)
            playlistToDelete = null
          },
        ) {
          Text("Delete", color = MaterialTheme.colorScheme.error)
        }
      },
      dismissButton = {
        TextButton(onClick = { playlistToDelete = null }) {
          Text("Cancel")
        }
      },
    )
  }
}

@Composable
private fun EmptyPlaylistsView() {
  val colors = MaterialTheme.colorScheme

  Box(
    modifier =
      Modifier
        .fillMaxSize()
        .padding(32.dp),
    contentAlignment = Alignment.Center,
  ) {
    Column(
      horizontalAlignment = Alignment.CenterHorizontally,
      verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
      Surface(
        modifier = Modifier.size(80.dp),
        shape = RoundedCornerShape(24.dp),
        color = colors.surfaceVariant,
      ) {
        Box(contentAlignment = Alignment.Center) {
          Icon(
            imageVector = Icons.AutoMirrored.Filled.PlaylistPlay,
            contentDescription = null,
            tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
            modifier = Modifier.size(40.dp),
          )
        }
      }
      Text(
        text = "No playlists yet",
        style = MaterialTheme.typography.titleMedium,
        fontWeight = FontWeight.SemiBold,
        color = colors.onSurface,
      )
      Text(
        text = "Create a playlist to organize your favorite music",
        style = MaterialTheme.typography.bodyMedium,
        color = colors.onSurfaceVariant,
        textAlign = TextAlign.Center,
      )
    }
  }
}

@Composable
private fun PlaylistItem(
  playlist: Playlist,
  onClick: () -> Unit,
  onDelete: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme
  val songCount = playlist.childCount ?: 0

  Surface(
    modifier =
      Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(12.dp))
        .clickable(onClick = onClick),
    color = colors.surface,
    tonalElevation = 1.dp,
  ) {
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .padding(12.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      // Playlist artwork placeholder
      AlbumArt(
        imageUrl = null, // Playlists don't have images by default
        size = 56.dp,
        cornerRadius = 8.dp,
        style = AlbumArtStyle.Playlist,
        containerColor = colors.primaryContainer,
        contentColor = colors.onPrimaryContainer,
      )

      Spacer(modifier = Modifier.width(16.dp))

      Column(modifier = Modifier.weight(1f)) {
        Text(
          text = playlist.name,
          style = MaterialTheme.typography.titleMedium,
          fontWeight = FontWeight.Medium,
          color = colors.onSurface,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
        Text(
          text = "$songCount songs",
          style = MaterialTheme.typography.bodySmall,
          color = colors.onSurfaceVariant,
        )
      }

      if (playlist.canDelete == true) {
        IconButton(onClick = onDelete) {
          Icon(
            imageVector = Icons.Default.Delete,
            contentDescription = "Delete playlist",
            tint = colors.onSurfaceVariant,
          )
        }
      }
    }
  }
}

@Composable
private fun CreatePlaylistDialog(
  isCreating: Boolean,
  onDismiss: () -> Unit,
  onCreate: (String) -> Unit,
) {
  var name by remember { mutableStateOf("") }

  AlertDialog(
    onDismissRequest = onDismiss,
    title = { Text("Create Playlist") },
    text = {
      OutlinedTextField(
        value = name,
        onValueChange = { name = it },
        label = { Text("Playlist name") },
        singleLine = true,
        enabled = !isCreating,
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
        keyboardActions =
          KeyboardActions(
            onDone = {
              if (name.isNotBlank()) {
                onCreate(name.trim())
              }
            },
          ),
        modifier = Modifier.fillMaxWidth(),
      )
    },
    confirmButton = {
      TextButton(
        onClick = { onCreate(name.trim()) },
        enabled = name.isNotBlank() && !isCreating,
      ) {
        if (isCreating) {
          CircularProgressIndicator(
            modifier = Modifier.size(16.dp),
            strokeWidth = 2.dp,
          )
        } else {
          Text("Create")
        }
      }
    },
    dismissButton = {
      TextButton(onClick = onDismiss, enabled = !isCreating) {
        Text("Cancel")
      }
    },
  )
}
