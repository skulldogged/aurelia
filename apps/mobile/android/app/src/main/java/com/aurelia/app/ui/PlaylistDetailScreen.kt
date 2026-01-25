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
import androidx.compose.material.icons.automirrored.filled.PlaylistPlay
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Shuffle
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AlbumArtStyle

@Composable
fun PlaylistDetailScreen(
    playlistId: String,
    playlistName: String,
    viewModel: PlaylistViewModel,
    onBack: () -> Unit,
    onOpenPlayer: () -> Unit,
) {
    val state by viewModel.detailState.collectAsState()
    val colors = MaterialTheme.colorScheme

    LaunchedEffect(playlistId) {
        viewModel.loadPlaylistDetail(playlistId, playlistName)
    }

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
                    Text(
                        text = state.error ?: "An error occurred",
                        style = MaterialTheme.typography.bodyLarge,
                        color = colors.error,
                        textAlign = TextAlign.Center,
                    )
                }
            }

            else -> {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(bottom = 16.dp),
                ) {
                    // Playlist header
                    item {
                        Column(
                            modifier =
                                Modifier
                                    .fillMaxWidth()
                                    .padding(horizontal = 24.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            // Playlist art placeholder
                            Surface(
                                modifier =
                                    Modifier
                                        .size(200.dp)
                                        .clip(RoundedCornerShape(24.dp)),
                                shape = RoundedCornerShape(24.dp),
                                color = colors.primaryContainer,
                                tonalElevation = 8.dp,
                            ) {
                                Box(
                                    modifier = Modifier.fillMaxSize(),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    Icon(
                                        imageVector = Icons.AutoMirrored.Filled.PlaylistPlay,
                                        contentDescription = null,
                                        modifier = Modifier.size(64.dp),
                                        tint = colors.onPrimaryContainer.copy(alpha = 0.5f),
                                    )
                                }
                            }

                            Spacer(modifier = Modifier.height(16.dp))

                            // Playlist name
                            Text(
                                text = playlistName,
                                style = MaterialTheme.typography.headlineMedium,
                                fontWeight = FontWeight.Bold,
                                color = colors.onPrimaryContainer,
                                maxLines = 2,
                                overflow = TextOverflow.Ellipsis,
                            )

                            Spacer(modifier = Modifier.height(4.dp))

                            // Song count
                            Text(
                                text = "${state.songs.size} songs",
                                style = MaterialTheme.typography.bodySmall,
                                color = colors.onPrimaryContainer.copy(alpha = 0.5f),
                            )

                            Spacer(modifier = Modifier.height(16.dp))

                            // Play and shuffle buttons
                            Row(
                                horizontalArrangement = Arrangement.spacedBy(16.dp),
                            ) {
                                // Play all button
                                FilledIconButton(
                                    onClick = {
                                        if (state.songs.isNotEmpty()) {
                                            viewModel.playPlaylist(0)
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

                                // Shuffle button
                                FilledIconButton(
                                    onClick = {
                                        if (state.songs.isNotEmpty()) {
                                            viewModel.shufflePlaylist()
                                            onOpenPlayer()
                                        }
                                    },
                                    modifier = Modifier.size(56.dp),
                                    shape = CircleShape,
                                    colors =
                                        IconButtonDefaults.filledIconButtonColors(
                                            containerColor = colors.secondaryContainer,
                                            contentColor = colors.onSecondaryContainer,
                                        ),
                                ) {
                                    Icon(
                                        imageVector = Icons.Filled.Shuffle,
                                        contentDescription = "Shuffle",
                                        modifier = Modifier.size(24.dp),
                                    )
                                }
                            }

                            Spacer(modifier = Modifier.height(24.dp))
                        }
                    }

                    // Song list
                    if (state.songs.isEmpty()) {
                        item {
                            Box(
                                modifier =
                                    Modifier
                                        .fillMaxWidth()
                                        .padding(32.dp),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    text = "This playlist is empty",
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = colors.onSurfaceVariant,
                                )
                            }
                        }
                    } else {
                        itemsIndexed(
                            items = state.songs,
                            key = { index, song -> "${song.id}_$index" },
                        ) { index, song ->
                            PlaylistSongItem(
                                title = song.name,
                                artist = song.artists?.joinToString(", ") ?: "Unknown Artist",
                                albumArtUrl = song.albumArtUrl,
                                duration = song.duration?.let { formatDuration((it * 1000).toLong()) },
                                onClick = {
                                    viewModel.playPlaylist(index)
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

@Composable
private fun PlaylistSongItem(
    title: String,
    artist: String,
    albumArtUrl: String?,
    duration: String?,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme

    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = colors.surface.copy(alpha = 0f),
        onClick = onClick,
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            AlbumArt(
                imageUrl = albumArtUrl,
                size = 48.dp,
                cornerRadius = 8.dp,
                style = AlbumArtStyle.Song,
                containerColor = colors.surfaceVariant,
                contentColor = colors.onSurfaceVariant.copy(alpha = 0.5f),
            )

            Spacer(modifier = Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Medium,
                    color = colors.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = artist,
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

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
