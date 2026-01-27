package com.aurelia.app.ui

import androidx.compose.foundation.ExperimentalFoundationApi
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.PlayArrow
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AlbumArtStyle
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.components.PlaylistPickerDialog
import com.aurelia.app.ui.components.SongContextMenu
import com.aurelia.app.ui.navigation.Screen
import uniffi.aurelia_core.Song
import uniffi.aurelia_core.buildStreamUrl

@Composable
fun LibraryScreen(
  sessionStore: SessionStore,
  playerController: PlayerController,
  playlistViewModel: PlaylistViewModel,
  onOpenPlayer: () -> Unit,
  onNavigateToAlbum: ((Screen.AlbumDetail) -> Unit)? = null,
  onNavigateToArtist: ((Screen.ArtistDetail) -> Unit)? = null,
  hasPlayerBar: Boolean = false,
) {
  val viewModel: LibraryViewModel =
    viewModel(
      factory = LibraryViewModelFactory(sessionStore, playerController),
    )
  val state by viewModel.state.collectAsState()
  val playlistState by playlistViewModel.state.collectAsState()
  val colors = MaterialTheme.colorScheme
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  // Context menu state
  var selectedSong by remember { mutableStateOf<Song?>(null) }
  var showContextMenu by remember { mutableStateOf(false) }
  var showPlaylistPicker by remember { mutableStateOf(false) }

  LaunchedEffect(Unit) {
    viewModel.loadLibrary()
    playlistViewModel.loadPlaylists()
  }

  Column(
    modifier =
      Modifier
        .fillMaxSize()
        .statusBarsPadding(),
  ) {
    // Header
    Column(
      modifier =
        Modifier
          .fillMaxWidth()
          .padding(horizontal = 24.dp, vertical = 16.dp),
    ) {
      Text(
        text = "Songs",
        style = MaterialTheme.typography.displayLarge,
        color = colors.onBackground,
      )
      Spacer(modifier = Modifier.height(4.dp))
      Text(
        text = "${state.songs.size} songs",
        style = MaterialTheme.typography.bodyMedium,
        color = colors.onSurfaceVariant,
      )
    }

    // Song list
    when {
      state.isLoading -> {
        Box(
          modifier =
            Modifier
              .fillMaxWidth()
              .weight(1f),
          contentAlignment = Alignment.Center,
        ) {
          CircularProgressIndicator(color = colors.primary)
        }
      }

      state.error != null -> {
        Box(
          modifier =
            Modifier
              .fillMaxWidth()
              .weight(1f)
              .padding(24.dp),
          contentAlignment = Alignment.Center,
        ) {
          Text(
            text = state.error ?: "",
            color = colors.error,
            style = MaterialTheme.typography.bodyLarge,
          )
        }
      }

      else -> {
        val currentSongId = state.currentSongId
        val isPlaying = state.nowPlaying?.isPlaying == true

        LazyColumn(
          modifier =
            Modifier
              .fillMaxWidth()
              .weight(1f),
          contentPadding =
            PaddingValues(
              start = 16.dp,
              end = 16.dp,
              top = 8.dp,
              bottom = bottomPadding,
            ),
          verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
          items(
            items = state.songs,
            key = { song -> song.id },
          ) { song ->
            val isCurrentSong = song.id == currentSongId

            EnhancedSongItem(
              song = song,
              isPlaying = isPlaying && isCurrentSong,
              isCurrentSong = isCurrentSong,
              onClick = {
                viewModel.playFromList(song.id)
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
              onAddToQueue = {
                val serverUrl = sessionStore.getServerUrl() ?: return@EnhancedSongItem
                val token = sessionStore.getToken() ?: return@EnhancedSongItem
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
                    codec = song.codec,
                    bitRate = song.bitRate,
                    sampleRate = song.sampleRate,
                  ),
                )
              },
              onPlayNext = {
                val serverUrl = sessionStore.getServerUrl() ?: return@EnhancedSongItem
                val token = sessionStore.getToken() ?: return@EnhancedSongItem
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
                    codec = song.codec,
                    bitRate = song.bitRate,
                    sampleRate = song.sampleRate,
                  ),
                )
              },
              onAddToPlaylist = {
                selectedSong = song
                showPlaylistPicker = true
              },
              onGoToAlbum =
                if (onNavigateToAlbum != null && song.albumId != null) {
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
    }
  }

  // Context menu
  if (showContextMenu && selectedSong != null) {
    SongContextMenu(
      song = selectedSong!!,
      expanded = true,
      onDismiss = { showContextMenu = false },
      onAddToQueue = {
        val serverUrl = sessionStore.getServerUrl()
        val token = sessionStore.getToken()
        if (serverUrl != null && token != null) {
          val song = selectedSong!!
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
              codec = song.codec,
              bitRate = song.bitRate,
              sampleRate = song.sampleRate,
            )
          )
        }
      },
      onPlayNext = {
        val serverUrl = sessionStore.getServerUrl()
        val token = sessionStore.getToken()
        if (serverUrl != null && token != null) {
          val song = selectedSong!!
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
              codec = song.codec,
              bitRate = song.bitRate,
              sampleRate = song.sampleRate,
            )
          )
        }
      },
      onAddToPlaylist = {
        showPlaylistPicker = true
      },
      onGoToAlbum = if (selectedSong?.albumId != null && onNavigateToAlbum != null) {
        {
          onNavigateToAlbum(
            Screen.AlbumDetail(
              albumId = selectedSong!!.albumId!!,
              albumName = selectedSong!!.album ?: "Unknown Album",
            )
          )
        }
      } else null,
      onGoToArtist = if (selectedSong?.artistIds?.isNotEmpty() == true && onNavigateToArtist != null) {
        {
          onNavigateToArtist(
            Screen.ArtistDetail(
              artistId = selectedSong!!.artistIds!!.first(),
              artistName = selectedSong!!.artists?.firstOrNull() ?: "Unknown Artist",
            )
          )
        }
      } else null,
    )
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
private fun EnhancedSongItem(
  song: Song,
  isPlaying: Boolean,
  isCurrentSong: Boolean,
  onClick: () -> Unit,
  onLongClick: () -> Unit,
  onMoreClick: () -> Unit,
  onAddToQueue: () -> Unit,
  onPlayNext: () -> Unit,
  onAddToPlaylist: () -> Unit,
  onGoToAlbum: (() -> Unit)?,
  onGoToArtist: (() -> Unit)?,
  modifier: Modifier = Modifier,
) {
  val colors = MaterialTheme.colorScheme

  val cornerRadius = if (isCurrentSong) 50.dp else 22.dp
  val albumCornerRadius = if (isCurrentSong) 50.dp else 12.dp

  val containerColor = if (isCurrentSong) colors.primaryContainer else colors.surfaceContainerLow
  val contentColor = if (isCurrentSong) colors.onPrimaryContainer else colors.onSurface
  val secondaryContentColor =
    if (isCurrentSong) colors.onPrimaryContainer.copy(alpha = 0.7f) else colors.onSurfaceVariant

  Box {
    Surface(
      modifier =
        modifier
          .fillMaxWidth()
          .clip(RoundedCornerShape(cornerRadius))
          .combinedClickable(
            onClick = onClick,
            onLongClick = onLongClick,
          ),
      shape = RoundedCornerShape(cornerRadius),
      color = containerColor,
    ) {
      Row(
        modifier =
          Modifier
            .fillMaxWidth()
            .padding(horizontal = 13.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        // Album art with play indicator overlay
        Box(modifier = Modifier.size(56.dp)) {
          AlbumArt(
            imageUrl = song.albumArtUrl,
            size = 56.dp,
            cornerRadius = albumCornerRadius,
            style = AlbumArtStyle.Song,
            containerColor = if (isCurrentSong) colors.primary.copy(alpha = 0.2f) else colors.surfaceVariant,
          )
          if (isPlaying) {
            Surface(
              modifier = Modifier.size(56.dp),
              shape = RoundedCornerShape(albumCornerRadius),
              color = colors.primary.copy(alpha = 0.7f),
            ) {
              Box(contentAlignment = Alignment.Center) {
                Icon(
                  imageVector = Icons.Filled.PlayArrow,
                  contentDescription = null,
                  tint = colors.onPrimary,
                  modifier = Modifier.size(24.dp),
                )
              }
            }
          }
        }

        Spacer(modifier = Modifier.width(12.dp))

        // Song info
        Column(modifier = Modifier.weight(1f)) {
          Text(
            text = song.name,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = if (isCurrentSong) FontWeight.Bold else FontWeight.Medium,
            color = contentColor,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
          Spacer(modifier = Modifier.height(2.dp))
          Text(
            text = song.artists?.joinToString(", ") ?: "Unknown Artist",
            style = MaterialTheme.typography.bodyMedium,
            color = secondaryContentColor,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier =
              Modifier.clickable(
                enabled = onGoToArtist != null,
                onClick = { onGoToArtist?.invoke() },
              ),
          )
        }

        // More options button
        Box {
          IconButton(onClick = onMoreClick) {
            Icon(
              imageVector = Icons.Filled.MoreVert,
              contentDescription = "More options",
              tint = secondaryContentColor,
            )
          }
        }
      }
    }
  }
}
