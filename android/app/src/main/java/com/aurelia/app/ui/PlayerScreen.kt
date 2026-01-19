package com.aurelia.app.ui

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.QueueMusic
import androidx.compose.material.icons.filled.Album
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.Repeat
import androidx.compose.material.icons.filled.RepeatOne
import androidx.compose.material.icons.filled.Shuffle
import androidx.compose.material.icons.filled.SkipNext
import androidx.compose.material.icons.filled.SkipPrevious
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import coil.compose.SubcomposeAsyncImage
import com.aurelia.app.data.model.Lyrics
import com.aurelia.app.player.PlayerController
import com.aurelia.app.player.QueueItem
import com.aurelia.app.player.RepeatMode
import com.aurelia.app.ui.components.AlbumArt
import com.aurelia.app.ui.components.AnimatedPlayPauseIcon
import com.aurelia.app.ui.components.WavyMusicSlider
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

private enum class ControlButton { NONE, PREVIOUS, PLAY_PAUSE, NEXT }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlayerScreen(
    playerController: PlayerController,
    sessionStore: com.aurelia.app.storage.SessionStore,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    isEmbedded: Boolean = false,
) {
    val viewModel: PlayerViewModel =
        viewModel(
            factory = PlayerViewModelFactory(playerController, sessionStore),
        )
    val state by viewModel.state.collectAsState()

    var showQueueSheet by remember { mutableStateOf(false) }
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    val colors = MaterialTheme.colorScheme

    Column(
        modifier =
            modifier
                .fillMaxSize()
                .background(colors.primaryContainer)
                .statusBarsPadding(),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            FilledIconButton(
                onClick = onBack,
                colors =
                    IconButtonDefaults.filledIconButtonColors(
                        containerColor = colors.onPrimary,
                        contentColor = colors.primary,
                    ),
                modifier = Modifier.size(42.dp),
            ) {
                Icon(
                    imageVector = Icons.Filled.KeyboardArrowDown,
                    contentDescription = "Close player",
                )
            }

            Spacer(modifier = Modifier.weight(1f))

            FilledIconButton(
                onClick = { viewModel.toggleLyrics() },
                colors =
                    IconButtonDefaults.filledIconButtonColors(
                        containerColor = if (state.showLyrics) colors.primary else colors.onPrimary.copy(alpha = 0.8f),
                        contentColor = if (state.showLyrics) colors.onPrimary else colors.primary,
                    ),
                modifier = Modifier.size(42.dp),
            ) {
                Icon(
                    imageVector = Icons.Filled.Mic,
                    contentDescription = "Show lyrics",
                )
            }

            Spacer(modifier = Modifier.size(8.dp))

            IconButton(
                onClick = { showQueueSheet = true },
                colors =
                    IconButtonDefaults.iconButtonColors(
                        containerColor = colors.onPrimary.copy(alpha = 0.8f),
                        contentColor = colors.onPrimaryContainer,
                    ),
                modifier = Modifier.size(42.dp),
            ) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.QueueMusic,
                    contentDescription = "Show queue",
                )
            }
        }

        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(PaddingValues(horizontal = 24.dp, vertical = 16.dp))
                    .navigationBarsPadding(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.SpaceEvenly,
        ) {
            if (state.showLyrics) {
                LyricsView(
                    lyrics = state.lyrics,
                    currentPosition = state.positionMs,
                    onLineClick = { timeMs -> viewModel.seekTo(timeMs.toLong()) },
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = 16.dp)
                            .aspectRatio(1f),
                )
            } else {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = 16.dp)
                            .aspectRatio(1f),
                ) {
                    Surface(
                        modifier =
                            Modifier
                                .fillMaxSize()
                                .clip(RoundedCornerShape(32.dp)),
                        color = colors.surfaceVariant,
                        tonalElevation = 8.dp,
                    ) {
                        if (state.albumArtUrl.isNullOrBlank()) {
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
                                model = state.albumArtUrl,
                                contentDescription = "Album art",
                                modifier = Modifier.fillMaxSize(),
                                contentScale = ContentScale.Crop,
                                loading = {
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
                                },
                                error = {
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
                                },
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.Start,
            ) {
                Text(
                    text = state.title.ifBlank { "Nothing playing" },
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold,
                    color = colors.onPrimaryContainer,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = state.artist.ifBlank { "Unknown artist" },
                    style = MaterialTheme.typography.bodyLarge,
                    color = colors.onPrimaryContainer.copy(alpha = 0.7f),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            if (!isEmbedded) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 8.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Text(
                            text = formatTime(state.positionMs),
                            style = MaterialTheme.typography.labelMedium,
                            fontWeight = FontWeight.Medium,
                            color = colors.onPrimaryContainer.copy(alpha = 0.7f),
                        )
                        val durationMs = state.durationMs
                        val progressFraction =
                            if (durationMs > 0L) {
                                (state.positionMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f)
                            } else {
                                0f
                            }

                        WavyMusicSlider(
                            value = progressFraction,
                            onValueChange = { newFraction: Float ->
                                viewModel.seekTo((newFraction * durationMs).toLong())
                            },
                            modifier = Modifier.weight(1f),
                            trackHeight = 6.dp,
                            thumbRadius = 8.dp,
                            activeTrackColor = colors.primary,
                            inactiveTrackColor = colors.primary.copy(alpha = 0.2f),
                            thumbColor = colors.primary,
                            waveLength = 30.dp,
                            isPlaying = state.isPlaying,
                            isWaveEligible = true,
                        )
                        Text(
                            text = formatTime(state.durationMs),
                            style = MaterialTheme.typography.labelMedium,
                            fontWeight = FontWeight.Medium,
                            color = colors.onPrimaryContainer.copy(alpha = 0.7f),
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            AnimatedPlaybackControls(
                isPlaying = state.isPlaying,
                hasPrevious = state.hasPrevious,
                hasNext = state.hasNext,
                onPrevious = { viewModel.skipPrevious() },
                onPlayPause = { viewModel.togglePlayPause() },
                onNext = { viewModel.skipNext() },
                height = 80.dp,
                colors = colors,
            )

            Spacer(modifier = Modifier.height(16.dp))

            SecondaryControls(
                isShuffled = state.isShuffled,
                repeatMode = state.repeatMode,
                isFavorite = state.isFavorite,
                isFavoriteLoading = state.isFavoriteLoading,
                onShuffleClick = { viewModel.toggleShuffle() },
                onRepeatClick = { viewModel.cycleRepeatMode() },
                onFavoriteClick = { viewModel.toggleFavorite() },
                colors = colors,
            )

            Spacer(modifier = Modifier.height(16.dp))
        }
    }

    if (showQueueSheet) {
        ModalBottomSheet(
            onDismissRequest = { showQueueSheet = false },
            sheetState = sheetState,
            containerColor = colors.surface,
            dragHandle = {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Box(
                        modifier =
                            Modifier
                                .size(width = 32.dp, height = 4.dp)
                                .clip(RoundedCornerShape(2.dp))
                                .background(colors.onSurfaceVariant.copy(alpha = 0.4f)),
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    Text(
                        text = "Queue",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.onSurface,
                    )
                    if (state.queue.isNotEmpty()) {
                        Text(
                            text = "${state.queue.size} tracks",
                            style = MaterialTheme.typography.bodySmall,
                            color = colors.onSurfaceVariant,
                        )
                    }
                    Spacer(modifier = Modifier.height(8.dp))
                }
            },
        ) {
            QueueContent(
                queue = state.queue,
                currentIndex = state.currentQueueIndex,
                onItemClick = { index ->
                    viewModel.playQueueItem(index)
                    scope.launch {
                        sheetState.hide()
                        showQueueSheet = false
                    }
                },
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .navigationBarsPadding(),
            )
        }
    }
}

@Composable
private fun AnimatedPlaybackControls(
    isPlaying: Boolean,
    hasPrevious: Boolean,
    hasNext: Boolean,
    onPrevious: () -> Unit,
    onPlayPause: () -> Unit,
    onNext: () -> Unit,
    height: Dp,
    colors: androidx.compose.material3.ColorScheme,
    modifier: Modifier = Modifier,
) {
    var lastClicked by remember { mutableStateOf<ControlButton?>(null) }
    val baseWeight = 1f
    val expandedWeight = 1.2f
    val compressedWeight = 0.7f

    LaunchedEffect(lastClicked) {
        if (lastClicked != null) {
            delay(250L)
            lastClicked = null
        }
    }

    fun weightFor(button: ControlButton): Float =
        when (lastClicked) {
            button -> expandedWeight
            null -> baseWeight
            else -> compressedWeight
        }

    val animationSpec = tween<Float>(durationMillis = 240, easing = FastOutSlowInEasing)

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .height(height),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        val prevWeight by animateFloatAsState(
            targetValue = weightFor(ControlButton.PREVIOUS),
            animationSpec = animationSpec,
            label = "prevWeight",
        )
        val prevBgAlpha = if (hasPrevious) 0.15f else 0.08f
        val prevIconAlpha = if (hasPrevious) 1f else 0.4f
        Box(
            modifier =
                Modifier
                    .weight(prevWeight)
                    .fillMaxHeight()
                    .clip(CircleShape)
                    .background(colors.primary.copy(alpha = prevBgAlpha))
                    .then(
                        if (hasPrevious) {
                            Modifier.clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null,
                            ) {
                                lastClicked = ControlButton.PREVIOUS
                                onPrevious()
                            }
                        } else {
                            Modifier
                        },
                    ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = Icons.Filled.SkipPrevious,
                contentDescription = "Previous",
                tint = colors.primary.copy(alpha = prevIconAlpha),
                modifier = Modifier.size(32.dp),
            )
        }

        val playWeight by animateFloatAsState(
            targetValue = weightFor(ControlButton.PLAY_PAUSE),
            animationSpec = animationSpec,
            label = "playWeight",
        )
        val playCorner by animateDpAsState(
            targetValue = if (isPlaying) 26.dp else 50.dp,
            animationSpec =
                spring(
                    dampingRatio = Spring.DampingRatioNoBouncy,
                    stiffness = Spring.StiffnessMedium,
                ),
            label = "playCorner",
        )
        Box(
            modifier =
                Modifier
                    .weight(playWeight)
                    .fillMaxHeight()
                    .clip(RoundedCornerShape(playCorner))
                    .background(colors.primary)
                    .clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null,
                    ) {
                        lastClicked = ControlButton.PLAY_PAUSE
                        onPlayPause()
                    },
            contentAlignment = Alignment.Center,
        ) {
            AnimatedPlayPauseIcon(
                isPlaying = isPlaying,
                tint = colors.onPrimary,
                modifier = Modifier.size(36.dp),
            )
        }

        val nextWeight by animateFloatAsState(
            targetValue = weightFor(ControlButton.NEXT),
            animationSpec = animationSpec,
            label = "nextWeight",
        )
        val nextBgAlpha = if (hasNext) 0.15f else 0.08f
        val nextIconAlpha = if (hasNext) 1f else 0.4f
        Box(
            modifier =
                Modifier
                    .weight(nextWeight)
                    .fillMaxHeight()
                    .clip(CircleShape)
                    .background(colors.primary.copy(alpha = nextBgAlpha))
                    .then(
                        if (hasNext) {
                            Modifier.clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null,
                            ) {
                                lastClicked = ControlButton.NEXT
                                onNext()
                            }
                        } else {
                            Modifier
                        },
                    ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = Icons.Filled.SkipNext,
                contentDescription = "Next",
                tint = colors.primary.copy(alpha = nextIconAlpha),
                modifier = Modifier.size(32.dp),
            )
        }
    }
}

@Composable
private fun SecondaryControls(
    isShuffled: Boolean,
    repeatMode: RepeatMode,
    isFavorite: Boolean,
    isFavoriteLoading: Boolean,
    onShuffleClick: () -> Unit,
    onRepeatClick: () -> Unit,
    onFavoriteClick: () -> Unit,
    colors: androidx.compose.material3.ColorScheme,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // Shuffle button
        IconButton(
            onClick = onShuffleClick,
            modifier = Modifier.size(48.dp),
        ) {
            Icon(
                imageVector = Icons.Filled.Shuffle,
                contentDescription = if (isShuffled) "Shuffle on" else "Shuffle off",
                tint = if (isShuffled) colors.primary else colors.onPrimaryContainer.copy(alpha = 0.5f),
                modifier = Modifier.size(24.dp),
            )
        }

        // Repeat button
        IconButton(
            onClick = onRepeatClick,
            modifier = Modifier.size(48.dp),
        ) {
            val (icon, contentDesc, tint) =
                when (repeatMode) {
                    RepeatMode.ONE -> {
                        Triple(
                            Icons.Filled.RepeatOne,
                            "Repeat one",
                            colors.primary,
                        )
                    }

                    RepeatMode.ALL -> {
                        Triple(
                            Icons.Filled.Repeat,
                            "Repeat all",
                            colors.primary,
                        )
                    }

                    RepeatMode.NONE -> {
                        Triple(
                            Icons.Filled.Repeat,
                            "Repeat off",
                            colors.onPrimaryContainer.copy(alpha = 0.5f),
                        )
                    }
                }
            Icon(
                imageVector = icon,
                contentDescription = contentDesc,
                tint = tint,
                modifier = Modifier.size(24.dp),
            )
        }

        // Favorite button
        IconButton(
            onClick = onFavoriteClick,
            enabled = !isFavoriteLoading,
            modifier = Modifier.size(48.dp),
        ) {
            val iconAlpha = if (isFavoriteLoading) 0.3f else 1f
            Icon(
                imageVector = if (isFavorite) Icons.Filled.Favorite else Icons.Filled.FavoriteBorder,
                contentDescription = if (isFavorite) "Remove from favorites" else "Add to favorites",
                tint = if (isFavorite) colors.secondary else colors.onPrimaryContainer.copy(alpha = 0.5f * iconAlpha),
                modifier =
                    Modifier
                        .size(24.dp)
                        .alpha(iconAlpha),
            )
        }
    }
}

@Composable
private fun formatTime(timeMs: Long): String {
    val totalSeconds = timeMs.coerceAtLeast(0L) / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "%d:%02d".format(minutes, seconds)
}

@Composable
private fun LyricsView(
    lyrics: Lyrics?,
    currentPosition: Long,
    onLineClick: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val syncedLines = lyrics?.synced
    val plainLines = lyrics?.plain

    val currentLineIndex =
        remember(currentPosition, syncedLines) {
            if (syncedLines.isNullOrEmpty()) {
                -1
            } else {
                syncedLines
                    .withIndex()
                    .lastOrNull { (index, line) ->
                        val nextTime = syncedLines.getOrNull(index + 1)?.time?.toLong() ?: Long.MAX_VALUE
                        currentPosition in line.time.toLong()..<nextTime
                    }?.index ?: -1
            }
        }

    BoxWithConstraints(modifier = modifier) {
        val containerHeight = maxHeight
        val density = LocalDensity.current
        val viewportHeightPx = with(density) { containerHeight.toPx() }

        if (!syncedLines.isNullOrEmpty()) {
            val listState = rememberLazyListState()
            val lineHeights = remember { mutableStateMapOf<Int, Int>() }
            val density = LocalDensity.current

            // Auto-scroll when the current line changes
            LaunchedEffect(currentLineIndex) {
                if (currentLineIndex >= 0) {
                    snapshotFlow { lineHeights[currentLineIndex] }
                        .collectLatest { height ->
                            val offset = if (height != null) height / 2 else with(density) { 30.dp.roundToPx() }
                            listState.animateScrollToItem(currentLineIndex, scrollOffset = offset)
                        }
                }
            }

            LazyColumn(
                state = listState,
                modifier = Modifier.fillMaxSize(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(0.dp),
                contentPadding = PaddingValues(vertical = containerHeight / 2),
            ) {
                itemsIndexed(syncedLines) { index, line ->
                    val isCurrentLine = index == currentLineIndex

                    val blurRadius by animateDpAsState(
                        targetValue = if (isCurrentLine) 0.dp else 4.dp,
                        animationSpec = tween(durationMillis = 300),
                        label = "blur",
                    )

                    val textColor by animateColorAsState(
                        targetValue = if (isCurrentLine) colors.primary else colors.onPrimaryContainer,
                        animationSpec = tween(durationMillis = 300),
                        label = "color",
                    )

                    Text(
                        text = line.line,
                        style = MaterialTheme.typography.headlineMedium.copy(fontSize = 28.sp),
                        fontWeight = if (isCurrentLine) FontWeight.Bold else FontWeight.Normal,
                        color = textColor,
                        textAlign = TextAlign.Center,
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .onSizeChanged { lineHeights[index] = it.height }
                                .blur(blurRadius)
                                .clip(RoundedCornerShape(12.dp))
                                .clickable(
                                    interactionSource = remember { MutableInteractionSource() },
                                    indication = null,
                                ) { onLineClick(line.time) }
                                .padding(vertical = 8.dp, horizontal = 4.dp),
                    )
                }
            }

            val fadeColor = colors.primaryContainer

            // Top Fade
            Box(
                modifier =
                    Modifier
                        .align(Alignment.TopCenter)
                        .fillMaxWidth()
                        .height(containerHeight / 6)
                        .background(
                            Brush.verticalGradient(
                                colors = listOf(fadeColor, Color.Transparent),
                            ),
                        ),
            )

            // Bottom Fade
            Box(
                modifier =
                    Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(containerHeight / 6)
                        .background(
                            Brush.verticalGradient(
                                colors = listOf(Color.Transparent, fadeColor),
                            ),
                        ),
            )
        } else if (!plainLines.isNullOrEmpty()) {
            LazyColumn(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .padding(horizontal = 16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp),
                contentPadding = PaddingValues(vertical = 200.dp),
            ) {
                itemsIndexed(plainLines) { _, line ->
                    Text(
                        text = line,
                        style = MaterialTheme.typography.headlineSmall,
                        color = colors.onPrimaryContainer.copy(alpha = 0.7f),
                        textAlign = TextAlign.Center,
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(vertical = 8.dp),
                    )
                }
            }
        } else {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "No lyrics available",
                    style = MaterialTheme.typography.bodyLarge,
                    color = colors.onPrimaryContainer.copy(alpha = 0.5f),
                )
            }
        }
    }
}

@Composable
private fun QueueContent(
    queue: List<QueueItem>,
    currentIndex: Int,
    onItemClick: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val listState = rememberLazyListState()

    LaunchedEffect(currentIndex) {
        if (currentIndex >= 0 && queue.isNotEmpty()) {
            listState.animateScrollToItem(currentIndex.coerceAtMost(queue.lastIndex))
        }
    }

    if (queue.isEmpty()) {
        Box(
            modifier =
                modifier
                    .height(200.dp)
                    .padding(24.dp),
            contentAlignment = Alignment.Center,
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.QueueMusic,
                    contentDescription = null,
                    modifier = Modifier.size(48.dp),
                    tint = colors.onSurfaceVariant.copy(alpha = 0.5f),
                )
                Spacer(modifier = Modifier.height(12.dp))
                Text(
                    text = "Queue is empty",
                    style = MaterialTheme.typography.bodyLarge,
                    color = colors.onSurfaceVariant,
                )
                Text(
                    text = "Play some songs to build your queue",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant.copy(alpha = 0.7f),
                )
            }
        }
    } else {
        LazyColumn(
            modifier = modifier,
            state = listState,
            contentPadding = PaddingValues(bottom = 16.dp),
        ) {
            itemsIndexed(queue, key = { idx, item -> "${item.id}_$idx" }) { index, item ->
                QueueItemRow(
                    item = item,
                    isPlaying = index == currentIndex,
                    position = index + 1,
                    onClick = { onItemClick(index) },
                )
            }
        }
    }
}

@Composable
private fun QueueItemRow(
    item: QueueItem,
    isPlaying: Boolean,
    position: Int,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val backgroundColor =
        if (isPlaying) colors.primaryContainer.copy(alpha = 0.3f) else colors.surface

    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(backgroundColor)
                .clickable(onClick = onClick)
                .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(
            modifier = Modifier.size(24.dp),
            contentAlignment = Alignment.Center,
        ) {
            if (isPlaying) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(2.dp),
                    verticalAlignment = Alignment.Bottom,
                    modifier = Modifier.size(16.dp),
                ) {
                    repeat(3) { idx ->
                        val height by animateFloatAsState(
                            targetValue = if (isPlaying) (0.4f + (idx * 0.2f)) else 0.3f,
                            animationSpec = tween(300),
                            label = "bar$idx",
                        )
                        Box(
                            modifier =
                                Modifier
                                    .weight(1f)
                                    .fillMaxHeight(height)
                                    .background(colors.primary, RoundedCornerShape(1.dp)),
                        )
                    }
                }
            } else {
                Text(
                    text = position.toString(),
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                )
            }
        }

        AlbumArt(
            imageUrl = item.albumArtUrl,
            modifier = Modifier.size(48.dp),
            cornerRadius = 6.dp,
        )

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = item.title,
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = if (isPlaying) FontWeight.SemiBold else FontWeight.Normal,
                color = if (isPlaying) colors.primary else colors.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = item.artist,
                style = MaterialTheme.typography.bodySmall,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
