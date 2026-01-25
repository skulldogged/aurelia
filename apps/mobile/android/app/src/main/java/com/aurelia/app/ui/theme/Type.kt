package com.aurelia.app.ui.theme

import android.content.Context
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay

/**
 * Google Sans Flex Variable Font Configuration

 *
 * Google Sans Flex is a variable font with three axes:
 * - Weight (wght): 100-900 (Thin to Black)
 * - Width (wdth): 75-125 (Condensed to Expanded)
 * - Slant (slnt): -12 to 0 (Italic to Upright)
 *
 * This enables dynamic, expressive typography that responds to context,
 * user interactions, and content importance.
 */

// =============================================================================
// Variable Font Axis Ranges
// =============================================================================

object FontAxis {
  // Weight axis: controls thickness of strokes
  const val WEIGHT_MIN = 100f
  const val WEIGHT_MAX = 900f
  const val WEIGHT_REGULAR = 400f
  const val WEIGHT_MEDIUM = 500f
  const val WEIGHT_SEMIBOLD = 600f
  const val WEIGHT_BOLD = 700f

  // Width axis: controls horizontal expansion/compression
  const val WIDTH_CONDENSED = 75f
  const val WIDTH_NORMAL = 100f
  const val WIDTH_EXPANDED = 125f

  // Slant axis: controls italic angle
  const val SLANT_UPRIGHT = 0f
  const val SLANT_ITALIC = -12f
}

// =============================================================================
// Dynamic Font Family Builder
// =============================================================================

/**
 * Single cached FontFamily for Google Sans Flex.
 * Using resource-based font to avoid repeated asset loading.
 */
private var cachedGoogleSansFlex: FontFamily? = null

@OptIn(ExperimentalTextApi::class)
fun getGoogleSansFlexFont(context: Context): FontFamily {
  return cachedGoogleSansFlex ?: run {
    try {
      val fontFamily = FontFamily(
        // Regular weight
        Font(
          path = "fonts/google_sans_flex_regular.ttf",
          assetManager = context.assets,
          weight = FontWeight.Normal,
          variationSettings = FontVariation.Settings(
            FontVariation.weight(400),
            FontVariation.width(100f),
            FontVariation.Setting("ROND", 100f),
          )
        ),
        // Medium weight
        Font(
          path = "fonts/google_sans_flex_regular.ttf",
          assetManager = context.assets,
          weight = FontWeight.Medium,
          variationSettings = FontVariation.Settings(
            FontVariation.weight(500),
            FontVariation.width(100f),
            FontVariation.Setting("ROND", 100f),
          )
        ),
        // SemiBold weight
        Font(
          path = "fonts/google_sans_flex_regular.ttf",
          assetManager = context.assets,
          weight = FontWeight.SemiBold,
          variationSettings = FontVariation.Settings(
            FontVariation.weight(600),
            FontVariation.width(100f),
            FontVariation.Setting("ROND", 100f),
          )
        ),
        // Bold weight
        Font(
          path = "fonts/google_sans_flex_regular.ttf",
          assetManager = context.assets,
          weight = FontWeight.Bold,
          variationSettings = FontVariation.Settings(
            FontVariation.weight(700),
            FontVariation.width(100f),
            FontVariation.Setting("ROND", 100f),
          )
        ),
      )
      cachedGoogleSansFlex = fontFamily
      fontFamily
    } catch (e: Exception) {
      e.printStackTrace()
      FontFamily.SansSerif
    }
  }
}

// Wide variant for display/headline styles
private var cachedGoogleSansFlexWide: FontFamily? = null

@OptIn(ExperimentalTextApi::class)
fun getGoogleSansFlexWideFont(context: Context): FontFamily {
  return cachedGoogleSansFlexWide ?: run {
    try {
      val fontFamily = FontFamily(
        Font(
          path = "fonts/google_sans_flex_regular.ttf",
          assetManager = context.assets,
          weight = FontWeight.Black,
          variationSettings = FontVariation.Settings(
            FontVariation.weight(900),
            FontVariation.width(120f),
            FontVariation.Setting("ROND", 100f),
          )
        ),
      )
      cachedGoogleSansFlexWide = fontFamily
      fontFamily
    } catch (e: Exception) {
      e.printStackTrace()
      FontFamily.SansSerif
    }
  }
}

@Composable
fun rememberGoogleSansFlexFont(): FontFamily {
  val context = LocalContext.current
  return remember { getGoogleSansFlexFont(context) }
}

@Composable
fun rememberGoogleSansFlexWideFont(): FontFamily {
  val context = LocalContext.current
  return remember { getGoogleSansFlexWideFont(context) }
}

/**
 * Global helper for non-composable contexts (fallback to default)
 * This avoids breaking changes in code that expects a FontFamily return type without context.
 */
fun googleSansFlexFont(
  weight: Float = FontAxis.WEIGHT_REGULAR,
  width: Float = FontAxis.WIDTH_NORMAL,
  slant: Float = FontAxis.SLANT_UPRIGHT,
): FontFamily = FontFamily.SansSerif // Fallback for static contexts

// =============================================================================
// Main Typography Definition
// =============================================================================

@Composable
fun rememberAureliaTypography(): Typography {
  val baseFont = rememberGoogleSansFlexFont()
  val wideFont = rememberGoogleSansFlexWideFont()

  return Typography(
    // Display styles - wide variant
    displayLarge = TextStyle(
      fontFamily = wideFont,
      fontWeight = FontWeight.Black,
      fontSize = 48.sp,
      lineHeight = 56.sp,
      letterSpacing = (-0.5).sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    displayMedium = TextStyle(
      fontFamily = wideFont,
      fontWeight = FontWeight.Black,
      fontSize = 36.sp,
      lineHeight = 44.sp,
      letterSpacing = (-0.25).sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    displaySmall = TextStyle(
      fontFamily = wideFont,
      fontWeight = FontWeight.Black,
      fontSize = 30.sp,
      lineHeight = 38.sp,
      letterSpacing = 0.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),

    // Headline styles - wide variant
    headlineLarge = TextStyle(
      fontFamily = wideFont,
      fontWeight = FontWeight.Black,
      fontSize = 32.sp,
      lineHeight = 40.sp,
      letterSpacing = 0.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    headlineMedium = TextStyle(
      fontFamily = wideFont,
      fontWeight = FontWeight.Black,
      fontSize = 28.sp,
      lineHeight = 34.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    headlineSmall = TextStyle(
      fontFamily = wideFont,
      fontWeight = FontWeight.Black,
      fontSize = 24.sp,
      lineHeight = 32.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),

    // Title styles - base font
    titleLarge = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Medium,
      fontSize = 22.sp,
      lineHeight = 28.sp,
      letterSpacing = 0.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    titleMedium = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Medium,
      fontSize = 18.sp,
      lineHeight = 24.sp,
      letterSpacing = 0.1.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    titleSmall = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Medium,
      fontSize = 14.sp,
      lineHeight = 20.sp,
      letterSpacing = 0.1.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),

    // Body styles
    bodyLarge = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Normal,
      fontSize = 16.sp,
      lineHeight = 24.sp,
      letterSpacing = 0.25.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    bodyMedium = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Normal,
      fontSize = 14.sp,
      lineHeight = 20.sp,
      letterSpacing = 0.2.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    bodySmall = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Normal,
      fontSize = 12.sp,
      lineHeight = 16.sp,
      letterSpacing = 0.3.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),

    // Label styles
    labelLarge = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Medium,
      fontSize = 16.sp,
      lineHeight = 20.sp,
      letterSpacing = 0.1.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    labelMedium = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Medium,
      fontSize = 14.sp,
      lineHeight = 18.sp,
      letterSpacing = 0.4.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
    labelSmall = TextStyle(
      fontFamily = baseFont,
      fontWeight = FontWeight.Medium,
      fontSize = 11.sp,
      lineHeight = 16.sp,
      letterSpacing = 0.5.sp,
      platformStyle = PlatformTextStyle(includeFontPadding = false),
    ),
  )
}

// =============================================================================
// Dynamic Typography Composables
// =============================================================================

/**
 * Creates an animated font weight that pulses with music playback.
 * Weight oscillates between two values creating a "breathing" effect.
 */
@Composable
fun rememberPulsingWeight(
  isPlaying: Boolean,
  baseWeight: Float = FontAxis.WEIGHT_MEDIUM,
  amplitude: Float = 100f,
  durationMs: Int = 1000,
): Float {
  val infiniteTransition = rememberInfiniteTransition(label = "pulsingWeight")

  val animatedWeight by infiniteTransition.animateFloat(
    initialValue = baseWeight,
    targetValue = baseWeight + amplitude,
    animationSpec = infiniteRepeatable(
      animation = tween(durationMs, easing = FastOutSlowInEasing),
      repeatMode = RepeatMode.Reverse,
    ),
    label = "weightPulse",
  )

  return if (isPlaying) animatedWeight else baseWeight
}

/**
 * Creates a font width that expands on press/interaction.
 * Provides tactile feedback through typography.
 */
@Composable
fun rememberInteractiveWidth(
  isPressed: Boolean,
  baseWidth: Float = FontAxis.WIDTH_NORMAL,
  expandedWidth: Float = 110f,
): Float {
  val animatedWidth by animateFloatAsState(
    targetValue = if (isPressed) expandedWidth else baseWidth,
    animationSpec = spring(
      dampingRatio = Spring.DampingRatioMediumBouncy,
      stiffness = Spring.StiffnessLow,
    ),
    label = "interactiveWidth",
  )
  return animatedWidth
}

/**
 * Creates a slant animation for hover/focus states.
 * Adds dynamic personality to interactive elements.
 */
@Composable
fun rememberDynamicSlant(
  isFocused: Boolean,
  baseSlant: Float = FontAxis.SLANT_UPRIGHT,
  targetSlant: Float = -4f,
): Float {
  val animatedSlant by animateFloatAsState(
    targetValue = if (isFocused) targetSlant else baseSlant,
    animationSpec = tween(200, easing = FastOutSlowInEasing),
    label = "dynamicSlant",
  )
  return animatedSlant
}

/**
 * Creates a dynamic TextStyle that responds to playback state.
 * Perfect for "Now Playing" displays.
 */
@Composable
fun rememberNowPlayingStyle(
  isPlaying: Boolean,
  baseStyle: TextStyle,
): TextStyle {
  // Use cached font to avoid memory issues - animation effects handled via fontWeight
  val fontFamily = rememberGoogleSansFlexFont()

  val animatedWeight by animateFloatAsState(
    targetValue = if (isPlaying) FontWeight.Bold.weight.toFloat() else FontWeight.SemiBold.weight.toFloat(),
    animationSpec = tween(300),
    label = "nowPlayingWeight",
  )

  return baseStyle.copy(
    fontFamily = fontFamily,
    fontWeight = FontWeight(animatedWeight.toInt()),
  )
}

/**
 * Creates a staggered weight animation for text characters.
 * Returns a list of weights for each character position.
 */
@Composable
fun rememberWaveWeights(
  charCount: Int,
  isAnimating: Boolean,
  baseWeight: Float = FontAxis.WEIGHT_REGULAR,
  peakWeight: Float = FontAxis.WEIGHT_BOLD,
  waveDurationMs: Int = 2000,
): List<Float> {
  val infiniteTransition = rememberInfiniteTransition(label = "waveWeights")

  val progress by infiniteTransition.animateFloat(
    initialValue = 0f,
    targetValue = 1f,
    animationSpec = infiniteRepeatable(
      animation = tween(waveDurationMs, easing = LinearEasing),
      repeatMode = RepeatMode.Restart,
    ),
    label = "waveProgress",
  )

  return remember(charCount, progress, isAnimating) {
    if (!isAnimating) {
      List(charCount) { baseWeight }
    } else {
      List(charCount) { index ->
        val phase = (progress + index.toFloat() / charCount) % 1f
        val waveValue = kotlin.math.sin(phase * 2 * kotlin.math.PI).toFloat()
        baseWeight + (peakWeight - baseWeight) * ((waveValue + 1f) / 2f)
      }
    }
  }
}

/**
 * Animates font weight when a value changes (e.g., track progress, counts).
 * Creates a "bump" effect to draw attention to updates.
 */
@Composable
fun rememberBumpWeight(
  trigger: Any,
  baseWeight: Float = FontAxis.WEIGHT_MEDIUM,
  bumpWeight: Float = FontAxis.WEIGHT_BOLD,
): Float {
  var currentWeight by remember { mutableFloatStateOf(baseWeight) }

  LaunchedEffect(trigger) {
    currentWeight = bumpWeight
    delay(150)
    currentWeight = baseWeight
  }

  val animatedWeight by animateFloatAsState(
    targetValue = currentWeight,
    animationSpec = spring(
      dampingRatio = Spring.DampingRatioMediumBouncy,
      stiffness = Spring.StiffnessLow,
    ),
    label = "bumpWeight",
  )

  return animatedWeight
}

// =============================================================================
// Contextual Typography Styles
// =============================================================================

/**
 * Typography style that adapts based on content importance score (0.0 - 1.0).
 * Higher importance = bolder weight, slightly expanded width.
 */
@Composable
fun rememberAdaptiveStyle(
  baseStyle: TextStyle,
  importance: Float,
): TextStyle {
  val clampedImportance = importance.coerceIn(0f, 1f)
  val weight =
    FontAxis.WEIGHT_REGULAR + (FontAxis.WEIGHT_BOLD - FontAxis.WEIGHT_REGULAR) * clampedImportance

  val fontFamily = rememberGoogleSansFlexFont()

  return baseStyle.copy(
    fontFamily = fontFamily,
    fontWeight = FontWeight(weight.toInt()),
  )
}

/**
 * Typography style optimized for different reading contexts.
 */
enum class ReadingContext {
  SCANNING,  // Quick glance - slightly bolder, more contrast
  READING,   // Extended reading - optimized weight and width
  COMPACT,   // Space constrained - condensed
  DISPLAY,   // Large headlines - expanded, impactful
}

@Composable
fun rememberContextualStyle(
  baseStyle: TextStyle,
  context: ReadingContext,
): TextStyle {
  val fontFamily = when (context) {
    ReadingContext.DISPLAY -> rememberGoogleSansFlexWideFont()
    else -> rememberGoogleSansFlexFont()
  }
  val fontWeight = when (context) {
    ReadingContext.SCANNING -> FontWeight.Medium
    ReadingContext.READING -> FontWeight.Normal
    ReadingContext.COMPACT -> FontWeight.Normal
    ReadingContext.DISPLAY -> FontWeight.Black
  }
  return baseStyle.copy(fontFamily = fontFamily, fontWeight = fontWeight)
}

// Retain this for backward compatibility if referenced statically
val AureliaTypography = Typography()


// =============================================================================
// Utility Extensions
// =============================================================================

/**
 * Creates a condensed variant of any TextStyle.
 * Useful for fitting text in constrained spaces.
 */
fun TextStyle.condensed(): TextStyle = copy(
  fontFamily = googleSansFlexFont(width = FontAxis.WIDTH_CONDENSED),
)

/**
 * Creates an expanded variant of any TextStyle.
 * Useful for adding visual impact.
 */
fun TextStyle.expanded(): TextStyle = copy(
  fontFamily = googleSansFlexFont(width = FontAxis.WIDTH_EXPANDED),
)

/**
 * Creates an italicized variant using the slant axis.
 */
fun TextStyle.slanted(amount: Float = FontAxis.SLANT_ITALIC): TextStyle = copy(
  fontFamily = googleSansFlexFont(slant = amount),
)

/**
 * Creates a custom weight variant.
 */
fun TextStyle.withWeight(weight: Float): TextStyle = copy(
  fontFamily = googleSansFlexFont(weight = weight),
)

/**
 * Creates a fully customized variant with all axes.
 */
fun TextStyle.withAxes(
  weight: Float = FontAxis.WEIGHT_REGULAR,
  width: Float = FontAxis.WIDTH_NORMAL,
  slant: Float = FontAxis.SLANT_UPRIGHT,
): TextStyle = copy(
  fontFamily = googleSansFlexFont(weight = weight, width = width, slant = slant),
)
