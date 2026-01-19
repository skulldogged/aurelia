package com.aurelia.app.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.PlaylistPlay
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore

// Heights matching MainScreen
private val MiniPlayerHeight = 64.dp
private val NavBarContentHeight = 90.dp

@Composable
fun PlaylistsScreen(
    sessionStore: SessionStore,
    playerController: PlayerController,
    onOpenPlayer: () -> Unit,
    hasPlayerBar: Boolean = false,
) {
    val colors = MaterialTheme.colorScheme

    // Calculate bottom padding to float FAB above navbar/playerbar
    val systemNavBarInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
    val fabBottomPadding =
        NavBarContentHeight + systemNavBarInset + 24.dp + // navbar + system + spacing
            (if (hasPlayerBar) MiniPlayerHeight + 12.dp else 0.dp) // extra padding when player bar is visible

    Box(
        modifier =
            Modifier
                .fillMaxSize()
                .statusBarsPadding(),
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            // Header
            Column(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 24.dp, vertical = 16.dp),
            ) {
                Text(
                    text = "Playlists",
                    style = MaterialTheme.typography.displayLarge,
                    fontWeight = FontWeight.Bold,
                    color = colors.onBackground,
                )
                Text(
                    text = "Your music collections",
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.onSurfaceVariant,
                )
            }

            // Empty state
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
                                imageVector = Icons.AutoMirrored.Filled.PlaylistPlay,
                                contentDescription = null,
                                tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
                                modifier = Modifier.size(40.dp),
                            )
                        }
                    }
                    Text(
                        text = "No playlists yet",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.onSurface,
                    )
                    Text(
                        text = "Create a playlist to organize your favorite music",
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.onSurfaceVariant,
                        textAlign = TextAlign.Center,
                    )
                }
            }
        }

        // FAB to create playlist - positioned above navbar/playerbar
        FloatingActionButton(
            onClick = { /* TODO: Create playlist */ },
            modifier =
                Modifier
                    .align(Alignment.BottomEnd)
                    .padding(end = 16.dp, bottom = fabBottomPadding),
            containerColor = colors.primary,
            contentColor = colors.onPrimary,
        ) {
            Icon(
                imageVector = Icons.Filled.Add,
                contentDescription = "Create playlist",
            )
        }
    }
}
