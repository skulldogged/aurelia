package com.aurelia.app.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ExitToApp
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Palette
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.aurelia.app.storage.SessionStore

// Heights matching MainScreen
private val MiniPlayerHeight = 64.dp
private val NavBarContentHeight = 90.dp

@Composable
fun SettingsScreen(
  sessionStore: SessionStore,
  settingsViewModel: SettingsViewModel,
  onLogout: () -> Unit,
  hasPlayerBar: Boolean = false,
) {
  val colors = MaterialTheme.colorScheme
  val settingsState by settingsViewModel.state.collectAsState()
  var showLogoutDialog by remember { mutableStateOf(false) }
  var showClearCacheDialog by remember { mutableStateOf(false) }
  var useDynamicColor by remember { mutableStateOf(sessionStore.getUseDynamicColor()) }
  val snackbarHostState = remember { SnackbarHostState() }

  // Show snackbar for sync/clear results
  LaunchedEffect(settingsState.syncSuccess, settingsState.clearSuccess, settingsState.error) {
    when {
      settingsState.syncSuccess == true -> {
        snackbarHostState.showSnackbar("Library synced successfully")
        settingsViewModel.clearMessages()
      }

      settingsState.clearSuccess == true -> {
        snackbarHostState.showSnackbar("Cache cleared successfully")
        settingsViewModel.clearMessages()
      }

      settingsState.error != null -> {
        snackbarHostState.showSnackbar(settingsState.error ?: "An error occurred")
        settingsViewModel.clearMessages()
      }
    }
  }

  // Calculate bottom padding to avoid navbar/playerbar overlap
  val systemNavBarInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
  val bottomPadding =
    NavBarContentHeight + systemNavBarInset + 24.dp +
      (if (hasPlayerBar) MiniPlayerHeight + 12.dp else 0.dp)

  if (showClearCacheDialog) {
    AlertDialog(
      onDismissRequest = { showClearCacheDialog = false },
      title = { Text("Clear cache?") },
      text = { Text("This will remove all locally stored song data. You'll need to sync again to restore it.") },
      confirmButton = {
        Button(
          onClick = {
            showClearCacheDialog = false
            settingsViewModel.clearLocalCache()
          },
          colors =
            ButtonDefaults.buttonColors(
              containerColor = colors.error,
              contentColor = colors.onError,
            ),
        ) {
          Text("Clear")
        }
      },
      dismissButton = {
        TextButton(onClick = { showClearCacheDialog = false }) {
          Text("Cancel")
        }
      },
    )
  }

  if (showLogoutDialog) {
    AlertDialog(
      onDismissRequest = { showLogoutDialog = false },
      title = { Text("Log out?") },
      text = { Text("You'll need to sign in again to access your library.") },
      confirmButton = {
        Button(
          onClick = {
            showLogoutDialog = false
            onLogout()
          },
          colors =
            ButtonDefaults.buttonColors(
              containerColor = colors.error,
              contentColor = colors.onError,
            ),
        ) {
          Text("Log out")
        }
      },
      dismissButton = {
        TextButton(onClick = { showLogoutDialog = false }) {
          Text("Cancel")
        }
      },
    )
  }

  androidx.compose.foundation.layout.Box(
    modifier =
      Modifier
        .fillMaxSize()
        .statusBarsPadding(),
  ) {
    Column(modifier = Modifier.fillMaxSize()) {
      // Header (matching other screens like LibraryScreen)
      Column(
        modifier =
          Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp, vertical = 16.dp),
      ) {
        Text(
          text = "Settings",
          style = MaterialTheme.typography.displayLarge,
          color = colors.onBackground,
        )
        Text(
          text = "App preferences",
          style = MaterialTheme.typography.bodyMedium,
          color = colors.onSurfaceVariant,
        )
      }

      Column(
        modifier =
          Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = bottomPadding),
        verticalArrangement = Arrangement.spacedBy(16.dp),
      ) {
        // Appearance section
        SettingsSection(title = "Appearance") {
          SettingsToggleItem(
            icon = Icons.Filled.Palette,
            title = "Material You",
            subtitle = "Use system color palette",
            checked = useDynamicColor,
            onCheckedChange = {
              useDynamicColor = it
              sessionStore.setUseDynamicColor(it)
            },
          )
        }

        // Library section
        SettingsSection(title = "Library") {
          val lastSyncedText = when {
            settingsState.isSyncing -> "Syncing..."
            settingsState.lastSyncTime != null -> "Last synced ${SettingsViewModel.formatRelativeTime(settingsState.lastSyncTime)}"
            else -> "Never synced"
          }
          SettingsActionItem(
            icon = Icons.Filled.Sync,
            title = "Sync library",
            subtitle = lastSyncedText,
            isLoading = settingsState.isSyncing,
            onClick = { if (!settingsState.isSyncing) settingsViewModel.syncLibrary() },
          )
          HorizontalDivider(
            modifier = Modifier.padding(start = 56.dp),
            color = colors.outline.copy(alpha = 0.2f),
          )
          SettingsActionItem(
            icon = Icons.Filled.Delete,
            title = "Clear cache",
            subtitle = if (settingsState.isClearing) "Clearing..." else "Remove locally stored data",
            isLoading = settingsState.isClearing,
            onClick = { if (!settingsState.isClearing) showClearCacheDialog = true },
          )
        }

        // Auto-sync section
        SettingsSection(title = "Background Sync") {
          val context = androidx.compose.ui.platform.LocalContext.current
          SettingsToggleItem(
            icon = Icons.Filled.Sync,
            title = "Auto-sync",
            subtitle = "Sync library automatically on WiFi",
            checked = settingsState.autoSyncEnabled,
            onCheckedChange = { settingsViewModel.setAutoSyncEnabled(context, it) },
          )
          if (settingsState.autoSyncEnabled) {
            HorizontalDivider(
              modifier = Modifier.padding(start = 56.dp),
              color = colors.outline.copy(alpha = 0.2f),
            )
            val intervalText = when (settingsState.syncIntervalHours) {
              6L -> "Every 6 hours"
              12L -> "Every 12 hours"
              24L -> "Daily"
              168L -> "Weekly"
              else -> "Every ${settingsState.syncIntervalHours}h"
            }
            var showIntervalPicker by remember { mutableStateOf(false) }
            SettingsActionItem(
              icon = Icons.Filled.Storage,
              title = "Sync frequency",
              subtitle = intervalText,
              onClick = { showIntervalPicker = true },
            )
            if (showIntervalPicker) {
              AlertDialog(
                onDismissRequest = { showIntervalPicker = false },
                title = { Text("Sync frequency") },
                text = {
                  Column {
                    listOf(6L to "Every 6 hours", 12L to "Every 12 hours", 24L to "Daily", 168L to "Weekly").forEach { (hours, label) ->
                      TextButton(
                        onClick = {
                          settingsViewModel.setSyncInterval(context, hours)
                          showIntervalPicker = false
                        },
                        modifier = Modifier.fillMaxWidth(),
                      ) {
                        Text(
                          text = label,
                          style = MaterialTheme.typography.bodyLarge,
                          color = if (settingsState.syncIntervalHours == hours) colors.primary else colors.onSurface,
                        )
                      }
                    }
                  }
                },
                confirmButton = {
                  TextButton(onClick = { showIntervalPicker = false }) {
                    Text("Cancel")
                  }
                },
              )
            }
          }
        }

        // Server section
        SettingsSection(title = "Server") {
          val serverUrl = sessionStore.getServerUrl() ?: "Not connected"
          val username = sessionStore.getUserId() ?: "Unknown"

          SettingsInfoItem(
            icon = Icons.Filled.Storage,
            title = "Connected to",
            subtitle = serverUrl,
          )
          HorizontalDivider(
            modifier = Modifier.padding(start = 56.dp),
            color = colors.outline.copy(alpha = 0.2f),
          )
          SettingsInfoItem(
            icon = Icons.Filled.Info,
            title = "Logged in as",
            subtitle = username,
          )
          HorizontalDivider(
            modifier = Modifier.padding(start = 56.dp),
            color = colors.outline.copy(alpha = 0.2f),
          )

          var lyricsServerUrl by remember {
            mutableStateOf(sessionStore.getLyricsServerUrl() ?: "")
          }
          Row(
            modifier = Modifier
              .fillMaxWidth()
              .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp),
          ) {
            Icon(
              imageVector = Icons.Filled.Storage,
              contentDescription = null,
              tint = colors.onSurfaceVariant,
              modifier = Modifier.size(24.dp),
            )
            Column(modifier = Modifier.weight(1f)) {
              Text(
                text = "Lyrics Server",
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
                color = colors.onSurface,
              )
              androidx.compose.material3.OutlinedTextField(
                value = lyricsServerUrl,
                onValueChange = {
                  lyricsServerUrl = it
                  sessionStore.setLyricsServerUrl(it.ifBlank { null })
                },
                placeholder = { Text("http://localhost:3030") },
                singleLine = true,
                textStyle = MaterialTheme.typography.bodySmall.copy(color = colors.onSurface),
                modifier = Modifier.fillMaxWidth(),
              )
              Text(
                text = "For synced lyrics from sidecar files (daemon URL)",
                style = MaterialTheme.typography.bodySmall,
                color = colors.onSurfaceVariant,
              )
            }
          }
        }

        // Account section
        SettingsSection(title = "Account") {
          SettingsActionItem(
            icon = Icons.AutoMirrored.Filled.ExitToApp,
            title = "Log out",
            subtitle = "Sign out from this device",
            isDestructive = true,
            onClick = { showLogoutDialog = true },
          )
        }

        // About section
        SettingsSection(title = "About") {
          SettingsInfoItem(
            icon = Icons.Filled.Info,
            title = "Version",
            subtitle = "1.0.0",
          )
        }
      }
    }

    SnackbarHost(
      hostState = snackbarHostState,
      modifier = Modifier
        .align(Alignment.BottomCenter)
        .padding(bottom = bottomPadding),
    )
  }
}

@Composable
private fun SettingsSection(
  title: String,
  content: @Composable () -> Unit,
) {
  val colors = MaterialTheme.colorScheme

  Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
    Text(
      text = title,
      style = MaterialTheme.typography.titleSmall,
      fontWeight = FontWeight.SemiBold,
      color = colors.primary,
      modifier = Modifier.padding(horizontal = 8.dp),
    )
    Surface(
      shape = RoundedCornerShape(16.dp),
      color = colors.surfaceVariant.copy(alpha = 0.5f),
      tonalElevation = 1.dp,
    ) {
      Column(modifier = Modifier.fillMaxWidth()) {
        content()
      }
    }
  }
}

@Composable
private fun SettingsToggleItem(
  icon: ImageVector,
  title: String,
  subtitle: String,
  checked: Boolean,
  onCheckedChange: (Boolean) -> Unit,
) {
  val colors = MaterialTheme.colorScheme

  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .clickable { onCheckedChange(!checked) }
        .padding(16.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    Icon(
      imageVector = icon,
      contentDescription = null,
      tint = colors.onSurfaceVariant,
      modifier = Modifier.size(24.dp),
    )
    Column(modifier = Modifier.weight(1f)) {
      Text(
        text = title,
        style = MaterialTheme.typography.bodyLarge,
        fontWeight = FontWeight.Medium,
        color = colors.onSurface,
      )
      Text(
        text = subtitle,
        style = MaterialTheme.typography.bodySmall,
        color = colors.onSurfaceVariant,
      )
    }
    Switch(
      checked = checked,
      onCheckedChange = onCheckedChange,
      colors =
        SwitchDefaults.colors(
          checkedThumbColor = colors.primary,
          checkedTrackColor = colors.primaryContainer,
        ),
    )
  }
}

@Composable
private fun SettingsActionItem(
  icon: ImageVector,
  title: String,
  subtitle: String,
  isDestructive: Boolean = false,
  isLoading: Boolean = false,
  onClick: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme
  val contentColor = if (isDestructive) colors.error else colors.onSurface

  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .clickable(enabled = !isLoading, onClick = onClick)
        .padding(16.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    Icon(
      imageVector = icon,
      contentDescription = null,
      tint = if (isDestructive) colors.error else colors.onSurfaceVariant,
      modifier = Modifier.size(24.dp),
    )
    Column(modifier = Modifier.weight(1f)) {
      Text(
        text = title,
        style = MaterialTheme.typography.bodyLarge,
        fontWeight = FontWeight.Medium,
        color = contentColor,
      )
      Text(
        text = subtitle,
        style = MaterialTheme.typography.bodySmall,
        color = if (isDestructive) colors.error.copy(alpha = 0.7f) else colors.onSurfaceVariant,
      )
    }
    if (isLoading) {
      CircularProgressIndicator(
        modifier = Modifier.size(20.dp),
        strokeWidth = 2.dp,
        color = colors.primary,
      )
    } else {
      Icon(
        imageVector = Icons.Filled.ChevronRight,
        contentDescription = null,
        tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
        modifier = Modifier.size(20.dp),
      )
    }
  }
}

@Composable
private fun SettingsInfoItem(
  icon: ImageVector,
  title: String,
  subtitle: String,
) {
  val colors = MaterialTheme.colorScheme

  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .padding(16.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(16.dp),
  ) {
    Icon(
      imageVector = icon,
      contentDescription = null,
      tint = colors.onSurfaceVariant,
      modifier = Modifier.size(24.dp),
    )
    Column(modifier = Modifier.weight(1f)) {
      Text(
        text = title,
        style = MaterialTheme.typography.bodyLarge,
        fontWeight = FontWeight.Medium,
        color = colors.onSurface,
      )
      Text(
        text = subtitle,
        style = MaterialTheme.typography.bodySmall,
        color = colors.onSurfaceVariant,
      )
    }
  }
}
