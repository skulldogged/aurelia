package com.aurelia.app.ui

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Album
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.MusicNote
import androidx.compose.material.icons.filled.Pause
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.SkipNext
import androidx.compose.material.icons.filled.SkipPrevious
import androidx.compose.material.icons.outlined.Album
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.MusicNote
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AlbumArtStyle
import com.aurelia.app.ui.navigation.Screen
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private val MiniPlayerHeight = 64.dp
private val NavBarContentHeight = 90.dp

data class NavItem(
  val screen: Screen,
  val label: String,
  val selectedIcon: ImageVector,
  val unselectedIcon: ImageVector
)

private val navItems = listOf(
  NavItem(Screen.Home, "Home", Icons.Filled.Home, Icons.Outlined.Home),
  NavItem(Screen.Songs, "Songs", Icons.Filled.MusicNote, Icons.Outlined.MusicNote),
  NavItem(Screen.Albums, "Albums", Icons.Filled.Album, Icons.Outlined.Album),
  NavItem(Screen.Artists, "Artists", Icons.Filled.Person, Icons.Outlined.Person),
  NavItem(Screen.Search, "Search", Icons.Filled.Search, Icons.Outlined.Search),
)

@Composable
fun MainScreen(
  sessionStore: SessionStore,
  playerController: PlayerController,
  onLogout: () -> Unit
) {
  var currentScreen by remember { mutableStateOf<Screen>(Screen.Home) }
  var isPlayerOpen by remember { mutableStateOf(false) }

  val libraryViewModel: LibraryViewModel = viewModel(
    factory = LibraryViewModelFactory(sessionStore, playerController)
  )
  val libraryState by libraryViewModel.state.collectAsState()
  val scope = rememberCoroutineScope()

  BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
    val screenHeightPx = constraints.maxHeight.toFloat()
    val minDragThreshold = screenHeightPx * 0.18f
    val minPlayerOffset = -screenHeightPx
    val playerDragOffset = remember { Animatable(0f) }
    val dragProgress = ((-playerDragOffset.value) / screenHeightPx).coerceIn(0f, 1f)
    val playerScale = 0.96f + (0.04f * dragProgress)
    val playerCornerRadius = (32.dp * (1f - dragProgress))
    val miniPlayerTopMargin = 12.dp
    val miniPlayerBottomMargin = NavBarContentHeight + 12.dp
    val density = LocalDensity.current
    val miniPlayerCollapsedOffset = with(density) {
      (miniPlayerBottomMargin + miniPlayerTopMargin).toPx()
    }
    val playerTranslationY = (1f - dragProgress) * miniPlayerCollapsedOffset

    fun openPlayerAnimated() {
      scope.launch {
        playerDragOffset.animateTo(
          targetValue = minPlayerOffset,
          animationSpec = spring(
            dampingRatio = Spring.DampingRatioNoBouncy,
            stiffness = Spring.StiffnessMediumLow
          )
        )
        isPlayerOpen = true
        playerDragOffset.snapTo(0f)
      }
    }

    fun closePlayer() {
      isPlayerOpen = false
      scope.launch { playerDragOffset.snapTo(0f) }
    }

    fun animatePlayerOffset(target: Float) {
      scope.launch {
        playerDragOffset.animateTo(
          targetValue = target,
          animationSpec = spring(
            dampingRatio = Spring.DampingRatioNoBouncy,
            stiffness = Spring.StiffnessMediumLow
          )
        )
      }
    }

    fun onPlayerDrag(delta: Float) {
      scope.launch {
        playerDragOffset.snapTo((playerDragOffset.value + delta).coerceIn(minPlayerOffset, 0f))
      }
    }

    fun onPlayerDragEnd() {
      val shouldOpen = playerDragOffset.value <= -minDragThreshold
      if (shouldOpen) {
        openPlayerAnimated()
      } else {
        animatePlayerOffset(0f)
      }
    }

    // Main content area
    when {
      isPlayerOpen -> {
        PlayerScreen(
          playerController = playerController,
          onBack = { closePlayer() }
        )
      }
      currentScreen == Screen.Settings -> {
        Box(
          modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
        ) {
          SettingsScreen(
            sessionStore = sessionStore,
            onLogout = onLogout,
            hasPlayerBar = libraryState.nowPlaying != null
          )

          Column(
            modifier = Modifier
              .align(Alignment.BottomCenter)
              .fillMaxWidth()
          ) {
            libraryState.nowPlaying?.let { nowPlaying ->
              MiniPlayerBar(
                title = nowPlaying.title,
                artist = nowPlaying.artist,
                albumArtUrl = nowPlaying.albumArtUrl,
                isPlaying = nowPlaying.isPlaying,
                onPrevious = { libraryViewModel.skipPrevious() },
                onPlayPause = { libraryViewModel.togglePlayPause() },
                onNext = { libraryViewModel.skipNext() },
                onClick = { openPlayerAnimated() },
                onDrag = { delta -> onPlayerDrag(delta) },
                onDragEnd = { onPlayerDragEnd() }
              )
            }

            BottomNavBar(
              items = navItems,
              currentScreen = currentScreen,
              onNavigate = { destination -> currentScreen = destination },
              onSettingsClick = { currentScreen = Screen.Settings },
              hasPlayerBar = libraryState.nowPlaying != null
            )
          }
        }
      }
      currentScreen is Screen.AlbumDetail -> {
        val detail = currentScreen as Screen.AlbumDetail
        AlbumDetailScreen(
          albumId = detail.albumId,
          albumName = detail.albumName,
          sessionStore = sessionStore,
          playerController = playerController,
          onBack = { currentScreen = Screen.Albums },
          onOpenPlayer = { openPlayerAnimated() }
        )
      }
      currentScreen is Screen.ArtistDetail -> {
        val detail = currentScreen as Screen.ArtistDetail
        ArtistDetailScreen(
          artistId = detail.artistId,
          artistName = detail.artistName,
          sessionStore = sessionStore,
          playerController = playerController,
          onBack = { currentScreen = Screen.Artists },
          onOpenPlayer = { openPlayerAnimated() }
        )
      }
      else -> {
        Box(
          modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
        ) {
          when (currentScreen) {
            Screen.Home -> HomeScreen(
              sessionStore = sessionStore,
              playerController = playerController,
              onOpenPlayer = { openPlayerAnimated() },
              onNavigateToAlbum = { currentScreen = it },
              hasPlayerBar = libraryState.nowPlaying != null
            )
            Screen.Songs -> LibraryScreen(
              sessionStore = sessionStore,
              playerController = playerController,
              onOpenPlayer = { openPlayerAnimated() },
              hasPlayerBar = libraryState.nowPlaying != null
            )
            Screen.Albums -> AlbumsScreen(
              sessionStore = sessionStore,
              playerController = playerController,
              onOpenPlayer = { openPlayerAnimated() },
              onNavigateToAlbum = { currentScreen = it },
              hasPlayerBar = libraryState.nowPlaying != null
            )
            Screen.Artists -> ArtistsScreen(
              sessionStore = sessionStore,
              playerController = playerController,
              onOpenPlayer = { openPlayerAnimated() },
              onNavigateToArtist = { currentScreen = it },
              hasPlayerBar = libraryState.nowPlaying != null
            )
            Screen.Playlists -> PlaylistsScreen(
              sessionStore = sessionStore,
              playerController = playerController,
              onOpenPlayer = { openPlayerAnimated() },
              hasPlayerBar = libraryState.nowPlaying != null
            )
            Screen.Search -> SearchScreen(
              sessionStore = sessionStore,
              playerController = playerController,
              onOpenPlayer = { openPlayerAnimated() },
              hasPlayerBar = libraryState.nowPlaying != null
            )
            else -> {}
          }

          Column(
            modifier = Modifier
              .align(Alignment.BottomCenter)
              .fillMaxWidth()
          ) {
            libraryState.nowPlaying?.let { nowPlaying ->
              MiniPlayerBar(
                title = nowPlaying.title,
                artist = nowPlaying.artist,
                albumArtUrl = nowPlaying.albumArtUrl,
                isPlaying = nowPlaying.isPlaying,
                onPrevious = { libraryViewModel.skipPrevious() },
                onPlayPause = { libraryViewModel.togglePlayPause() },
                onNext = { libraryViewModel.skipNext() },
                onClick = { openPlayerAnimated() },
                onDrag = { delta -> onPlayerDrag(delta) },
                onDragEnd = { onPlayerDragEnd() }
              )
            }

            BottomNavBar(
              items = navItems,
              currentScreen = currentScreen,
              onNavigate = { currentScreen = it },
              onSettingsClick = { currentScreen = Screen.Settings },
              hasPlayerBar = libraryState.nowPlaying != null
            )
          }
        }
      }
    }

    if (!isPlayerOpen && libraryState.nowPlaying != null) {
      PlayerScreen(
        playerController = playerController,
        onBack = { closePlayer() },
        modifier = Modifier
          .offset { IntOffset(0, (playerDragOffset.value + screenHeightPx).roundToInt()) }
          .graphicsLayer {
            translationY = playerTranslationY
            scaleX = playerScale
            scaleY = playerScale
            clip = true
            shape = RoundedCornerShape(playerCornerRadius)
          }
      )
    }
  }
}

/**
 * Mini player bar styled after PixelPlayer.
 * Features: 64dp height, circular album art, primaryContainer background,
 * smooth rounded corners, and proper control button styling.
 */
@Composable
fun BottomNavBar(
  items: List<NavItem>,
  currentScreen: Screen,
  onNavigate: (Screen) -> Unit,
  onSettingsClick: () -> Unit,
  hasPlayerBar: Boolean,
  modifier: Modifier = Modifier
) {
  val colors = MaterialTheme.colorScheme
  val bottomInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
  val bottomPadding = if (hasPlayerBar) 8.dp else 12.dp

  Surface(
    modifier = modifier
      .fillMaxWidth()
      .padding(horizontal = 12.dp)
      .padding(bottom = bottomInset + bottomPadding),
    color = colors.surface,
    tonalElevation = 4.dp,
    shadowElevation = 12.dp,
    shape = RoundedCornerShape(32.dp)
  ) {
    Column(
      modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 18.dp, vertical = 12.dp)
    ) {
      Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
      ) {
        items.forEach { item ->
          val isSelected = currentScreen == item.screen
          BottomNavItem(
            label = item.label,
            icon = if (isSelected) item.selectedIcon else item.unselectedIcon,
            selected = isSelected,
            onClick = { onNavigate(item.screen) }
          )
        }

        BottomNavItem(
          label = "Settings",
          icon = if (currentScreen == Screen.Settings) Icons.Filled.Settings else Icons.Outlined.Settings,
          selected = currentScreen == Screen.Settings,
          onClick = onSettingsClick
        )
      }
    }
  }
}

@Composable
private fun RowScope.BottomNavItem(
  label: String,
  icon: ImageVector,
  selected: Boolean,
  onClick: () -> Unit
) {
  val colors = MaterialTheme.colorScheme
  val iconTint by animateColorAsState(
    targetValue = if (selected) colors.primary else colors.onSurfaceVariant,
    animationSpec = tween(durationMillis = 200)
  )
  val textColor by animateColorAsState(
    targetValue = if (selected) colors.primary else colors.onSurfaceVariant,
    animationSpec = tween(durationMillis = 200)
  )

  Column(
    modifier = Modifier
      .weight(1f)
      .clip(RoundedCornerShape(20.dp))
      .clickable(
        interactionSource = remember { MutableInteractionSource() },
        indication = null,
        role = Role.Tab,
        onClick = onClick
      )
      .padding(vertical = 6.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.Center
  ) {
    Box(
      modifier = Modifier
        .size(36.dp)
        .clip(CircleShape)
        .background(colors.primary.copy(alpha = if (selected) 0.2f else 0f)),
      contentAlignment = Alignment.Center
    ) {
      Icon(
        imageVector = icon,
        contentDescription = label,
        tint = iconTint,
        modifier = Modifier.size(20.dp)
      )
    }

    Spacer(modifier = Modifier.height(6.dp))

    ProvideTextStyle(MaterialTheme.typography.labelSmall) {
      Text(
        text = label,
        color = textColor,
        maxLines = 1
      )
    }
  }
}

@Composable
fun MiniPlayerBar(
  title: String,
  artist: String,
  albumArtUrl: String?,
  isPlaying: Boolean,
  onPrevious: () -> Unit,
  onPlayPause: () -> Unit,
  onNext: () -> Unit,
  onClick: () -> Unit,
  onDrag: (Float) -> Unit,
  onDragEnd: () -> Unit,
  modifier: Modifier = Modifier
) {
  val colors = MaterialTheme.colorScheme
  val interactionSource = remember { MutableInteractionSource() }

  Box(
    modifier = modifier
      .fillMaxWidth()
      .padding(horizontal = 12.dp)
      .shadow(
        elevation = 6.dp,
        shape = RoundedCornerShape(32.dp),
        ambientColor = colors.primary.copy(alpha = 0.12f),
        spotColor = colors.primary.copy(alpha = 0.12f),
        clip = false
      )
      .clip(RoundedCornerShape(32.dp))
      .background(colors.primaryContainer)
      .pointerInput(Unit) {
        detectVerticalDragGestures(
          onVerticalDrag = { _, dragAmount ->
            if (dragAmount < 0f) {
              onDrag(dragAmount)
            }
          },
          onDragEnd = { onDragEnd() }
        )
      }
      .clickable(
        interactionSource = interactionSource,
        indication = null,
        onClick = onClick
      )
      .height(MiniPlayerHeight)
      .padding(start = 10.dp, end = 12.dp)
  ) {
    Row(
      modifier = Modifier.fillMaxSize(),
      verticalAlignment = Alignment.CenterVertically
    ) {
      AlbumArt(
        imageUrl = albumArtUrl,
        size = 44.dp,
        cornerRadius = 22.dp,
        style = AlbumArtStyle.Song,
        containerColor = colors.primary.copy(alpha = 0.2f),
        contentColor = colors.onPrimaryContainer
      )

      Spacer(modifier = Modifier.width(12.dp))

      Column(
        modifier = Modifier.weight(1f),
        verticalArrangement = Arrangement.Center
      ) {
        Text(
          text = title,
          style = MaterialTheme.typography.titleSmall.copy(
            fontSize = 15.sp,
            fontWeight = FontWeight.SemiBold,
            letterSpacing = (-0.2).sp
          ),
          color = colors.onPrimaryContainer,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis
        )
        Text(
          text = artist,
          style = MaterialTheme.typography.bodySmall.copy(
            fontSize = 13.sp
          ),
          color = colors.onPrimaryContainer.copy(alpha = 0.7f),
          maxLines = 1,
          overflow = TextOverflow.Ellipsis
        )
      }

      Spacer(modifier = Modifier.width(8.dp))

      Box(
        modifier = Modifier
          .size(36.dp)
          .clip(CircleShape)
          .background(colors.primary.copy(alpha = 0.2f))
          .clickable(
            interactionSource = remember { MutableInteractionSource() },
            indication = null,
            onClick = onPrevious
          ),
        contentAlignment = Alignment.Center
      ) {
        Icon(
          imageVector = Icons.Filled.SkipPrevious,
          contentDescription = "Previous",
          tint = colors.primary,
          modifier = Modifier.size(22.dp)
        )
      }

      Spacer(modifier = Modifier.width(8.dp))

      Box(
        modifier = Modifier
          .size(36.dp)
          .clip(CircleShape)
          .background(colors.primary)
          .clickable(
            interactionSource = remember { MutableInteractionSource() },
            indication = null,
            onClick = onPlayPause
          ),
        contentAlignment = Alignment.Center
      ) {
        Icon(
          imageVector = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
          contentDescription = if (isPlaying) "Pause" else "Play",
          tint = colors.onPrimary,
          modifier = Modifier.size(20.dp)
        )
      }

      Spacer(modifier = Modifier.width(8.dp))

      Box(
        modifier = Modifier
          .size(36.dp)
          .clip(CircleShape)
          .background(colors.primary.copy(alpha = 0.2f))
          .clickable(
            interactionSource = remember { MutableInteractionSource() },
            indication = null,
            onClick = onNext
          ),
        contentAlignment = Alignment.Center
      ) {
        Icon(
          imageVector = Icons.Filled.SkipNext,
          contentDescription = "Next",
          tint = colors.primary,
          modifier = Modifier.size(22.dp)
        )
      }
    }
  }
}
