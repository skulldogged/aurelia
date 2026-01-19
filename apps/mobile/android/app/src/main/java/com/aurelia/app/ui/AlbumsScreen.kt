package com.aurelia.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Album
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
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
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import coil.compose.SubcomposeAsyncImage
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.navigation.Screen

data class AlbumItem(
    val id: String,
    val name: String,
    val artist: String,
    val albumArtUrl: String?,
    val songCount: Int,
)

@Composable
fun AlbumsScreen(
    sessionStore: SessionStore,
    playerController: PlayerController,
    onNavigateToAlbum: (Screen.AlbumDetail) -> Unit = {},
    hasPlayerBar: Boolean = false,
) {
    val libraryViewModel: LibraryViewModel =
        viewModel(
            factory = LibraryViewModelFactory(sessionStore, playerController),
        )
    val state by libraryViewModel.state.collectAsState()
    val colors = MaterialTheme.colorScheme
    val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

    LaunchedEffect(Unit) {
        libraryViewModel.loadLibrary()
    }

    // Group songs by album
    val albums =
        remember(state.songs) {
            state.songs
                .filter { it.albumId != null }
                .groupBy { it.albumId }
                .map { (albumId, songs) ->
                    val firstSong = songs.first()
                    AlbumItem(
                        id = albumId ?: "",
                        name = firstSong.album ?: "Unknown Album",
                        artist = firstSong.artists?.firstOrNull() ?: "Unknown Artist",
                        albumArtUrl = firstSong.albumArtUrl,
                        songCount = songs.size,
                    )
                }.sortedBy { it.name.lowercase() }
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
                text = "Albums",
                style = MaterialTheme.typography.displayLarge,
                fontWeight = FontWeight.Bold,
                color = colors.onBackground,
            )
            Text(
                text = "${albums.size} albums",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
            )
        }

        when {
            state.isLoading -> {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator(color = colors.primary)
                }
            }

            else -> {
                LazyVerticalGrid(
                    columns = GridCells.Fixed(2),
                    modifier = Modifier.fillMaxSize(),
                    contentPadding =
                        PaddingValues(
                            start = 16.dp,
                            end = 16.dp,
                            top = 16.dp,
                            bottom = bottomPadding,
                        ),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    items(
                        items = albums,
                        key = { it.id },
                    ) { album ->
                        AlbumCard(
                            album = album,
                            onClick = {
                                onNavigateToAlbum(Screen.AlbumDetail(album.id, album.name))
                            },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun AlbumCard(
    album: AlbumItem,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme

    Surface(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(16.dp))
                .clickable(onClick = onClick),
        shape = RoundedCornerShape(16.dp),
        color = colors.surfaceContainerLow,
    ) {
        Column {
            // Album art
            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .aspectRatio(1f)
                        .clip(RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp))
                        .background(colors.surfaceVariant),
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
            }

            // Album info
            Column(
                modifier = Modifier.padding(12.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                Text(
                    text = album.name,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = album.artist,
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = "${album.songCount} songs",
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.onSurfaceVariant.copy(alpha = 0.7f),
                )
            }
        }
    }
}
