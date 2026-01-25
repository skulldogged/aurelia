package com.aurelia.app.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil.compose.SubcomposeAsyncImage
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.components.PlaylistPickerDialog
import com.aurelia.app.ui.components.SongContextMenu
import com.aurelia.app.ui.navigation.Screen
import com.aurelia.app.ui.theme.ReadingContext
import com.aurelia.app.ui.theme.rememberContextualStyle
import uniffi.aurelia_core.Song
import uniffi.aurelia_core.buildStreamUrl
import java.util.Calendar

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
    val state by viewModel.state.collectAsState()
    val playlistState by playlistViewModel.state.collectAsState()
    val colors = MaterialTheme.colorScheme
    val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

    // Context menu state
    var selectedSong by remember { mutableStateOf<Song?>(null) }
    var showContextMenu by remember { mutableStateOf(false) }
    var showPlaylistPicker by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        viewModel.loadHomeData()
        playlistViewModel.loadPlaylists()
    }

    // Time-based greeting
    val greeting =
        remember {
            when (Calendar.getInstance().get(Calendar.HOUR_OF_DAY)) {
                in 5..11 -> "Good morning"
                in 12..17 -> "Good afternoon"
                else -> "Good evening"
            }
        }

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
                        .take(6)
                }

            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier =
                    Modifier
                        .fillMaxSize()
                        .statusBarsPadding(),
                contentPadding =
                    PaddingValues(
                        start = 20.dp,
                        end = 20.dp,
                        top = 16.dp,
                        bottom = bottomPadding,
                    ),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                // Greeting header - spans full width with expanded display font
                item(span = { GridItemSpan(2) }) {
                    Column(
                        modifier = Modifier.padding(bottom = 8.dp),
                    ) {
                        Text(
                            text = greeting,
                            style = rememberContextualStyle(
                                baseStyle = MaterialTheme.typography.headlineLarge,
                                context = ReadingContext.DISPLAY,
                            ),
                            color = colors.onBackground,
                        )
                    }
                }

                // Quick Picks section header - spans full width with scanning context
                if (state.mostPlayed.isNotEmpty() || state.recentlyPlayed.isNotEmpty()) {
                    item(span = { GridItemSpan(2) }) {
                        Text(
                            text = "Quick Picks",
                            style = rememberContextualStyle(
                                baseStyle = MaterialTheme.typography.titleLarge,
                                context = ReadingContext.SCANNING,
                            ),
                            fontWeight = FontWeight.Bold,
                            color = colors.onBackground,
                            modifier = Modifier.padding(top = 8.dp),
                        )
                    }
                }

                // Quick picks - compact song cards in 2-column grid
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
                        onLongClick = {
                            selectedSong = song
                            showContextMenu = true
                        },
                        showContextMenu = showContextMenu && selectedSong?.id == song.id,
                        onDismissMenu = { showContextMenu = false },
                        onAddToQueue = {
                            val serverUrl = sessionStore.getServerUrl() ?: return@QuickPickCard
                            val token = sessionStore.getToken() ?: return@QuickPickCard
                            playerController.addToQueue(
                                QueueItem(
                                    id = song.id,
                                    uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                                    title = song.name,
                                    artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                                    albumArtUrl = song.albumArtUrl,
                                    durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                                    isFavorite = song.isFavorite ?: false,
                                ),
                            )
                        },
                        onPlayNext = {
                            val serverUrl = sessionStore.getServerUrl() ?: return@QuickPickCard
                            val token = sessionStore.getToken() ?: return@QuickPickCard
                            playerController.playNext(
                                QueueItem(
                                    id = song.id,
                                    uri = buildStreamUrl(serverUrl, token, song.id, song.container),
                                    title = song.name,
                                    artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                                    albumArtUrl = song.albumArtUrl,
                                    durationMs = (song.duration ?: 0.0).let { (it * 1000).toLong() },
                                    isFavorite = song.isFavorite ?: false,
                                ),
                            )
                        },
                        onAddToPlaylist = {
                            selectedSong = song
                            showPlaylistPicker = true
                        },
                        onGoToAlbum = if (song.albumId != null) {
                            {
                                onNavigateToAlbum(
                                    Screen.AlbumDetail(
                                        albumId = song.albumId!!,
                                        albumName = song.album ?: "Unknown Album",
                                    ),
                                )
                            }
                        } else null,
                        onGoToArtist = if (song.artistIds?.isNotEmpty() == true) {
                            {
                                onNavigateToArtist(
                                    Screen.ArtistDetail(
                                        artistId = song.artistIds!!.first(),
                                        artistName = song.artists?.firstOrNull() ?: "Unknown Artist",
                                    ),
                                )
                            }
                        } else null,
                    )
                }

                // Recently Added Albums section header - spans full width
                if (state.recentlyAddedAlbums.isNotEmpty()) {
                    item(span = { GridItemSpan(2) }) {
                        Text(
                            text = "Recently Added",
                            style = rememberContextualStyle(
                                baseStyle = MaterialTheme.typography.titleLarge,
                                context = ReadingContext.SCANNING,
                            ),
                            fontWeight = FontWeight.Bold,
                            color = colors.onBackground,
                            modifier = Modifier.padding(top = 12.dp),
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
                        Text(
                            text = "From Your Library",
                            style = rememberContextualStyle(
                                baseStyle = MaterialTheme.typography.titleLarge,
                                context = ReadingContext.SCANNING,
                            ),
                            fontWeight = FontWeight.Bold,
                            color = colors.onBackground,
                            modifier = Modifier.padding(top = 12.dp),
                        )
                    }

                    // Random albums in horizontal scroll
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

/**
 * Quick pick card - compact song card for the 2-column grid
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun QuickPickCard(
    song: Song,
    isCurrentSong: Boolean,
    isPlaying: Boolean,
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

    Box {
        Surface(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(16.dp))
                    .combinedClickable(
                        onClick = onClick,
                        onLongClick = onLongClick,
                    ),
            shape = RoundedCornerShape(16.dp),
            color = if (isCurrentSong) colors.primaryContainer else colors.surfaceContainerLow,
        ) {
            Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(64.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            // Album art - square, left aligned
            Box(
                modifier =
                    Modifier
                        .size(64.dp)
                        .clip(RoundedCornerShape(topStart = 16.dp, bottomStart = 16.dp))
                        .background(colors.surfaceVariant),
                contentAlignment = Alignment.Center,
            ) {
                if (song.albumArtUrl.isNullOrBlank()) {
                    Icon(
                        imageVector = Icons.Filled.MusicNote,
                        contentDescription = null,
                        tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
                        modifier = Modifier.size(24.dp),
                    )
                } else {
                    SubcomposeAsyncImage(
                        model = song.albumArtUrl,
                        contentDescription = song.name,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        loading = {
                            Box(
                                modifier =
                                    Modifier
                                        .fillMaxSize()
                                        .background(colors.surfaceVariant),
                            )
                        },
                        error = {
                            Box(
                                modifier =
                                    Modifier
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
                        },
                    )
                }

                // Playing indicator
                if (isPlaying) {
                    Box(
                        modifier =
                            Modifier
                                .fillMaxSize()
                                .background(colors.primary.copy(alpha = 0.8f)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = null,
                            tint = colors.onPrimary,
                            modifier = Modifier.size(24.dp),
                        )
                    }
                }
            }

            // Song info
            Column(
                modifier =
                    Modifier
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
                    color = if (isCurrentSong) colors.onPrimaryContainer.copy(alpha = 0.7f) else colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
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
 * Compact album card for horizontal scrolling sections
 */
@Composable
private fun CompactAlbumCard(
    album: AlbumItem,
    onClick: () -> Unit,
    onPlay: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme

    Surface(
        modifier =
            Modifier
                .width(140.dp)
                .clickable(onClick = onClick),
        shape = RoundedCornerShape(16.dp),
        color = colors.surfaceContainerLow,
    ) {
        Column {
            // Album artwork with play button
            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .aspectRatio(1f)
                        .clip(RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp))
                        .background(colors.surfaceVariant),
            ) {
                if (album.albumArtUrl.isNullOrBlank()) {
                    Box(
                        modifier = Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            imageVector = Icons.Filled.Album,
                            contentDescription = null,
                            tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
                            modifier = Modifier.fillMaxSize(0.4f),
                        )
                    }
                } else {
                    SubcomposeAsyncImage(
                        model = album.albumArtUrl,
                        contentDescription = album.name,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        loading = {
                            Box(
                                modifier =
                                    Modifier
                                        .fillMaxSize()
                                        .background(colors.surfaceVariant),
                            )
                        },
                        error = {
                            Box(
                                modifier =
                                    Modifier
                                        .fillMaxSize()
                                        .background(colors.surfaceVariant),
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Album,
                                    contentDescription = null,
                                    tint = colors.onSurfaceVariant.copy(alpha = 0.3f),
                                    modifier = Modifier.fillMaxSize(0.4f),
                                )
                            }
                        },
                    )
                }

                // Play button overlay
                Box(
                    modifier =
                        Modifier
                            .align(Alignment.BottomEnd)
                            .padding(8.dp)
                            .size(32.dp)
                            .clip(CircleShape)
                            .background(colors.primary)
                            .clickable(onClick = onPlay),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        imageVector = Icons.Filled.PlayArrow,
                        contentDescription = "Play album",
                        tint = colors.onPrimary,
                        modifier = Modifier.size(18.dp),
                    )
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
