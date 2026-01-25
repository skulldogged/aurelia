package com.aurelia.app.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
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
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
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

sealed class SearchResult {
  data class SongResult(
    val song: Song,
  ) : SearchResult()

  data class Album(
    val id: String,
    val name: String,
    val artist: String,
    val albumArtUrl: String?,
  ) : SearchResult()

  data class Artist(
    val id: String?,
    val name: String,
    val songCount: Int,
  ) : SearchResult()
}

@Composable
fun SearchScreen(
  sessionStore: SessionStore,
  playerController: PlayerController,
  playlistViewModel: PlaylistViewModel,
  onOpenPlayer: () -> Unit,
  onNavigateToAlbum: ((Screen.AlbumDetail) -> Unit)? = null,
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
  val keyboardController = LocalSoftwareKeyboardController.current
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  var searchQuery by remember { mutableStateOf("") }

  // Context menu state
  var selectedSong by remember { mutableStateOf<Song?>(null) }
  var showContextMenu by remember { mutableStateOf(false) }
  var showPlaylistPicker by remember { mutableStateOf(false) }

  LaunchedEffect(Unit) {
    libraryViewModel.loadLibrary()
    playlistViewModel.loadPlaylists()
  }

  // Search results
  val results =
    remember(searchQuery, state.songs) {
      if (searchQuery.length < 2) {
        emptyList()
      } else {
        val query = searchQuery.lowercase()
        val songResults =
          state.songs
            .filter { song ->
              song.name.lowercase().contains(query) ||
                song.artists?.any { it.lowercase().contains(query) } == true ||
                song.album?.lowercase()?.contains(query) == true
            }.take(20)
            .map { SearchResult.SongResult(it) }

        // Get unique albums matching query
        val albumResults =
          state.songs
            .filter { it.album?.lowercase()?.contains(query) == true }
            .mapNotNull { song ->
              song.albumId?.let { id ->
                Triple(id, song.album ?: "", song.albumArtUrl)
              }
            }.distinctBy { it.first }
            .take(5)
            .map { (id, name, artUrl) -> SearchResult.Album(id, name, "", artUrl) }

        // Get unique artists matching query
        val artistResults =
          state.songs
            .filter { it.artists?.any { artist -> artist.lowercase().contains(query) } == true }
            .flatMap { song ->
              song.artists?.mapIndexedNotNull { index, artist ->
                if (artist.lowercase().contains(query)) {
                  val artistId = song.artistIds?.getOrNull(index)
                  Triple(artistId, artist, song)
                } else {
                  null
                }
              } ?: emptyList()
            }
            .groupBy { it.second }
            .map { (name, entries) ->
              SearchResult.Artist(
                id = entries.firstOrNull()?.first,
                name = name,
                songCount = entries.size,
              )
            }
            .take(5)

        artistResults + albumResults + songResults
      }
    }

  Column(
    modifier =
      Modifier
        .fillMaxSize()
        .statusBarsPadding(),
  ) {
    // Search header
    Column(
      modifier =
        Modifier
          .fillMaxWidth()
          .padding(horizontal = 16.dp, vertical = 16.dp),
      verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
      Text(
        text = "Search",
        style = MaterialTheme.typography.displayLarge,
        color = colors.onBackground,
      )

      OutlinedTextField(
        value = searchQuery,
        onValueChange = { searchQuery = it },
        modifier = Modifier.fillMaxWidth(),
        placeholder = { Text("Songs, albums, artists...") },
        leadingIcon = {
          Icon(
            imageVector = Icons.Filled.Search,
            contentDescription = null,
            tint = colors.onSurfaceVariant,
          )
        },
        trailingIcon = {
          if (searchQuery.isNotEmpty()) {
            IconButton(onClick = { searchQuery = "" }) {
              Icon(
                imageVector = Icons.Filled.Clear,
                contentDescription = "Clear",
                tint = colors.onSurfaceVariant,
              )
            }
          }
        },
        singleLine = true,
        shape = RoundedCornerShape(16.dp),
        colors =
          OutlinedTextFieldDefaults.colors(
            focusedBorderColor = colors.primary,
            unfocusedBorderColor = colors.outline.copy(alpha = 0.5f),
            focusedContainerColor = colors.surfaceContainerLow,
            unfocusedContainerColor = colors.surfaceContainerLow,
          ),
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
        keyboardActions = KeyboardActions(onSearch = { keyboardController?.hide() }),
      )
    }

    // Results
    when {
      searchQuery.isEmpty() -> {
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
                  imageVector = Icons.Filled.Search,
                  contentDescription = null,
                  tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
                  modifier = Modifier.size(40.dp),
                )
              }
            }
            Text(
              text = "Search your library",
              style = MaterialTheme.typography.titleMedium,
              fontWeight = FontWeight.SemiBold,
              color = colors.onSurface,
            )
            Text(
              text = "Find songs, albums, and artists",
              style = MaterialTheme.typography.bodyMedium,
              color = colors.onSurfaceVariant,
              textAlign = TextAlign.Center,
            )
          }
        }
      }

      results.isEmpty() && searchQuery.length >= 2 -> {
        Box(
          modifier =
            Modifier
              .fillMaxSize()
              .padding(32.dp),
          contentAlignment = Alignment.Center,
        ) {
          Text(
            text = "No results found for \"$searchQuery\"",
            style = MaterialTheme.typography.bodyLarge,
            color = colors.onSurfaceVariant,
            textAlign = TextAlign.Center,
          )
        }
      }

      else -> {
        LazyColumn(
          modifier = Modifier.fillMaxSize(),
          contentPadding =
            PaddingValues(
              start = 16.dp,
              end = 16.dp,
              top = 8.dp,
              bottom = bottomPadding,
            ),
          verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
          items(results, key = { result ->
            when (result) {
              is SearchResult.SongResult -> "song_${result.song.id}"
              is SearchResult.Album -> "album_${result.id}"
              is SearchResult.Artist -> "artist_${result.name}"
            }
          }) { result ->
            when (result) {
              is SearchResult.SongResult -> {
                SongSearchResult(
                  song = result.song,
                  onClick = {
                    libraryViewModel.play(
                      result.song.id,
                      result.song.container,
                      result.song.name,
                      result.song.artists?.joinToString(", "),
                      result.song.albumArtUrl,
                    )
                    onOpenPlayer()
                  },
                  onLongClick = {
                    selectedSong = result.song
                    showContextMenu = true
                  },
                  onMoreClick = {
                    selectedSong = result.song
                    showContextMenu = true
                  },
                  showContextMenu = showContextMenu && selectedSong?.id == result.song.id,
                  onDismissMenu = { showContextMenu = false },
                  onAddToQueue = {
                    val serverUrl = sessionStore.getServerUrl() ?: return@SongSearchResult
                    val token = sessionStore.getToken() ?: return@SongSearchResult
                    playerController.addToQueue(
                      QueueItem(
                        id = result.song.id,
                        uri = buildStreamUrl(serverUrl, token, result.song.id, result.song.container),
                        title = result.song.name,
                        artist = result.song.artists?.joinToString(", ") ?: "Unknown Artist",
                        albumArtUrl = result.song.albumArtUrl,
                        durationMs = (result.song.duration ?: 0.0).let { (it * 1000).toLong() },
                        isFavorite = result.song.isFavorite ?: false,
                      ),
                    )
                  },
                  onPlayNext = {
                    val serverUrl = sessionStore.getServerUrl() ?: return@SongSearchResult
                    val token = sessionStore.getToken() ?: return@SongSearchResult
                    playerController.playNext(
                      QueueItem(
                        id = result.song.id,
                        uri = buildStreamUrl(serverUrl, token, result.song.id, result.song.container),
                        title = result.song.name,
                        artist = result.song.artists?.joinToString(", ") ?: "Unknown Artist",
                        albumArtUrl = result.song.albumArtUrl,
                        durationMs = (result.song.duration ?: 0.0).let { (it * 1000).toLong() },
                        isFavorite = result.song.isFavorite ?: false,
                      ),
                    )
                  },
                  onAddToPlaylist = {
                    selectedSong = result.song
                    showPlaylistPicker = true
                  },
                  onGoToAlbum = if (onNavigateToAlbum != null && result.song.albumId != null) {
                    {
                      onNavigateToAlbum(
                        Screen.AlbumDetail(
                          albumId = result.song.albumId!!,
                          albumName = result.song.album ?: "Unknown Album",
                        ),
                      )
                    }
                  } else null,
                  onGoToArtist = if (onNavigateToArtist != null && result.song.artistIds?.isNotEmpty() == true) {
                    {
                      onNavigateToArtist(
                        Screen.ArtistDetail(
                          artistId = result.song.artistIds!!.first(),
                          artistName = result.song.artists?.firstOrNull() ?: "Unknown Artist",
                        ),
                      )
                    }
                  } else null,
                )
              }

              is SearchResult.Album -> {
                AlbumSearchResult(
                  name = result.name,
                  albumArtUrl = result.albumArtUrl,
                  onClick = {
                    onNavigateToAlbum?.invoke(
                      Screen.AlbumDetail(
                        albumId = result.id,
                        albumName = result.name,
                      ),
                    )
                  },
                )
              }

              is SearchResult.Artist -> {
                ArtistSearchResult(
                  name = result.name,
                  songCount = result.songCount,
                  onClick = {
                    result.id?.let { artistId ->
                      onNavigateToArtist?.invoke(
                        Screen.ArtistDetail(
                          artistId = artistId,
                          artistName = result.name,
                        ),
                      )
                    }
                  },
                )
              }
            }
          }
        }
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
private fun SongSearchResult(
  song: Song,
  onClick: () -> Unit,
  onLongClick: () -> Unit,
  onMoreClick: () -> Unit,
  showContextMenu: Boolean,
  onDismissMenu: () -> Unit,
  onAddToQueue: () -> Unit,
  onPlayNext: () -> Unit,
  onAddToPlaylist: () -> Unit,
  onGoToAlbum: (() -> Unit)?,
  onGoToArtist: (() -> Unit)?,
) {
  val colors = MaterialTheme.colorScheme

  Box {
    Surface(
      modifier =
        Modifier
          .fillMaxWidth()
          .clip(RoundedCornerShape(12.dp))
          .combinedClickable(
            onClick = onClick,
            onLongClick = onLongClick,
          ),
      shape = RoundedCornerShape(12.dp),
      color = colors.surfaceContainerLow,
    ) {
      Row(
        modifier = Modifier.padding(start = 12.dp, top = 8.dp, bottom = 8.dp, end = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        AlbumArt(
          imageUrl = song.albumArtUrl,
          size = 44.dp,
          cornerRadius = 8.dp,
          style = AlbumArtStyle.Song,
        )
        Column(
          modifier =
            Modifier
              .weight(1f)
              .padding(horizontal = 12.dp),
        ) {
          Text(
            text = song.name,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = FontWeight.Medium,
            color = colors.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
          Text(
            text = song.artists?.joinToString(", ") ?: "Unknown Artist",
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
        }
        Text(
          text = "Song",
          style = MaterialTheme.typography.labelSmall,
          color = colors.onSurfaceVariant.copy(alpha = 0.7f),
        )
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
            onGoToAlbum = onGoToAlbum,
            onGoToArtist = onGoToArtist,
            onToggleFavorite = null,
          )
        }
      }
    }
  }
}

@Composable
private fun AlbumSearchResult(
  name: String,
  albumArtUrl: String?,
  onClick: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme

  Surface(
    modifier =
      Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(12.dp))
        .clickable(onClick = onClick),
    shape = RoundedCornerShape(12.dp),
    color = colors.surfaceContainerLow,
  ) {
    Row(
      modifier = Modifier.padding(12.dp),
      verticalAlignment = Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      AlbumArt(
        imageUrl = albumArtUrl,
        size = 44.dp,
        cornerRadius = 8.dp,
        style = AlbumArtStyle.Album,
      )
      Text(
        text = name,
        style = MaterialTheme.typography.bodyLarge,
        fontWeight = FontWeight.Medium,
        color = colors.onSurface,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.weight(1f),
      )
      Text(
        text = "Album",
        style = MaterialTheme.typography.labelSmall,
        color = colors.onSurfaceVariant.copy(alpha = 0.7f),
      )
    }
  }
}

@Composable
private fun ArtistSearchResult(
  name: String,
  songCount: Int,
  onClick: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme

  Surface(
    modifier =
      Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(12.dp))
        .clickable(onClick = onClick),
    shape = RoundedCornerShape(12.dp),
    color = colors.surfaceContainerLow,
  ) {
    Row(
      modifier = Modifier.padding(12.dp),
      verticalAlignment = Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      Surface(
        modifier = Modifier.size(44.dp),
        shape = CircleShape,
        color = colors.surfaceVariant,
      ) {
        Box(contentAlignment = Alignment.Center) {
          Icon(
            imageVector = Icons.Filled.Person,
            contentDescription = null,
            tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
            modifier = Modifier.size(20.dp),
          )
        }
      }
      Column(modifier = Modifier.weight(1f)) {
        Text(
          text = name,
          style = MaterialTheme.typography.bodyLarge,
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
      Text(
        text = "Artist",
        style = MaterialTheme.typography.labelSmall,
        color = colors.onSurfaceVariant.copy(alpha = 0.7f),
      )
    }
  }
}
