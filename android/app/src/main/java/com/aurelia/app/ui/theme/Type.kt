package com.aurelia.app.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

private val DefaultFont = FontFamily.SansSerif

val AureliaTypography =
    Typography(
        displayLarge =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Bold,
                fontSize = 48.sp,
                lineHeight = 56.sp,
                letterSpacing = 0.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        displayMedium =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Bold,
                fontSize = 36.sp,
                lineHeight = 44.sp,
                letterSpacing = 0.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        displaySmall =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Normal,
                fontSize = 30.sp,
                lineHeight = 38.sp,
                letterSpacing = 0.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        headlineLarge =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.SemiBold,
                fontSize = 32.sp,
                lineHeight = 40.sp,
                letterSpacing = 0.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        headlineMedium =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.SemiBold,
                fontSize = 28.sp,
                lineHeight = 34.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        headlineSmall =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.SemiBold,
                fontSize = 24.sp,
                lineHeight = 32.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        titleLarge =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Normal,
                fontSize = 22.sp,
                lineHeight = 28.sp,
                letterSpacing = 0.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        titleMedium =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Medium,
                fontSize = 18.sp,
                lineHeight = 24.sp,
                letterSpacing = 0.15.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        titleSmall =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Medium,
                fontSize = 14.sp,
                lineHeight = 20.sp,
                letterSpacing = 0.1.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        bodyLarge =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Normal,
                fontSize = 16.sp,
                lineHeight = 24.sp,
                letterSpacing = 0.5.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        bodyMedium =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Normal,
                fontSize = 14.sp,
                lineHeight = 20.sp,
                letterSpacing = 0.25.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        bodySmall =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Normal,
                fontSize = 12.sp,
                lineHeight = 16.sp,
                letterSpacing = 0.4.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        labelLarge =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Medium,
                fontSize = 16.sp,
                lineHeight = 20.sp,
                letterSpacing = 0.1.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        labelMedium =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Medium,
                fontSize = 14.sp,
                lineHeight = 18.sp,
                letterSpacing = 0.5.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
        labelSmall =
            TextStyle(
                fontFamily = DefaultFont,
                fontWeight = FontWeight.Medium,
                fontSize = 11.sp,
                lineHeight = 16.sp,
                letterSpacing = 0.5.sp,
                platformStyle = PlatformTextStyle(includeFontPadding = false),
            ),
    )
