package com.aurelia.app.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Shuffle
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.components.PlaylistPickerDialog
import com.aurelia.app.ui.components.SongContextMenu
import com.aurelia.app.ui.components.rememberContextMenuState
import com.aurelia.app.ui.navigation.Screen
import com.aurelia.app.ui.theme.rememberGoogleSansFlexWideFont
import com.aurelia.app.utils.formatDuration
import uniffi.aurelia_core.Song

@Composable
fun ArtistDetailScreen(
  libraryViewModel: LibraryViewModel,
  artistId: String,
  artistName: String,
  sessionStore: SessionStore,
  playerController: PlayerController,
  playlistViewModel: PlaylistViewModel,
  onBack: () -> Unit,
  onOpenPlayer: () -> Unit,
  onNavigateToAlbum: ((Screen.AlbumDetail) -> Unit)? = null,
  hasPlayerBar: Boolean = false,
) {
  val state by libraryViewModel.state.collectAsStateWithLifecycle()
  val playlistState by playlistViewModel.state.collectAsStateWithLifecycle()
  val colors = MaterialTheme.colorScheme
  val wideFont = rememberGoogleSansFlexWideFont()

  val contextMenu = rememberContextMenuState()

  // Calculate bottom padding for miniplayer
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  // Filter songs for this artist
  val artistSongs = remember(state.songs, artistId) {
    state.songs.filter { song -> song.artistIds?.contains(artistId) == true }
      .sortedWith(compareBy({ it.album ?: "" }, { it.trackNumber ?: Int.MAX_VALUE }))
  }

  val songCount = artistSongs.size
  val albumCount = artistSongs.mapNotNull { it.albumId }.distinct().size

  val gradient = Brush.verticalGradient(
    colors = listOf(
      colors.primaryContainer,
      colors.background,
    ),
  )

  Column(
    modifier = Modifier
      .fillMaxSize()
      .background(gradient)
      .statusBarsPadding(),
  ) {
    // Header with back button
    Row(
      modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 8.dp, vertical = 8.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      IconButton(onClick = onBack) {
        Icon(
          imageVector = Icons.AutoMirrored.Filled.ArrowBack,
          contentDescription = "Back",
          tint = colors.onPrimaryContainer,
        )
      }
    }

    LazyColumn(
      modifier = Modifier.fillMaxSize(),
      contentPadding = PaddingValues(bottom = bottomPadding),
    ) {
      // Artist header
      item {
        Column(
          modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp),
          horizontalAlignment = Alignment.CenterHorizontally,
        ) {
          // Artist avatar
          Surface(
            modifier = Modifier
              .size(180.dp)
              .clip(CircleShape),
            shape = CircleShape,
            color = colors.surfaceVariant,
            tonalElevation = 8.dp,
            shadowElevation = 12.dp,
          ) {
            Box(
              modifier = Modifier.fillMaxSize(),
              contentAlignment = Alignment.Center,
            ) {
              Icon(
                imageVector = Icons.Filled.Person,
                contentDescription = null,
                modifier = Modifier.size(72.dp),
                tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
              )
            }
          }

          Spacer(modifier = Modifier.height(24.dp))

          // Artist name with display font
          Text(
            text = artistName,
            style = MaterialTheme.typography.headlineLarge.copy(
              fontFamily = wideFont,
              fontSize = 32.sp,
              lineHeight = 40.sp,
            ),
            fontWeight = FontWeight.Black,
            color = colors.onPrimaryContainer,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center,
          )

          Spacer(modifier = Modifier.height(8.dp))

          // Stats
          Text(
            text = "$albumCount albums - $songCount songs",
            style = MaterialTheme.typography.labelMedium,
            color = colors.onPrimaryContainer.copy(alpha = 0.6f),
          )

          Spacer(modifier = Modifier.height(24.dp))

          // Play all and Shuffle buttons (grouped)
          Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.Center,
          ) {
            Button(
              onClick = {
                if (artistSongs.isNotEmpty()) {
                  val serverUrl = sessionStore.getServerUrl() ?: return@Button
                  val token = sessionStore.getToken() ?: return@Button

                  playerController.setQueue(artistSongs, serverUrl, token, 0)
                  onOpenPlayer()
                }
              },
              modifier = Modifier
                .height(56.dp)
                .width(160.dp),
              shape = RoundedCornerShape(topStart = 28.dp, bottomStart = 28.dp, topEnd = 4.dp, bottomEnd = 4.dp),
              colors = ButtonDefaults.buttonColors(
                containerColor = colors.primary,
                contentColor = colors.onPrimary,
              ),
              contentPadding = PaddingValues(horizontal = 24.dp),
            ) {
              Icon(
                imageVector = Icons.Filled.PlayArrow,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
              )
              Spacer(modifier = Modifier.width(8.dp))
              Text(
                text = "Play",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
              )
            }

            Spacer(modifier = Modifier.width(6.dp))

            // Shuffle Button
            Button(
              onClick = {
                if (artistSongs.isNotEmpty()) {
                  val serverUrl = sessionStore.getServerUrl() ?: return@Button
                  val token = sessionStore.getToken() ?: return@Button

                  playerController.setQueue(artistSongs.shuffled(), serverUrl, token, 0)
                  onOpenPlayer()
                }
              },
              modifier = Modifier.height(56.dp),
              shape = RoundedCornerShape(topStart = 4.dp, bottomStart = 4.dp, topEnd = 28.dp, bottomEnd = 28.dp),
              colors = ButtonDefaults.buttonColors(
                containerColor = colors.secondaryContainer,
                contentColor = colors.onSecondaryContainer,
              ),
              contentPadding = PaddingValues(horizontal = 20.dp),
            ) {
              Icon(
                imageVector = Icons.Filled.Shuffle,
                contentDescription = "Shuffle",
                modifier = Modifier.size(24.dp),
              )
            }
          }

          Spacer(modifier = Modifier.height(32.dp))
        }
      }

      // Song list
      itemsIndexed(
        items = artistSongs,
        key = { _, song -> song.id },
      ) { index, song ->
        val isCurrentSong = song.id == state.currentSongId
        val isPlaying = state.nowPlaying?.isPlaying == true && isCurrentSong

        ArtistSongItem(
          song = song,
          albumName = song.album ?: "Unknown Album",
          albumArtUrl = song.albumArtUrl,
          duration = song.duration?.let { formatDuration((it * 1000).toLong()) },
          isPlaying = isPlaying,
          isCurrentSong = isCurrentSong,
          onClick = {
            val serverUrl = sessionStore.getServerUrl() ?: return@ArtistSongItem
            val token = sessionStore.getToken() ?: return@ArtistSongItem

            playerController.setQueue(artistSongs, serverUrl, token, index)
            onOpenPlayer()
          },
          onLongClick = { contextMenu.openContextMenu(song) },
          onMoreClick = { contextMenu.openContextMenu(song) },
          showContextMenu = contextMenu.showContextMenu && contextMenu.selectedSong?.id == song.id,
          onDismissMenu = { contextMenu.dismissContextMenu() },
          onAddToQueue = {
            val serverUrl = sessionStore.getServerUrl() ?: return@ArtistSongItem
            val token = sessionStore.getToken() ?: return@ArtistSongItem
            playerController.addToQueue(song, serverUrl, token)
          },
          onPlayNext = {
            val serverUrl = sessionStore.getServerUrl() ?: return@ArtistSongItem
            val token = sessionStore.getToken() ?: return@ArtistSongItem
            playerController.playNext(song, serverUrl, token)
          },
          onAddToPlaylist = { contextMenu.openPlaylistPicker(song) },
          onGoToAlbum = if (onNavigateToAlbum != null && song.albumId != null) {
            {
              onNavigateToAlbum(
                Screen.AlbumDetail(
                  albumId = song.albumId!!,
                  albumName = song.album ?: "Unknown Album",
                ),
              )
            }
          } else {
            null
          },
        )
      }
    }
  }

  // Playlist picker dialog
  if (contextMenu.showPlaylistPicker && contextMenu.selectedSong != null) {
    PlaylistPickerDialog(
      playlists = playlistState.playlists,
      isLoading = playlistState.isLoading,
      onDismiss = { contextMenu.dismissPlaylistPicker() },
      onSelectPlaylist = { playlist ->
        contextMenu.selectedSong?.let { song ->
          playlistViewModel.addSongsToPlaylist(playlist.id, listOf(song.id))
        }
        contextMenu.dismissPlaylistPicker()
      },
      onCreatePlaylist = { name ->
        contextMenu.selectedSong?.let { song ->
          playlistViewModel.createPlaylist(name, listOf(song.id))
        }
        contextMenu.dismissPlaylistPicker()
      },
    )
  }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun ArtistSongItem(
  song: Song,
  albumName: String,
  albumArtUrl: String?,
  duration: String?,
  isPlaying: Boolean,
  isCurrentSong: Boolean,
  onClick: () -> Unit,
  onLongClick: () -> Unit,
  onMoreClick: () -> Unit,
  showContextMenu: Boolean,
  onDismissMenu: () -> Unit,
  onAddToQueue: () -> Unit,
  onPlayNext: () -> Unit,
  onAddToPlaylist: () -> Unit,
  onGoToAlbum: (() -> Unit)?,
) {
  val colors = MaterialTheme.colorScheme
  val containerColor =
    if (isCurrentSong) colors.primaryContainer.copy(alpha = 0.5f) else colors.surface.copy(alpha = 0f)
  val shape = if (isCurrentSong) RoundedCornerShape(16.dp) else RoundedCornerShape(0.dp)

  Box(
    modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp)
  ) {
    Surface(
      modifier = Modifier
        .fillMaxWidth()
        .clip(shape)
        .combinedClickable(
          onClick = onClick,
          onLongClick = onLongClick,
        ),
      color = containerColor,
      shape = shape,
    ) {
      Row(
        modifier = Modifier
          .fillMaxWidth()
          .padding(start = 16.dp, end = 8.dp, top = 12.dp, bottom = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
      ) {
        // Album art
        AlbumArt(
          imageUrl = albumArtUrl,
          size = 48.dp,
          cornerRadius = 8.dp,
        )

        // Song info
        Column(modifier = Modifier.weight(1f)) {
          Text(
            text = song.name,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = if (isCurrentSong) FontWeight.SemiBold else FontWeight.Normal,
            color = if (isCurrentSong) colors.primary else colors.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
          Text(
            text = albumName,
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier =
              Modifier.clickable(
                enabled = onGoToAlbum != null,
                onClick = { onGoToAlbum?.invoke() },
              ),
          )
        }

        // Duration
        duration?.let {
          Text(
            text = it,
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
          )
        }

        // Playing indicator or more options button
        Box {
          if (isPlaying) {
            Icon(
              imageVector = Icons.Filled.PlayArrow,
              contentDescription = null,
              tint = colors.primary,
              modifier = Modifier
                .size(36.dp)
                .padding(8.dp),
            )
          } else {
            IconButton(onClick = onMoreClick) {
              Icon(
                imageVector = Icons.Filled.MoreVert,
                contentDescription = "More options",
                tint = colors.onSurfaceVariant,
              )
            }
          }

          SongContextMenu(
            song = song,
            expanded = showContextMenu,
            onDismiss = onDismissMenu,
            onAddToQueue = onAddToQueue,
            onPlayNext = onPlayNext,
            onAddToPlaylist = onAddToPlaylist,
            onGoToAlbum = onGoToAlbum,
            onGoToArtist = null, // Already on artist
            onToggleFavorite = null,
          )
        }
      }
    }
  }
}
