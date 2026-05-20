package com.aurelia.app.ui

import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Album
import androidx.compose.material.icons.filled.MusicNote
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
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
import androidx.compose.ui.draw.scale
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import coil.request.ImageRequest
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.components.LibraryScreenHeader
import com.aurelia.app.ui.components.LibrarySectionHeader
import com.aurelia.app.ui.components.PlaylistPickerDialog
import com.aurelia.app.ui.components.SongContextMenu
import com.aurelia.app.ui.components.rememberContextMenuState
import com.aurelia.app.ui.navigation.Screen
import com.aurelia.app.ui.theme.AsymmetricSoftShape
import com.aurelia.app.ui.theme.PuffyShape
import com.aurelia.app.ui.theme.SquircleShape
import com.aurelia.app.ui.theme.rememberContextualStyle
import com.aurelia.app.ui.theme.rememberGoogleSansFlexWideFont
import com.aurelia.app.ui.theme.rememberPressScale
import com.aurelia.app.utils.optimizedArtworkUrl
import uniffi.aurelia_core.Song
@Composable
fun HomeScreen(
  viewModel: HomeViewModel,
  sessionStore: SessionStore,
  playerController: PlayerController,
  playlistViewModel: PlaylistViewModel,
  onOpenPlayer: () -> Unit,
  onNavigateToAlbum: (Screen.AlbumDetail) -> Unit = {},
  onNavigateToArtist: (Screen.ArtistDetail) -> Unit = {},
  hasPlayerBar: Boolean = false,
) {
  val state by viewModel.state.collectAsStateWithLifecycle()
  val playlistState by playlistViewModel.state.collectAsStateWithLifecycle()
  val colors = MaterialTheme.colorScheme
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  val contextMenu = rememberContextMenuState()

  when {
    state.isLoading && state.recentlyPlayed.isEmpty() && state.mostPlayed.isEmpty() -> {
      Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
      ) {
        CircularProgressIndicator(color = colors.primary)
      }
    }

    state.error != null && state.recentlyPlayed.isEmpty() -> {
      Box(
        modifier =
          Modifier
            .fillMaxSize()
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
      // Calculate quickPicks outside the grid (composable context)
      val quickPicks =
        remember(state.mostPlayed, state.recentlyPlayed) {
          (state.mostPlayed + state.recentlyPlayed)
            .distinctBy { it.id }
            .take(UiConstants.QUICK_PICKS_LIMIT)
        }

      // Get the most recently played song for the hero section
      val continueListeningSong = remember(state.recentlyPlayed) {
        state.recentlyPlayed.firstOrNull()
      }

      Column(
        modifier =
          Modifier
            .fillMaxSize()
            .statusBarsPadding(),
      ) {
        LazyVerticalGrid(
          columns = GridCells.Fixed(2),
          modifier = Modifier.fillMaxSize(),
          contentPadding =
            PaddingValues(
              start = 20.dp,
              end = 20.dp,
              top = 8.dp,
              bottom = bottomPadding,
            ),
          horizontalArrangement = Arrangement.spacedBy(12.dp),
          verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
          item(span = { GridItemSpan(2) }) {
            LibraryScreenHeader(
              title = "Home",
              subtitle = "Continue listening and rediscover your library",
              modifier = Modifier.padding(horizontal = 4.dp),
            )
          }

          // Continue Listening Hero Section
          if (continueListeningSong != null) {
            item(span = { GridItemSpan(2) }) {
              ContinueListeningHero(
                song = continueListeningSong,
                isCurrentSong = continueListeningSong.id == state.currentSongId,
                isPlaying = state.nowPlaying?.isPlaying == true && continueListeningSong.id == state.currentSongId,
                onClick = {
                  viewModel.playSongFromList(continueListeningSong.id, state.recentlyPlayed)
                  onOpenPlayer()
                },
                onGoToArtist =
                  continueListeningSong.safePrimaryArtistId()?.let { artistId ->
                    {
                      onNavigateToArtist(
                        Screen.ArtistDetail(
                          artistId = artistId,
                          artistName = continueListeningSong.artists?.firstOrNull() ?: "Unknown",
                        ),
                      )
                    }
                  },
              )
            }
          }

          // Quick Picks section header - spans full width with scanning context
          if (state.mostPlayed.isNotEmpty() || state.recentlyPlayed.isNotEmpty()) {
            item(span = { GridItemSpan(2) }) {
              LibrarySectionHeader(
                title = "Quick Picks",
                subtitle = "Fast starts from recent and frequent plays",
                modifier = Modifier.padding(top = 16.dp),
              )
            }
          }

          // Quick picks - compact song cards in 2-column grid with squircle shape
          items(
            items = quickPicks,
            key = { "quick_${it.id}" },
          ) { song ->
            QuickPickCard(
              song = song,
              isCurrentSong = song.id == state.currentSongId,
              isPlaying = state.nowPlaying?.isPlaying == true && song.id == state.currentSongId,
              onClick = {
                viewModel.playSongFromList(song.id, quickPicks)
                onOpenPlayer()
              },
              onLongClick = { contextMenu.openContextMenu(song) },
              showContextMenu = contextMenu.showContextMenu && contextMenu.selectedSong?.id == song.id,
              onDismissMenu = { contextMenu.dismissContextMenu() },
              onAddToQueue = {
                val serverUrl = sessionStore.getServerUrl() ?: return@QuickPickCard
                val token = sessionStore.getToken() ?: return@QuickPickCard
                playerController.addToQueue(song, serverUrl, token)
              },
              onPlayNext = {
                val serverUrl = sessionStore.getServerUrl() ?: return@QuickPickCard
                val token = sessionStore.getToken() ?: return@QuickPickCard
                playerController.playNext(song, serverUrl, token)
              },
              onAddToPlaylist = { contextMenu.openPlaylistPicker(song) },
              onGoToAlbum =
                song.safeAlbumId()?.let { albumId ->
                  {
                    onNavigateToAlbum(
                      Screen.AlbumDetail(
                        albumId = albumId,
                        albumName = song.album ?: "Unknown Album",
                      ),
                    )
                  }
                },
              onGoToArtist =
                song.safePrimaryArtistId()?.let { artistId ->
                  {
                    onNavigateToArtist(
                      Screen.ArtistDetail(
                        artistId = artistId,
                        artistName = song.artists?.firstOrNull() ?: "Unknown Artist",
                      ),
                    )
                  }
                },
            )
          }

          // Recently Added Albums section header - spans full width
          if (state.recentlyAddedAlbums.isNotEmpty()) {
            item(span = { GridItemSpan(2) }) {
              LibrarySectionHeader(
                title = "Recently Added",
                modifier = Modifier.padding(top = 20.dp),
              )
            }

            // Album cards in horizontal scroll (inside grid as full-width item)
            item(span = { GridItemSpan(2) }) {
              LazyRow(
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                contentPadding = PaddingValues(vertical = 4.dp),
              ) {
                items(
                  items = state.recentlyAddedAlbums,
                  key = { "recent_${it.id}" },
                ) { album ->
                  CompactAlbumCard(
                    album = album,
                    shape = AsymmetricSoftShape,
                    onClick = { onNavigateToAlbum(Screen.AlbumDetail(album.id, album.name)) },
                    onPlay = {
                      viewModel.playAlbum(album.id)
                      onOpenPlayer()
                    },
                  )
                }
              }
            }
          }

          // From Your Library section header - spans full width
          if (state.randomAlbums.isNotEmpty()) {
            item(span = { GridItemSpan(2) }) {
              LibrarySectionHeader(
                title = "From Your Library",
                modifier = Modifier.padding(top = 20.dp),
              )
            }

            // Random albums in horizontal scroll with squircle shape
            item(span = { GridItemSpan(2) }) {
              LazyRow(
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                contentPadding = PaddingValues(vertical = 4.dp),
              ) {
                items(
                  items = state.randomAlbums,
                  key = { "random_${it.id}" },
                ) { album ->
                  CompactAlbumCard(
                    album = album,
                    shape = SquircleShape,
                    onClick = { onNavigateToAlbum(Screen.AlbumDetail(album.id, album.name)) },
                    onPlay = {
                      viewModel.playAlbum(album.id)
                      onOpenPlayer()
                    },
                  )
                }
              }
            }
          }
        }
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

/**
 * Hero section for "Continue Listening" - large puffy album art with song info below.
 * Uses M3E Puffy shape for playful, expressive album art presentation.
 */
@Composable
private fun ContinueListeningHero(
  song: Song,
  @Suppress("UNUSED_PARAMETER") isCurrentSong: Boolean,
  isPlaying: Boolean,
  onClick: () -> Unit,
  onGoToArtist: (() -> Unit)? = null,
) {
  val colors = MaterialTheme.colorScheme
  val wideFont = rememberGoogleSansFlexWideFont()

  // Press animation
  var isPressed by remember { mutableStateOf(false) }
  val scale = rememberPressScale(isPressed, pressedScale = 0.98f)

  Column(
    modifier = Modifier
      .fillMaxWidth()
      .scale(scale)
      .pointerInput(Unit) {
        detectTapGestures(
          onPress = {
            isPressed = true
            tryAwaitRelease()
            isPressed = false
          },
          onTap = { onClick() }
        )
      }
      .padding(20.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
  ) {
    // Large Puffy album art - center stage, full width
    Box(
      modifier = Modifier
        .fillMaxWidth()
        .aspectRatio(4f / 3f)
        .clip(PuffyShape)
        .background(colors.surfaceVariant),
      contentAlignment = Alignment.Center,
    ) {
      if (song.albumArtUrl.isNullOrBlank()) {
        Box(
          modifier = Modifier
            .fillMaxSize()
            .background(colors.surfaceVariant),
          contentAlignment = Alignment.Center,
        ) {
          Icon(
            imageVector = Icons.Filled.MusicNote,
            contentDescription = null,
            tint = colors.onSurfaceVariant.copy(alpha = 0.4f),
            modifier = Modifier.size(64.dp),
          )
        }
      } else {
        val context = LocalContext.current
        // Use a larger size for full width image
        val pxSize = with(LocalDensity.current) { 400.dp.toPx().toInt() }
        Box(modifier = Modifier.fillMaxSize()) {
          Box(
            modifier = Modifier
              .fillMaxSize()
              .background(colors.surfaceVariant),
          )
          AsyncImage(
            model = ImageRequest.Builder(context)
              .data(optimizedArtworkUrl(song.albumArtUrl, pxSize))
              .crossfade(false)
              .size(pxSize)
              .build(),
            contentDescription = song.name,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
          )
        }
      }
    }

    Spacer(modifier = Modifier.height(16.dp))

    // Continue listening label
    Text(
      text = "Continue listening",
      style = MaterialTheme.typography.labelLarge,
      color = colors.onSurfaceVariant,
    )

    Spacer(modifier = Modifier.height(8.dp))

    // Song name - bold and prominent
    Text(
      text = song.name,
      style = MaterialTheme.typography.headlineMedium.copy(
        fontFamily = wideFont,
      ),
      fontWeight = FontWeight.Black,
      color = colors.onSurface,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
    )

    Spacer(modifier = Modifier.height(4.dp))

    // Artist name
    Text(
      text = song.artists?.joinToString(", ") ?: "Unknown Artist",
      style = MaterialTheme.typography.bodyLarge,
      color = colors.onSurfaceVariant,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
      modifier =
        Modifier.clickable(
          enabled = onGoToArtist != null,
          onClick = { onGoToArtist?.invoke() },
        ),
    )
  }
}

/**
 * Quick pick card - compact song card for the 2-column grid.
 * Uses asymmetric soft shape for M3E playful feel with springy press animation.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun QuickPickCard(
  song: Song,
  isCurrentSong: Boolean,
  @Suppress("UNUSED_PARAMETER") isPlaying: Boolean,
  onClick: () -> Unit,
  onLongClick: () -> Unit,
  showContextMenu: Boolean,
  onDismissMenu: () -> Unit,
  onAddToQueue: () -> Unit,
  onPlayNext: () -> Unit,
  onAddToPlaylist: () -> Unit,
  onGoToAlbum: (() -> Unit)?,
  onGoToArtist: (() -> Unit)?,
) {
  val colors = MaterialTheme.colorScheme

  // Rounded corners match the circular art (8dp padding + 28dp radius = 36dp)
  val cardShape = RoundedCornerShape(36.dp)

  // Press animation
  val interactionSource = remember { MutableInteractionSource() }
  val isPressed by interactionSource.collectIsPressedAsState()
  val scale = rememberPressScale(isPressed)

  // Elevation animation
  val elevation by animateDpAsState(
    targetValue = when {
      isPressed -> 0.dp
      isCurrentSong -> 4.dp
      else -> 2.dp
    },
    animationSpec = spring(
      dampingRatio = Spring.DampingRatioMediumBouncy,
      stiffness = Spring.StiffnessLow
    ),
    label = "quickPickElevation"
  )

  Box {
    Surface(
      modifier = Modifier
        .fillMaxWidth()
        .scale(scale)
        .shadow(elevation, cardShape)
        .combinedClickable(
          interactionSource = interactionSource,
          indication = null,
          onClick = onClick,
          onLongClick = onLongClick,
        ),
      shape = cardShape,
      color = if (isCurrentSong) colors.primaryContainer else colors.surfaceContainerLow,
    ) {
      Row(
        modifier = Modifier
          .fillMaxWidth()
          .height(72.dp)
          .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        // Circular album art
        Box(
          modifier = Modifier
            .size(56.dp)
            .clip(CircleShape),
          contentAlignment = Alignment.Center,
        ) {
          if (song.albumArtUrl.isNullOrBlank()) {
            Box(
              modifier = Modifier
                .fillMaxSize()
                .background(colors.surfaceVariant),
              contentAlignment = Alignment.Center,
            ) {
              Icon(
                imageVector = Icons.Filled.MusicNote,
                contentDescription = null,
                tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
                modifier = Modifier.size(24.dp),
              )
            }
          } else {
            val context = LocalContext.current
            val pxSize = with(LocalDensity.current) { 64.dp.toPx().toInt() }
            Box(modifier = Modifier.fillMaxSize()) {
              Box(
                modifier = Modifier
                  .fillMaxSize()
                  .background(colors.surfaceVariant),
              )
              AsyncImage(
                model = ImageRequest.Builder(context)
                  .data(optimizedArtworkUrl(song.albumArtUrl, pxSize))
                  .crossfade(false)
                  .size(pxSize)
                  .build(),
                contentDescription = song.name,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
              )
            }
          }
        }

        // Song info
        Column(
          modifier = Modifier
            .weight(1f)
            .padding(horizontal = 12.dp, vertical = 8.dp),
          verticalArrangement = Arrangement.Center,
        ) {
          Text(
            text = song.name,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = if (isCurrentSong) FontWeight.Bold else FontWeight.Medium,
            color = if (isCurrentSong) colors.onPrimaryContainer else colors.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
          Text(
            text = song.artists?.firstOrNull() ?: "Unknown",
            style = MaterialTheme.typography.bodySmall,
            color =
              if (isCurrentSong) {
                colors.onPrimaryContainer.copy(alpha = 0.7f)
              } else {
                colors.onSurfaceVariant
              },
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier =
              Modifier.clickable(
                enabled = onGoToArtist != null,
                onClick = { onGoToArtist?.invoke() },
              ),
          )
        }
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
      onGoToArtist = onGoToArtist,
      onToggleFavorite = null,
    )
  }
}

/**
 * Compact album card for horizontal scrolling sections.
 * Accepts a shape parameter for M3E variation per section.
 */
@Composable
private fun CompactAlbumCard(
  album: AlbumItem,
  shape: Shape = SquircleShape,
  onClick: () -> Unit,
  @Suppress("UNUSED_PARAMETER") onPlay: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme

  // Press animation
  var isPressed by remember { mutableStateOf(false) }
  val scale = rememberPressScale(isPressed, pressedScale = 0.97f)

  Surface(
    modifier = Modifier
      .width(140.dp)
      .scale(scale)
      .pointerInput(Unit) {
        detectTapGestures(
          onPress = {
            isPressed = true
            tryAwaitRelease()
            isPressed = false
          },
          onTap = { onClick() }
        )
      },
    shape = shape,
    color = colors.surfaceContainerLow,
    shadowElevation = 2.dp,
  ) {
    Column(
      modifier = Modifier.clip(shape),
    ) {
      // Album artwork - shape clips image properly
      Box(
        modifier = Modifier
          .fillMaxWidth()
          .aspectRatio(1f),
        contentAlignment = Alignment.Center,
      ) {
        if (album.albumArtUrl.isNullOrBlank()) {
          Icon(
            imageVector = Icons.Filled.Album,
            contentDescription = null,
            tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
            modifier = Modifier.fillMaxSize(0.4f),
          )
        } else {
          val context = LocalContext.current
          val pxSize = with(LocalDensity.current) { 140.dp.toPx().toInt() }
          Box(modifier = Modifier.fillMaxSize()) {
            Box(
              modifier = Modifier
                .fillMaxSize()
                .background(colors.surfaceVariant),
            )
            AsyncImage(
              model = ImageRequest.Builder(context)
                .data(optimizedArtworkUrl(album.albumArtUrl, pxSize))
                .crossfade(false)
                .size(pxSize)
                .build(),
              contentDescription = album.name,
              modifier = Modifier.fillMaxSize(),
              contentScale = ContentScale.Crop,
            )
          }
        }
      }

      // Album info
      Column(
        modifier = Modifier.padding(10.dp),
      ) {
        Text(
          text = album.name,
          style = MaterialTheme.typography.bodySmall,
          fontWeight = FontWeight.SemiBold,
          color = colors.onSurface,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
        Text(
          text = album.artist,
          style = MaterialTheme.typography.labelSmall,
          color = colors.onSurfaceVariant,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
      }
    }
  }
}
