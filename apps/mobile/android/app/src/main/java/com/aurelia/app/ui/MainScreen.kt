package com.aurelia.app.ui

import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.material.icons.filled.Person
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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.lerp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.util.lerp
import androidx.compose.ui.zIndex
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavDestination
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import com.aurelia.app.player.PlayerController
import com.aurelia.app.storage.SessionStore
import com.aurelia.app.audio.AudioManager
import com.aurelia.app.audio.VisualizerStyle
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AlbumArtStyle
import com.aurelia.app.ui.components.AnimatedPlayPauseIcon
import com.aurelia.app.ui.components.AudioVisualizer
import com.aurelia.app.ui.navigation.Screen
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private val MiniPlayerHeight = 64.dp
private val NavBarContentHeight = 90.dp

data class NavItem(
  val screen: Screen,
  val label: String,
  val selectedIcon: ImageVector,
  val unselectedIcon: ImageVector,
)

private val navItems =
  listOf(
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
  onLogout: () -> Unit,
) {
  val navController = rememberNavController()

  // HomeViewModel hoisted here to survive tab switches
  val homeViewModel: HomeViewModel = viewModel(
    factory = remember { viewModelFactory { HomeViewModel(sessionStore, playerController) } },
  )

  // SettingsViewModel hoisted here to survive tab switches
  val settingsViewModel: SettingsViewModel = viewModel(
    factory = remember { viewModelFactory { SettingsViewModel(sessionStore) } },
  )

  // PlaylistViewModel hoisted here to survive tab switches
  val playlistViewModel: PlaylistViewModel = viewModel(
    factory = remember { viewModelFactory { PlaylistViewModel(sessionStore, playerController) } },
  )

  val libraryViewModel: LibraryViewModel = viewModel(
    factory = remember { viewModelFactory { LibraryViewModel(sessionStore, playerController) } },
  )
  val libraryState by libraryViewModel.state.collectAsState()
  val scope = rememberCoroutineScope()

  // Load home data once when ViewModel is created (not on every tab switch)
  LaunchedEffect(homeViewModel) {
    homeViewModel.loadHomeData()
  }

  // --- NAVIGATION HELPERS ---
  fun navigateToTab(screen: Screen) {
    navController.navigate(screen) {
      // Pop up to the start destination of the graph to avoid building up a large stack of tabs
      popUpTo(navController.graph.findStartDestination().id) {
        saveState = true
      }
      // Avoid multiple copies of the same destination
      launchSingleTop = true
      // Restore state when reselecting a previously selected item
      restoreState = true
    }
  }

  @Suppress("UnusedBoxWithConstraintsScope")
  BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
    val density = LocalDensity.current
    val screenHeightPx = constraints.maxHeight.toFloat()
    val miniPlayerTopMargin = 4.dp
    val expandFractionThreshold = 0.35f
    val bottomInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
    val navBarBottomPadding = bottomInset + 12.dp
    val navBarHeightPx = with(density) { (NavBarContentHeight + navBarBottomPadding).toPx() }
    val miniPlayerHeightPx = with(density) { MiniPlayerHeight.toPx() }
    val miniPlayerTopMarginPx = with(density) { miniPlayerTopMargin.toPx() }
    val collapsedSheetY =
      (screenHeightPx - miniPlayerHeightPx - navBarHeightPx - miniPlayerTopMarginPx)
        .coerceAtLeast(0f)
    val playerDragOffset = remember { Animatable(collapsedSheetY) }
    val dragProgress =
      if (collapsedSheetY > 0f) {
        ((collapsedSheetY - playerDragOffset.value) / collapsedSheetY).coerceIn(0f, 1f)
      } else {
        1f
      }
    val playerScale = 1f
    val miniPlayerAlpha = (1f - dragProgress * 2f).coerceIn(0f, 1f)
    val fullPlayerAlpha = ((dragProgress - 0.15f).coerceIn(0f, 0.85f) / 0.85f)
    val sheetTopCornerRadius = lerp(32.dp, 0.dp, dragProgress)
    val sheetBottomCornerRadius = lerp(12.dp, 0.dp, dragProgress)
    val sheetHorizontalPadding = lerp(12.dp, 0.dp, dragProgress)
    val sheetHeightPx = miniPlayerHeightPx + (screenHeightPx - miniPlayerHeightPx) * dragProgress
    val sheetHeightDp = with(density) { sheetHeightPx.toDp() }

    // Keep collapsed position in sync when layout changes (e.g., initial measurement)
    LaunchedEffect(collapsedSheetY) {
      // Only snap if player is collapsed - don't interrupt drag/expand animations
      if (dragProgress < 0.1f) {
        playerDragOffset.snapTo(collapsedSheetY)
      }
    }

    fun openPlayerAnimated(initialVelocity: Float = 0f) {
      scope.launch {
        playerDragOffset.animateTo(
          targetValue = 0f,
          initialVelocity = initialVelocity,
          animationSpec =
            spring(
              dampingRatio = Spring.DampingRatioNoBouncy,
              stiffness = Spring.StiffnessMediumLow,
            ),
        )
      }
    }

    fun closePlayer(initialVelocity: Float = 0f) {
      scope.launch {
        playerDragOffset.animateTo(
          targetValue = collapsedSheetY,
          initialVelocity = initialVelocity,
          animationSpec =
            spring(
              dampingRatio = Spring.DampingRatioNoBouncy,
              stiffness = Spring.StiffnessMediumLow,
            ),
        )
      }
    }

    fun animatePlayerOffset(
      target: Float,
      initialVelocity: Float = 0f,
    ) {
      scope.launch {
        playerDragOffset.animateTo(
          targetValue = target,
          initialVelocity = initialVelocity,
          animationSpec =
            spring(
              dampingRatio = Spring.DampingRatioNoBouncy,
              stiffness = Spring.StiffnessMediumLow,
            ),
        )
      }
    }

    fun onPlayerDrag(delta: Float) {
      scope.launch {
        playerDragOffset.snapTo((playerDragOffset.value + delta).coerceIn(0f, collapsedSheetY))
      }
    }

    fun onPlayerDragEnd(velocity: Float) {
      val shouldOpen =
        when {
          velocity < -80f -> true
          velocity > 120f -> false
          else -> dragProgress > expandFractionThreshold
        }
      if (shouldOpen) {
        openPlayerAnimated(initialVelocity = velocity)
      } else {
        animatePlayerOffset(collapsedSheetY, initialVelocity = velocity)
      }
    }

    // 1. SCAFFOLD / CONTENT AREA
    Box(
      modifier =
        Modifier
          .fillMaxSize()
          .background(MaterialTheme.colorScheme.background),
    ) {

      NavHost(
        navController = navController,
        startDestination = Screen.Home,
        // SETTINGS-STYLE ANIMATIONS
        enterTransition = { slideIntoContainer(AnimatedContentTransitionScope.SlideDirection.Start, tween(400)) },
        exitTransition = { slideOutOfContainer(AnimatedContentTransitionScope.SlideDirection.Start, tween(400)) },
        popEnterTransition = { slideIntoContainer(AnimatedContentTransitionScope.SlideDirection.End, tween(400)) },
        popExitTransition = { slideOutOfContainer(AnimatedContentTransitionScope.SlideDirection.End, tween(400)) }
      ) {
        // --- TABS ---
        composable<Screen.Home> {
          HomeScreen(
            viewModel = homeViewModel,
            sessionStore = sessionStore,
            playerController = playerController,
            playlistViewModel = playlistViewModel,
            onOpenPlayer = { openPlayerAnimated() },
            onNavigateToAlbum = { navController.navigate(it) },
            onNavigateToArtist = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.Songs> {
          LibraryScreen(
            sessionStore = sessionStore,
            playerController = playerController,
            playlistViewModel = playlistViewModel,
            onOpenPlayer = { openPlayerAnimated() },
            onNavigateToAlbum = { navController.navigate(it) },
            onNavigateToArtist = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.Albums> {
          AlbumsScreen(
            sessionStore = sessionStore,
            playerController = playerController,
            onNavigateToAlbum = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.Artists> {
          ArtistsScreen(
            sessionStore = sessionStore,
            playerController = playerController,
            onNavigateToArtist = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.Playlists> {
          PlaylistsScreen(
            viewModel = playlistViewModel,
            onOpenPlayer = { openPlayerAnimated() },
            onNavigateToPlaylist = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.Search> {
          SearchScreen(
            sessionStore = sessionStore,
            playerController = playerController,
            onOpenPlayer = { openPlayerAnimated() },
            hasPlayerBar = libraryState.nowPlaying != null,
            playlistViewModel = playlistViewModel,
          )
        }

        composable<Screen.Settings> {
          SettingsScreen(
            sessionStore = sessionStore,
            settingsViewModel = settingsViewModel,
            onLogout = onLogout,
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        // --- DETAILS SCREENS ---
        composable<Screen.AlbumDetail> { backStackEntry ->
          val args = backStackEntry.toRoute<Screen.AlbumDetail>()
          AlbumDetailScreen(
            albumId = args.albumId,
            albumName = args.albumName,
            sessionStore = sessionStore,
            playerController = playerController,
            playlistViewModel = playlistViewModel,
            onBack = { navController.popBackStack() },
            onOpenPlayer = { openPlayerAnimated() },
            onNavigateToArtist = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.ArtistDetail> { backStackEntry ->
          val args = backStackEntry.toRoute<Screen.ArtistDetail>()
          ArtistDetailScreen(
            artistId = args.artistId,
            artistName = args.artistName,
            sessionStore = sessionStore,
            playerController = playerController,
            playlistViewModel = playlistViewModel,
            onBack = { navController.popBackStack() },
            onOpenPlayer = { openPlayerAnimated() },
            onNavigateToAlbum = { navController.navigate(it) },
            hasPlayerBar = libraryState.nowPlaying != null,
          )
        }

        composable<Screen.PlaylistDetail> { backStackEntry ->
          val args = backStackEntry.toRoute<Screen.PlaylistDetail>()
          PlaylistDetailScreen(
            playlistId = args.playlistId,
            playlistName = args.playlistName,
            viewModel = playlistViewModel,
            onBack = { navController.popBackStack() },
            onOpenPlayer = { openPlayerAnimated() },
          )
        }
      }

      // 2. BOTTOM NAV BAR OVERLAY
      val navBackStackEntry by navController.currentBackStackEntryAsState()
      val currentDestination = navBackStackEntry?.destination

      // Show bottom bar on all screens
      Column(
        modifier = Modifier
          .align(Alignment.BottomCenter)
          .fillMaxWidth()
      ) {
        BottomNavBar(
          items = navItems,
          currentDestination = currentDestination,
          onNavigate = { navigateToTab(it) },
          onSettingsClick = { navigateToTab(Screen.Settings) },
          hasPlayerBar = libraryState.nowPlaying != null,
        )
      }
    }

    // 3. MINIPLAYER OVERLAY
    if (libraryState.nowPlaying != null) {
      // Handle predictive back gesture when player is expanded
      PredictiveBackHandler(enabled = dragProgress > 0.5f) { progress ->
        try {
          progress.collect { backEvent ->
            // Map back gesture progress (0-1) to player offset
            val targetOffset = collapsedSheetY * backEvent.progress
            playerDragOffset.snapTo(targetOffset)
          }
          // Gesture completed - close player
          closePlayer()
        } catch (_: kotlin.coroutines.cancellation.CancellationException) {
          // Gesture cancelled - snap back to open
          openPlayerAnimated()
        }
      }

      val playerSurfaceColor = MaterialTheme.colorScheme.primaryContainer
      Box(
        modifier =
          Modifier
            .offset { IntOffset(0, playerDragOffset.value.roundToInt()) }
            .fillMaxWidth()
            .padding(horizontal = sheetHorizontalPadding)
            .height(sheetHeightDp)
            .graphicsLayer {
              scaleX = playerScale
              scaleY = playerScale
              clip = true
              shape =
                RoundedCornerShape(
                  topStart = sheetTopCornerRadius,
                  topEnd = sheetTopCornerRadius,
                  bottomStart = sheetBottomCornerRadius,
                  bottomEnd = sheetBottomCornerRadius,
                )
              transformOrigin = TransformOrigin(0.5f, 1f)
            }
            .background(
              color = playerSurfaceColor,
              shape =
                RoundedCornerShape(
                  topStart = sheetTopCornerRadius,
                  topEnd = sheetTopCornerRadius,
                  bottomStart = sheetBottomCornerRadius,
                  bottomEnd = sheetBottomCornerRadius,
                ),
            )
            .zIndex(1f),
      ) {
        libraryState.nowPlaying?.let { nowPlaying ->
           MiniPlayerBar(
             title = nowPlaying.title,
             artist = nowPlaying.artist,
             albumArtUrl = nowPlaying.albumArtUrl,
             isPlaying = nowPlaying.isPlaying,
             isBuffering = nowPlaying.isBuffering,
             hasPrevious = nowPlaying.hasPrevious,
             hasNext = nowPlaying.hasNext,
             onPrevious = { libraryViewModel.skipPrevious() },
             onPlayPause = { libraryViewModel.togglePlayPause() },
             onNext = { libraryViewModel.skipNext() },
             onClick = { openPlayerAnimated() },
             onDrag = { delta -> onPlayerDrag(delta) },
             onDragEnd = { velocity -> onPlayerDragEnd(velocity) },
             modifier =
               Modifier
                 .align(Alignment.BottomCenter)
                 .graphicsLayer { alpha = miniPlayerAlpha }
                 .zIndex(if (dragProgress < 0.5f) 1f else 0f),
             albumId = nowPlaying.albumId,
             artistId = nowPlaying.artistId,
             albumName = nowPlaying.albumName,
             onNavigateToAlbum = { navController.navigate(it) },
             onNavigateToArtist = { navController.navigate(it) },
             sessionStore = sessionStore,
           )
         }

        Box(
          modifier =
            Modifier
              .fillMaxSize()
              .pointerInput(Unit) {
                var totalDrag = 0f
                detectVerticalDragGestures(
                  onVerticalDrag = { _, dragAmount ->
                    totalDrag += dragAmount
                    onPlayerDrag(dragAmount)
                  },
                  onDragEnd = {
                    val velocity = if (totalDrag != 0f) totalDrag * 12f else 0f
                    onPlayerDragEnd(velocity)
                    totalDrag = 0f
                  },
                )
              }
              .graphicsLayer {
                alpha = fullPlayerAlpha
                scaleX = lerp(0.92f, 1f, dragProgress)
                scaleY = lerp(0.92f, 1f, dragProgress)
                transformOrigin = TransformOrigin(0.5f, 1f)
              }
              .zIndex(if (dragProgress >= 0.5f) 1f else 0f),
        ) {
          PlayerScreen(
            playerController = playerController,
            sessionStore = sessionStore,
            onBack = { closePlayer() },
            modifier = Modifier.fillMaxSize(),
            onNavigateToAlbum = {
              closePlayer()
              navController.navigate(it)
            },
            onNavigateToArtist = {
              closePlayer()
              navController.navigate(it)
            },
          )
        }
      }
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
  currentDestination: NavDestination?,
  onNavigate: (Screen) -> Unit,
  onSettingsClick: () -> Unit,
  hasPlayerBar: Boolean,
  modifier: Modifier = Modifier,
) {
  val colors = MaterialTheme.colorScheme
  val bottomInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
  val bottomPadding = if (hasPlayerBar) 8.dp else 12.dp
  val topCornerRadius = if (hasPlayerBar) 12.dp else 32.dp

  Surface(
    modifier =
      modifier
        .fillMaxWidth()
        .padding(horizontal = 12.dp)
        .padding(bottom = bottomInset + bottomPadding),
    color = colors.surface,
    tonalElevation = 4.dp,
    shadowElevation = 12.dp,
    shape =
      RoundedCornerShape(
        topStart = topCornerRadius,
        topEnd = topCornerRadius,
        bottomStart = 32.dp,
        bottomEnd = 32.dp,
      ),
  ) {
    Column(
      modifier =
        Modifier
          .fillMaxWidth()
          .padding(horizontal = 18.dp, vertical = 12.dp),
    ) {
      Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
      ) {
        items.forEach { item ->
          // Use hierarchy to check if we are in this tab
          val isSelected = currentDestination?.hierarchy?.any { it.hasRoute(item.screen::class) } == true
          BottomNavItem(
            label = item.label,
            icon = if (isSelected) item.selectedIcon else item.unselectedIcon,
            selected = isSelected,
            onClick = { onNavigate(item.screen) },
          )
        }

        val isSettingsSelected = currentDestination?.hierarchy?.any { it.hasRoute<Screen.Settings>() } == true
        BottomNavItem(
          label = "Settings",
          icon = if (isSettingsSelected) Icons.Filled.Settings else Icons.Outlined.Settings,
          selected = isSettingsSelected,
          onClick = onSettingsClick,
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
  onClick: () -> Unit,
) {
  val colors = MaterialTheme.colorScheme
  val iconTint by animateColorAsState(
    targetValue = if (selected) colors.primary else colors.onSurfaceVariant,
    animationSpec = tween(durationMillis = 200),
  )
  val textColor by animateColorAsState(
    targetValue = if (selected) colors.primary else colors.onSurfaceVariant,
    animationSpec = tween(durationMillis = 200),
  )

  Column(
    modifier =
      Modifier
        .weight(1f)
        .clip(RoundedCornerShape(20.dp))
        .clickable(
          interactionSource = remember { MutableInteractionSource() },
          indication = null,
          role = Role.Tab,
          onClick = onClick,
        )
        .padding(vertical = 6.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.Center,
  ) {
    Box(
      modifier =
        Modifier
          .size(36.dp)
          .clip(CircleShape)
          .background(colors.primary.copy(alpha = if (selected) 0.2f else 0f)),
      contentAlignment = Alignment.Center,
    ) {
      Icon(
        imageVector = icon,
        contentDescription = label,
        tint = iconTint,
        modifier = Modifier.size(20.dp),
      )
    }

    Spacer(modifier = Modifier.height(6.dp))

    ProvideTextStyle(MaterialTheme.typography.labelSmall) {
      Text(
        text = label,
        color = textColor,
        maxLines = 1,
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
  isBuffering: Boolean,
  hasPrevious: Boolean,
  hasNext: Boolean,
  onPrevious: () -> Unit,
  onPlayPause: () -> Unit,
  onNext: () -> Unit,
  onClick: () -> Unit,
  onDrag: (Float) -> Unit,
  onDragEnd: (Float) -> Unit,
  modifier: Modifier = Modifier,
  albumId: String? = null,
  artistId: String? = null,
  albumName: String? = null,
  onNavigateToAlbum: ((Screen.AlbumDetail) -> Unit)? = null,
  onNavigateToArtist: ((Screen.ArtistDetail) -> Unit)? = null,
  sessionStore: SessionStore? = null,
) {
  val colors = MaterialTheme.colorScheme

  val visualizerState by AudioManager.visualizerState.collectAsState()
  val visualizerEnabled = sessionStore?.getVisualizerEnabled() ?: false
  val visualizerStyleName = sessionStore?.getVisualizerStyle() ?: "BARS"
  val visualizerStyle = remember(visualizerStyleName) {
    try { VisualizerStyle.valueOf(visualizerStyleName) } catch (_: Exception) { VisualizerStyle.BARS }
  }

  Box(
    modifier =
      modifier
        .fillMaxWidth()
        .pointerInput(Unit) {
          var netDrag = 0f
          detectVerticalDragGestures(
            onVerticalDrag = { _, dragAmount ->
              netDrag += dragAmount
              // Only apply upward drags visually
              if (dragAmount < 0f) {
                onDrag(dragAmount)
              }
            },
            onDragEnd = {
              // Use net displacement for velocity - if user dragged back down, this cancels out
              val velocity = if (netDrag < 0f) netDrag * 12f else 0f
              onDragEnd(velocity)
              netDrag = 0f
            },
          )
        }
        .height(MiniPlayerHeight)
        .padding(horizontal = 18.dp),
  ) {
    // Mini visualizer background
    if (visualizerEnabled && visualizerState.enabled && isPlaying) {
      AudioVisualizer(
        frequencyData = visualizerState.frequencyData,
        timeDomainData = visualizerState.waveform,
        isPlaying = isPlaying,
        style = VisualizerStyle.BARS,
        accentColor = colors.primary,
        modifier = Modifier
          .fillMaxSize()
          .graphicsLayer { alpha = 0.15f },
        boost = 0.5f,
      )
    }

    Row(
      modifier = Modifier.fillMaxSize(),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      // Clickable area for opening player (album art + text)
      Row(
        modifier =
          Modifier
            .weight(1f)
            .fillMaxHeight()
            .clickable(
              interactionSource = remember { MutableInteractionSource() },
              indication = null,
              onClick = onClick,
            ),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        AlbumArt(
          imageUrl = albumArtUrl,
          size = 44.dp,
          cornerRadius = 22.dp,
          style = AlbumArtStyle.Song,
          containerColor = colors.primary.copy(alpha = 0.2f),
          contentColor = colors.onPrimaryContainer,
          modifier =
            Modifier
              .size(44.dp)
              .clickable(
                enabled = !albumId.isNullOrBlank(),
                onClick = {
                  albumId?.let { id ->
                    onNavigateToAlbum?.invoke(Screen.AlbumDetail(id, albumName ?: "Unknown Album"))
                  }
                },
              ),
        )

        Spacer(modifier = Modifier.width(12.dp))

        Column(
          modifier = Modifier.weight(1f),
          verticalArrangement = Arrangement.Center,
        ) {
          Text(
            text = title,
            style =
              MaterialTheme.typography.titleSmall.copy(
                fontSize = 15.sp,
                letterSpacing = (-0.2).sp,
              ),
            fontWeight = FontWeight.Medium,
            color = colors.onPrimaryContainer,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
          Text(
            text = artist,
            style =
              MaterialTheme.typography.bodySmall.copy(
                fontSize = 13.sp,
              ),
            color = colors.onPrimaryContainer.copy(alpha = 0.7f),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier =
              Modifier.clickable(
                enabled = !artistId.isNullOrBlank(),
                onClick = {
                  artistId?.let { id ->
                    onNavigateToArtist?.invoke(Screen.ArtistDetail(id, artist))
                  }
                },
              ),
          )
        }
      }

      Spacer(modifier = Modifier.width(8.dp))

      Box(
        modifier =
          Modifier
            .size(36.dp)
            .clip(CircleShape)
            .background(colors.primary.copy(alpha = if (hasPrevious) 0.2f else 0.08f))
            .then(
              if (hasPrevious) {
                Modifier.clickable(
                  interactionSource = remember { MutableInteractionSource() },
                  indication = null,
                  onClick = onPrevious,
                )
              } else {
                Modifier
              },
            ),
        contentAlignment = Alignment.Center,
      ) {
        Icon(
          imageVector = Icons.Filled.SkipPrevious,
          contentDescription = "Previous",
          tint = colors.primary.copy(alpha = if (hasPrevious) 1f else 0.4f),
          modifier = Modifier.size(22.dp),
        )
      }

      Spacer(modifier = Modifier.width(8.dp))

      Box(
        modifier =
          Modifier
            .size(36.dp)
            .clip(CircleShape)
            .background(colors.primary)
            .then(
              if (!isBuffering) {
                Modifier.clickable(
                  interactionSource = remember { MutableInteractionSource() },
                  indication = null,
                  onClick = onPlayPause,
                )
              } else {
                Modifier
              },
            ),
        contentAlignment = Alignment.Center,
      ) {
        if (isBuffering) {
          CircularProgressIndicator(
            modifier = Modifier.size(18.dp),
            color = colors.onPrimary,
            strokeWidth = 2.dp,
          )
        } else {
          AnimatedPlayPauseIcon(
            isPlaying = isPlaying,
            tint = colors.onPrimary,
            modifier = Modifier.size(20.dp),
          )
        }
      }

      Spacer(modifier = Modifier.width(8.dp))

      Box(
        modifier =
          Modifier
            .size(36.dp)
            .clip(CircleShape)
            .background(colors.primary.copy(alpha = if (hasNext) 0.2f else 0.08f))
            .then(
              if (hasNext) {
                Modifier.clickable(
                  interactionSource = remember { MutableInteractionSource() },
                  indication = null,
                  onClick = onNext,
                )
              } else {
                Modifier
              },
            ),
        contentAlignment = Alignment.Center,
      ) {
        Icon(
          imageVector = Icons.Filled.SkipNext,
          contentDescription = "Next",
          tint = colors.primary.copy(alpha = if (hasNext) 1f else 0.4f),
          modifier = Modifier.size(22.dp),
        )
      }
    }
  }
}
