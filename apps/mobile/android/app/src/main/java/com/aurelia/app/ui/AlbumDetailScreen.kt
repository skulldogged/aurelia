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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Album
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import coil.compose.SubcomposeAsyncImage
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.QueueItem
import com.aurelia.app.storage.SessionStore
import uniffi.aurelia_core.buildStreamUrl

@Composable
fun AlbumDetailScreen(
    albumId: String,
    albumName: String,
    sessionStore: SessionStore,
    playerController: PlayerController,
    onBack: () -> Unit,
    onOpenPlayer: () -> Unit,
) {
    val libraryViewModel: LibraryViewModel =
        viewModel(
            factory = LibraryViewModelFactory(sessionStore, playerController),
        )
    val state by libraryViewModel.state.collectAsState()
    val colors = MaterialTheme.colorScheme

    LaunchedEffect(Unit) {
        libraryViewModel.loadLibrary()
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
            contentPadding = PaddingValues(bottom = 16.dp),
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
                                .size(200.dp)
                                .clip(RoundedCornerShape(24.dp)),
                        shape = RoundedCornerShape(24.dp),
                        color = colors.surfaceVariant,
                        tonalElevation = 8.dp,
                    ) {
                        if (albumArtUrl.isNullOrBlank()) {
                            Box(
                                modifier = Modifier.fillMaxSize(),
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Album,
                                    contentDescription = null,
                                    modifier = Modifier.size(64.dp),
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

                    Spacer(modifier = Modifier.height(16.dp))

                    // Album name with display font for impact
                    Text(
                        text = albumName,
                        style = MaterialTheme.typography.headlineMedium,
                        fontWeight = FontWeight.Bold,
                        color = colors.onPrimaryContainer,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )

                    Spacer(modifier = Modifier.height(4.dp))

                    // Artist name
                    Text(
                        text = artistName,
                        style = MaterialTheme.typography.bodyLarge,
                        color = colors.onPrimaryContainer.copy(alpha = 0.7f),
                    )

                    // Song count
                    Text(
                        text = "${albumSongs.size} songs",
                        style = MaterialTheme.typography.bodySmall,
                        color = colors.onPrimaryContainer.copy(alpha = 0.5f),
                    )

                    Spacer(modifier = Modifier.height(16.dp))

                    // Play all button
                    FilledIconButton(
                        onClick = {
                            if (albumSongs.isNotEmpty()) {
                                val serverUrl = sessionStore.getServerUrl() ?: return@FilledIconButton
                                val token = sessionStore.getToken() ?: return@FilledIconButton

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
                                        )
                                    }
                                playerController.setQueue(queueItems, 0)
                                onOpenPlayer()
                            }
                        },
                        modifier = Modifier.size(56.dp),
                        shape = CircleShape,
                        colors =
                            IconButtonDefaults.filledIconButtonColors(
                                containerColor = colors.primary,
                                contentColor = colors.onPrimary,
                            ),
                    ) {
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = "Play all",
                            modifier = Modifier.size(28.dp),
                        )
                    }

                    Spacer(modifier = Modifier.height(24.dp))
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
                    trackNumber = song.trackNumber ?: (index + 1),
                    title = song.name,
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
                                )
                            }
                        playerController.setQueue(queueItems, index)
                        onOpenPlayer()
                    },
                )
            }
        }
    }
}

@Composable
private fun AlbumSongItem(
    trackNumber: Int,
    title: String,
    duration: String?,
    isPlaying: Boolean,
    isCurrentSong: Boolean,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val containerColor =
        if (isCurrentSong) colors.primaryContainer.copy(alpha = 0.5f) else colors.surface.copy(alpha = 0f)

    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = containerColor,
        onClick = onClick,
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp),
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

            // Song title
            Text(
                text = title,
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
        }
    }
}

private fun formatDuration(durationMs: Long): String {
    val totalSeconds = durationMs / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "%d:%02d".format(minutes, seconds)
}
