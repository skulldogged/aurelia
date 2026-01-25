package com.aurelia.app.ui

import androidx.compose.foundation.clickable
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Person
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.BottomBarDimensions
import com.aurelia.app.ui.navigation.Screen

data class ArtistItem(
  val id: String,
  val name: String,
  val songCount: Int,
  val albumCount: Int,
)

@Composable
fun ArtistsScreen(
  sessionStore: SessionStore,
  playerController: PlayerController,
  onNavigateToArtist: (Screen.ArtistDetail) -> Unit = {},
  hasPlayerBar: Boolean = false,
) {
  val libraryViewModel: LibraryViewModel = viewModel(
    factory = LibraryViewModelFactory(sessionStore, playerController),
  )
  val state by libraryViewModel.state.collectAsState()
  val colors = MaterialTheme.colorScheme
  val bottomPadding = BottomBarDimensions.calculateBottomPadding(hasPlayerBar)

  LaunchedEffect(Unit) {
    libraryViewModel.loadLibrary()
  }

  // Group songs by artist (using artist ID)
  val artists = remember(state.songs) {
    val artistData =
      mutableMapOf<String, Triple<String, MutableList<uniffi.aurelia_core.Song>, MutableSet<String>>>()

    for (song in state.songs) {
      val artistNames = song.artists ?: listOf("Unknown Artist")
      val artistIds = song.artistIds ?: emptyList()

      for ((index, artistName) in artistNames.withIndex()) {
        val artistId = artistIds.getOrNull(index) ?: artistName // Fall back to name as ID
        val data =
          artistData.getOrPut(artistId) { Triple(artistName, mutableListOf(), mutableSetOf()) }
        data.second.add(song)
        song.albumId?.let { data.third.add(it) }
      }
    }

    artistData.map { (id, data) ->
      ArtistItem(
        id = id,
        name = data.first,
        songCount = data.second.size,
        albumCount = data.third.size,
      )
    }.sortedBy { it.name.lowercase() }
  }

  Column(
    modifier = Modifier
      .fillMaxSize()
      .statusBarsPadding(),
  ) {
    // Header
    Column(
      modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 24.dp, vertical = 16.dp),
    ) {
      Text(
        text = "Artists",
        style = MaterialTheme.typography.displayLarge,
        color = colors.onBackground,
      )
      Text(
        text = "${artists.size} artists",
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
        LazyColumn(
          modifier = Modifier.fillMaxSize(),
          contentPadding = PaddingValues(
            start = 16.dp,
            end = 16.dp,
            top = 8.dp,
            bottom = bottomPadding,
          ),
          verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
          items(
            items = artists,
            key = { it.id },
          ) { artist ->
            ArtistCard(
              artist = artist,
              onClick = {
                onNavigateToArtist(Screen.ArtistDetail(artist.id, artist.name))
              },
            )
          }
        }
      }
    }
  }
}

@Composable
private fun ArtistCard(
  artist: ArtistItem,
  onClick: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme

  Surface(
    modifier = Modifier
      .fillMaxWidth()
      .clip(RoundedCornerShape(16.dp))
      .clickable(onClick = onClick),
    shape = RoundedCornerShape(16.dp),
    color = colors.surfaceContainerLow,
  ) {
    Row(
      modifier = Modifier
        .fillMaxWidth()
        .padding(12.dp),
      verticalAlignment = Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
      // Artist avatar placeholder
      Surface(
        modifier = Modifier.size(56.dp),
        shape = CircleShape,
        color = colors.surfaceVariant,
      ) {
        Box(contentAlignment = Alignment.Center) {
          Icon(
            imageVector = Icons.Filled.Person,
            contentDescription = null,
            tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
            modifier = Modifier.size(28.dp),
          )
        }
      }

      // Artist info
      Column(modifier = Modifier.weight(1f)) {
        Text(
          text = artist.name,
          style = MaterialTheme.typography.titleMedium,
          fontWeight = FontWeight.SemiBold,
          color = colors.onSurface,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
        Text(
          text = "${artist.songCount} songs · ${artist.albumCount} albums",
          style = MaterialTheme.typography.bodySmall,
          color = colors.onSurfaceVariant,
        )
      }
    }
  }
}
