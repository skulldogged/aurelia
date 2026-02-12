package com.aurelia.app.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import com.aurelia.app.audio.VisualizerStyle

@Composable
fun AudioVisualizer(
    frequencyData: ByteArray,
    timeDomainData: ByteArray,
    style: VisualizerStyle,
    accentColor: Color,
    modifier: Modifier = Modifier,
    boost: Float = 1.0f,
) {
    // Cache gradients keyed on accentColor to avoid recreating GPU shaders every frame.
    val barGradient = remember(accentColor) {
        Brush.verticalGradient(
            colors = listOf(
                accentColor.copy(alpha = 0.06f),
                accentColor.copy(alpha = 0.38f),
                accentColor.copy(alpha = 0.06f),
            )
        )
    }
    val curveGradient = remember(accentColor) {
        Brush.verticalGradient(
            colors = listOf(
                accentColor.copy(alpha = 0.08f),
                accentColor.copy(alpha = 0.28f),
                accentColor.copy(alpha = 0.08f),
            )
        )
    }

    Canvas(modifier = modifier.fillMaxSize()) {
        if (frequencyData.isEmpty() && timeDomainData.isEmpty()) {
            return@Canvas
        }

        when (style) {
            VisualizerStyle.BARS -> drawBars(
                frequencyData = frequencyData,
                gradient = barGradient,
                boost = boost,
            )
            VisualizerStyle.CURVE -> drawCurve(
                frequencyData = frequencyData,
                accentColor = accentColor,
                gradient = curveGradient,
                boost = boost,
            )
            VisualizerStyle.WAVE -> drawWave(
                waveformData = timeDomainData,
                accentColor = accentColor,
                boost = boost,
            )
        }
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawBars(
    frequencyData: ByteArray,
    gradient: Brush,
    boost: Float,
) {
    if (frequencyData.isEmpty()) return

    val barCount = 64
    val barWidth = size.width / barCount
    val centerY = size.height / 2f
    val maxHalfHeight = (size.height * 0.46f) * boost
    val minHalfHeight = size.height * 0.02f
    val dataStep = frequencyData.size.toFloat() / barCount

    for (i in 0 until barCount) {
        val dataIndex = (i * dataStep).toInt().coerceIn(0, frequencyData.size - 1)
        val value = frequencyData[dataIndex].toInt() and 0xFF
        val normalized = value / 255f
        val halfHeight = (normalized * maxHalfHeight).coerceAtLeast(minHalfHeight)
        val left = i * barWidth + 1f
        val width = (barWidth - 2f).coerceAtLeast(1f)
        val top = centerY - halfHeight
        val height = (halfHeight * 2f).coerceAtMost(size.height)
        drawRoundRect(
            brush = gradient,
            topLeft = Offset(left, top.coerceAtLeast(0f)),
            size = Size(width, height),
            cornerRadius = CornerRadius(width / 2f, width / 2f),
        )
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawCurve(
    frequencyData: ByteArray,
    accentColor: Color,
    gradient: Brush,
    boost: Float,
) {
    if (frequencyData.isEmpty()) return

    val sampleCount = 64
    val stepX = size.width / (sampleCount - 1)
    val maxHeight = (size.height * 0.86f) * boost
    val minHeight = size.height * 0.03f
    val dataStep = frequencyData.size.toFloat() / sampleCount

    val crestPoints = mutableListOf<Offset>()
    for (i in 0 until sampleCount) {
        val dataIndex = (i * dataStep).toInt().coerceIn(0, frequencyData.size - 1)
        val value = frequencyData[dataIndex].toInt() and 0xFF
        val amplitude = ((value / 255f) * maxHeight).coerceAtLeast(minHeight)
        val x = i * stepX
        crestPoints.add(Offset(x, size.height - amplitude))
    }

    if (crestPoints.size < 2) return

    val path = Path().apply {
        moveTo(0f, size.height)
        lineTo(crestPoints[0].x, crestPoints[0].y)

        for (i in 0 until crestPoints.size - 1) {
            val current = crestPoints[i]
            val next = crestPoints[i + 1]
            val controlX = (current.x + next.x) / 2
            val controlY = (current.y + next.y) / 2
            quadraticTo(current.x, current.y, controlX, controlY)
        }
        lineTo(crestPoints.last().x, crestPoints.last().y)
        lineTo(size.width, size.height)
        close()
    }

    drawPath(path, brush = gradient)

    val strokePath = Path().apply {
        moveTo(crestPoints[0].x, crestPoints[0].y)
        for (i in 0 until crestPoints.size - 1) {
            val current = crestPoints[i]
            val next = crestPoints[i + 1]
            val controlX = (current.x + next.x) / 2
            val controlY = (current.y + next.y) / 2
            quadraticTo(current.x, current.y, controlX, controlY)
        }
        lineTo(crestPoints.last().x, crestPoints.last().y)
    }

    drawPath(
        path = strokePath,
        color = accentColor.copy(alpha = 0.62f),
        style = androidx.compose.ui.graphics.drawscope.Stroke(width = 2f),
    )
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawWave(
    waveformData: ByteArray,
    accentColor: Color,
    boost: Float,
) {
    if (waveformData.isEmpty()) return

    val sampleCount = 96
    val dataStep = waveformData.size.toFloat() / sampleCount
    val stepX = size.width / (sampleCount - 1)
    val centerY = size.height / 2f
    val points = mutableListOf<Offset>()

    for (i in 0 until sampleCount.coerceAtLeast(2)) {
        val dataIndex = (i * dataStep).toInt().coerceIn(0, waveformData.size - 1)
        val sample = waveformData[dataIndex].toInt() and 0xFF
        val deviation = (sample - 128f) * (boost * 0.9f)
        val y = centerY - (deviation / 128f) * centerY
        val x = i * stepX
        points.add(Offset(x, y))
    }

    if (points.size < 2) return

    val path = Path().apply {
        moveTo(points[0].x, points[0].y)
        for (i in 0 until points.size - 1) {
            val current = points[i]
            val next = points[i + 1]
            val midX = (current.x + next.x) / 2f
            val midY = (current.y + next.y) / 2f
            quadraticTo(current.x, current.y, midX, midY)
        }
        lineTo(points.last().x, points.last().y)
    }

    drawPath(
        path = path,
        color = accentColor.copy(alpha = 0.58f),
        style = androidx.compose.ui.graphics.drawscope.Stroke(
            width = 2.25f,
            cap = StrokeCap.Round,
            join = StrokeJoin.Round,
        ),
    )
}
