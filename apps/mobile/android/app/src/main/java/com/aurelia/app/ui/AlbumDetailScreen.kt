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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Album
import androidx.compose.material.icons.filled.MoreVert
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import coil.compose.SubcomposeAsyncImage
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.components.PlaylistPickerDialog
import com.aurelia.app.ui.components.SongContextMenu
import com.aurelia.app.ui.navigation.Screen
import com.aurelia.app.ui.theme.SquircleShape
import com.aurelia.app.ui.theme.rememberGoogleSansFlexWideFont
import uniffi.aurelia_core.Song
import uniffi.aurelia_core.buildStreamUrl

@Composable
fun AlbumDetailScreen(
  albumId: String,
  albumName: String,
  sessionStore: SessionStore,
  playerController: PlayerController,
  playlistViewModel: PlaylistViewModel,
  onBack: () -> Unit,
  onOpenPlayer: () -> Unit,
  onNavigateToArtist: ((Screen.ArtistDetail) -> Unit)? = null,
  hasPlayerBar: Boolean = false,
) {
  val libraryViewModel: LibraryViewModel =
    viewModel(
      factory = LibraryViewModelFactory(sessionStore, playerController),
    )
  val state by libraryViewModel.state.collectAsState()
  val playlistState by playlistViewModel.state.collectAsState()
  val colors = MaterialTheme.colorScheme
  val wideFont = rememberGoogleSansFlexWideFont()

  // Context menu state
  var selectedSong by remember { mutableStateOf<Song?>(null) }
  var showContextMenu by remember { mutableStateOf(false) }
  var showPlaylistPicker by remember { mutableStateOf(false) }

  // Calculate bottom padding for miniplayer
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  LaunchedEffect(Unit) {
    libraryViewModel.loadLibrary()
    playlistViewModel.loadPlaylists()
  }

  // Filter songs for this album
  val albumSongs =
    remember(state.songs, albumId) {
      state.songs
        .filter { it.albumId == albumId }
        .sortedBy { it.trackNumber ?: Int.MAX_VALUE }
    }

  val albumArtUrl = albumSongs.firstOrNull()?.albumArtUrl
  val artistName = albumSongs.firstOrNull()?.artists?.joinToString(", ") ?: "Unknown Artist"
  val artistId = albumSongs.firstOrNull()?.artistIds?.firstOrNull()

  val gradient =
    Brush.verticalGradient(
      colors =
        listOf(
          colors.primaryContainer,
          colors.background,
        ),
    )

  Column(
    modifier =
      Modifier
        .fillMaxSize()
        .background(gradient)
        .statusBarsPadding(),
  ) {
    // Header with back button
    Row(
      modifier =
        Modifier
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
      // Album header with art
      item {
        Column(
          modifier =
            Modifier
              .fillMaxWidth()
              .padding(horizontal = 24.dp),
          horizontalAlignment = Alignment.CenterHorizontally,
        ) {
          // Album art
          Surface(
            modifier =
              Modifier
                .size(240.dp)
                .clip(SquircleShape),
            shape = SquircleShape,
            color = colors.surfaceVariant,
            tonalElevation = 8.dp,
            shadowElevation = 12.dp,
          ) {
            if (albumArtUrl.isNullOrBlank()) {
              Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
              ) {
                Icon(
                  imageVector = Icons.Filled.Album,
                  contentDescription = null,
                  modifier = Modifier.size(80.dp),
                  tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
                )
              }
            } else {
              SubcomposeAsyncImage(
                model = albumArtUrl,
                contentDescription = albumName,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
              )
            }
          }

          Spacer(modifier = Modifier.height(24.dp))

          // Album name with display font for impact
          Text(
            text = albumName,
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

          // Artist name
          Text(
            text = artistName,
            style = MaterialTheme.typography.titleMedium,
            color = colors.onPrimaryContainer.copy(alpha = 0.8f),
            textAlign = TextAlign.Center,
            modifier =
              Modifier.clickable(
                enabled = artistId != null && onNavigateToArtist != null,
                onClick = {
                  artistId?.let { id ->
                    onNavigateToArtist?.invoke(Screen.ArtistDetail(id, artistName))
                  }
                },
              ),
          )

          // Song count
          Text(
            text = "${albumSongs.size} songs • ${calculateTotalDuration(albumSongs)}",
            style = MaterialTheme.typography.labelMedium,
            color = colors.onPrimaryContainer.copy(alpha = 0.6f),
            modifier = Modifier.padding(top = 4.dp),
          )

          Spacer(modifier = Modifier.height(24.dp))

          // Play all and Shuffle buttons (grouped)
          Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.Center,
          ) {
            Button(
              onClick = {
                if (albumSongs.isNotEmpty()) {
                  val serverUrl = sessionStore.getServerUrl() ?: return@Button
                  val token = sessionStore.getToken() ?: return@Button

                  val queueItems =
                    albumSongs.map { song ->
                      QueueItem(
                        id = song.id,
                        uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                        title = song.name,
                        artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                        albumArtUrl = song.albumArtUrl,
                        durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                        isFavorite = song.isFavorite ?: false,
                        albumId = song.albumId,
                        artistId = song.artistIds?.firstOrNull(),
                        albumName = song.album,
                      )
                    }
                  playerController.setQueue(queueItems, 0)
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

            // Shuffle Button (Secondary Action)
            Button(
              onClick = {
                if (albumSongs.isNotEmpty()) {
                  val serverUrl = sessionStore.getServerUrl() ?: return@Button
                  val token = sessionStore.getToken() ?: return@Button

                  val queueItems =
                    albumSongs.map { song ->
                      QueueItem(
                        id = song.id,
                        uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                        title = song.name,
                        artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                        albumArtUrl = song.albumArtUrl,
                        durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                        isFavorite = song.isFavorite ?: false,
                        albumId = song.albumId,
                        artistId = song.artistIds?.firstOrNull(),
                        albumName = song.album,
                      )
                    }.shuffled()
                  playerController.setQueue(queueItems, 0)
                  onOpenPlayer()
                }
              },
              modifier = Modifier
                .height(56.dp),
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
        items = albumSongs,
        key = { _, song -> song.id },
      ) { index, song ->
        val isCurrentSong = song.id == state.currentSongId
        val isPlaying = state.nowPlaying?.isPlaying == true && isCurrentSong

        AlbumSongItem(
          song = song,
          trackNumber = song.trackNumber ?: (index + 1),
          duration = song.duration?.let { formatDuration((it * 1000).toLong()) },
          isPlaying = isPlaying,
          isCurrentSong = isCurrentSong,
          onClick = {
            val serverUrl = sessionStore.getServerUrl() ?: return@AlbumSongItem
            val token = sessionStore.getToken() ?: return@AlbumSongItem

            val queueItems =
              albumSongs.map { s ->
                QueueItem(
                  id = s.id,
                  uri = buildStreamUrl(serverUrl, token, s.id, s.container),
                  title = s.name,
                  artist = s.artists?.joinToString(", ") ?: "Unknown Artist",
                  albumArtUrl = s.albumArtUrl,
                  durationMs = (s.duration ?: 0.0).let { (it * 1000).toLong() },
                  isFavorite = s.isFavorite ?: false,
                  albumId = s.albumId,
                  artistId = s.artistIds?.firstOrNull(),
                  albumName = s.album,
                )
              }
            playerController.setQueue(queueItems, index)
            onOpenPlayer()
          },
          onLongClick = {
            selectedSong = song
            showContextMenu = true
          },
          onMoreClick = {
            selectedSong = song
            showContextMenu = true
          },
          showContextMenu = showContextMenu && selectedSong?.id == song.id,
          onDismissMenu = { showContextMenu = false },
          onAddToQueue = {
            val serverUrl = sessionStore.getServerUrl() ?: return@AlbumSongItem
            val token = sessionStore.getToken() ?: return@AlbumSongItem
            playerController.addToQueue(
              QueueItem(
                id = song.id,
                uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                title = song.name,
                artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                albumArtUrl = song.albumArtUrl,
                durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                isFavorite = song.isFavorite ?: false,
                albumId = song.albumId,
                artistId = song.artistIds?.firstOrNull(),
                albumName = song.album,
              ),
            )
          },
          onPlayNext = {
            val serverUrl = sessionStore.getServerUrl() ?: return@AlbumSongItem
            val token = sessionStore.getToken() ?: return@AlbumSongItem
            playerController.playNext(
              QueueItem(
                id = song.id,
                uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                title = song.name,
                artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                albumArtUrl = song.albumArtUrl,
                durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                isFavorite = song.isFavorite ?: false,
                albumId = song.albumId,
                artistId = song.artistIds?.firstOrNull(),
                albumName = song.album,
              ),
            )
          },
          onAddToPlaylist = {
            selectedSong = song
            showPlaylistPicker = true
          },
          onGoToArtist =
            if (onNavigateToArtist != null && song.artistIds?.isNotEmpty() == true) {
              {
                onNavigateToArtist(
                  Screen.ArtistDetail(
                    artistId = song.artistIds!!.first(),
                    artistName = song.artists?.firstOrNull() ?: "Unknown Artist",
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
  if (showPlaylistPicker && selectedSong != null) {
    PlaylistPickerDialog(
      playlists = playlistState.playlists,
      isLoading = playlistState.isLoading,
      onDismiss = {
        showPlaylistPicker = false
        selectedSong = null
      },
      onSelectPlaylist = { playlist ->
        selectedSong?.let { song ->
          playlistViewModel.addSongsToPlaylist(playlist.id, listOf(song.id))
        }
        showPlaylistPicker = false
        selectedSong = null
      },
      onCreatePlaylist = { name ->
        selectedSong?.let { song ->
          playlistViewModel.createPlaylist(name, listOf(song.id))
        }
        showPlaylistPicker = false
        selectedSong = null
      },
    )
  }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun AlbumSongItem(
  song: Song,
  trackNumber: Int,
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
  onGoToArtist: (() -> Unit)?,
) {
  val colors = MaterialTheme.colorScheme
  val containerColor =
    if (isCurrentSong) colors.primaryContainer.copy(alpha = 0.5f) else colors.surface.copy(alpha = 0f)
  val shape = if (isCurrentSong) RoundedCornerShape(16.dp) else RoundedCornerShape(0.dp)

  Box(
    modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp)
  ) {
    Surface(
      modifier =
        Modifier
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
        modifier =
          Modifier
            .fillMaxWidth()
            .padding(start = 16.dp, end = 8.dp, top = 12.dp, bottom = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        // Track number or playing indicator
        Box(
          modifier = Modifier.width(24.dp),
          contentAlignment = Alignment.Center,
        ) {
          if (isPlaying) {
            Icon(
              imageVector = Icons.Filled.PlayArrow,
              contentDescription = null,
              tint = colors.primary,
              modifier = Modifier.size(20.dp),
            )
          } else {
            Text(
              text = trackNumber.toString(),
              style = MaterialTheme.typography.bodyMedium,
              color = if (isCurrentSong) colors.primary else colors.onSurfaceVariant,
            )
          }
        }

        Spacer(modifier = Modifier.width(16.dp))

        // Song title
        Text(
          text = song.name,
          style = MaterialTheme.typography.bodyLarge,
          fontWeight = if (isCurrentSong) FontWeight.SemiBold else FontWeight.Normal,
          color = if (isCurrentSong) colors.primary else colors.onSurface,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
          modifier = Modifier.weight(1f),
        )

        // Duration
        duration?.let {
          Text(
            text = it,
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
          )
        }

        // More options button
        Box {
          IconButton(onClick = onMoreClick) {
            Icon(
              imageVector = Icons.Filled.MoreVert,
              contentDescription = "More options",
              tint = colors.onSurfaceVariant,
            )
          }

          SongContextMenu(
            song = song,
            expanded = showContextMenu,
            onDismiss = onDismissMenu,
            onAddToQueue = onAddToQueue,
            onPlayNext = onPlayNext,
            onAddToPlaylist = onAddToPlaylist,
            onGoToAlbum = null, // Already on album
            onGoToArtist = onGoToArtist,
            onToggleFavorite = null,
          )
        }
      }
    }
  }
}

private fun formatDuration(durationMs: Long): String {
  val totalSeconds = durationMs / 1000
  val minutes = totalSeconds / 60
  val seconds = totalSeconds % 60
  return "%d:%02d".format(minutes, seconds)
}

private fun calculateTotalDuration(songs: List<Song>): String {
  val totalSeconds = songs.sumOf { (it.duration ?: 0.0) }.toLong()
  val hours = totalSeconds / 3600
  val minutes = (totalSeconds % 3600) / 60

  return if (hours > 0) {
    "${hours}h ${minutes}m"
  } else {
    "${minutes} min"
  }
}
