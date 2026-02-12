package com.aurelia.app.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.platform.LocalDensity
import com.aurelia.app.audio.VisualizerStyle

@Composable
fun AudioVisualizer(
    frequencyData: List<Float>,
    timeDomainData: List<Float>,
    isPlaying: Boolean,
    style: VisualizerStyle,
    accentColor: Color,
    modifier: Modifier = Modifier,
    boost: Float = 1.0f,
) {
    val density = LocalDensity.current

    // Cache gradients keyed on accentColor to avoid recreating GPU shaders every frame.
    // Colors are listed top-to-bottom (no explicit startY/endY) so they don't depend on size.
    val barGradient = remember(accentColor) {
        Brush.verticalGradient(
            colors = listOf(
                accentColor.copy(alpha = 0.13f),
                accentColor.copy(alpha = 0.5f),
            )
        )
    }
    val curveGradient = remember(accentColor) {
        Brush.verticalGradient(
            colors = listOf(
                accentColor.copy(alpha = 0.09f),
                accentColor.copy(alpha = 0.25f),
                accentColor.copy(alpha = 0.44f),
            )
        )
    }

    val smoothedFrequency = remember { FloatArray(128) }
    val smoothedWaveform = remember { FloatArray(256) }

    val attackSmoothing = 0.8f
    val decaySmoothing = 0.15f

    val freqArray = frequencyData.toFloatArray()
    val waveArray = timeDomainData.toFloatArray()

    for (i in freqArray.indices) {
        if (i >= smoothedFrequency.size) break
        val raw = freqArray[i]
        val current = smoothedFrequency[i]
        val rate = if (raw > current) attackSmoothing else decaySmoothing
        smoothedFrequency[i] = current + (raw - current) * rate
    }

    for (i in waveArray.indices) {
        if (i >= smoothedWaveform.size) break
        val raw = waveArray[i]
        val current = smoothedWaveform[i]
        val rate = if (raw > current) attackSmoothing else decaySmoothing
        smoothedWaveform[i] = current + (raw - current) * rate
    }

    Canvas(modifier = modifier.fillMaxSize()) {
        if (!isPlaying || frequencyData.isEmpty()) {
            return@Canvas
        }

        when (style) {
            VisualizerStyle.BARS -> drawBars(
                frequencyData = smoothedFrequency,
                gradient = barGradient,
                boost = boost,
            )
            VisualizerStyle.CURVE -> drawCurve(
                frequencyData = smoothedFrequency,
                accentColor = accentColor,
                gradient = curveGradient,
                boost = boost,
            )
            VisualizerStyle.WAVE -> drawWave(
                waveformData = smoothedWaveform,
                accentColor = accentColor,
                boost = boost,
            )
        }
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawBars(
    frequencyData: FloatArray,
    gradient: Brush,
    boost: Float,
) {
    val barCount = 64
    val barWidth = size.width / barCount
    val heightScale = (size.height / 255f) * boost
    val dataStep = frequencyData.size.toFloat() / barCount

    for (i in 0 until barCount) {
        val dataIndex = (i * dataStep).toInt().coerceIn(0, frequencyData.size - 1)
        val barHeight = (frequencyData[dataIndex] * heightScale).coerceIn(0f, size.height)
        drawRect(
            brush = gradient,
            topLeft = Offset(i * barWidth, size.height - barHeight),
            size = Size(barWidth - 2, barHeight),
        )
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawCurve(
    frequencyData: FloatArray,
    accentColor: Color,
    gradient: Brush,
    boost: Float,
) {
    val sampleCount = 64
    val stepX = size.width / (sampleCount - 1)
    val heightScale = (size.height / 255f) * boost
    val dataStep = frequencyData.size.toFloat() / sampleCount

    val points = mutableListOf<Offset>()
    for (i in 0 until sampleCount) {
        val dataIndex = (i * dataStep).toInt().coerceIn(0, frequencyData.size - 1)
        val value = (frequencyData[dataIndex] * heightScale).coerceIn(0f, size.height)
        points.add(Offset(i * stepX, size.height - value))
    }

    if (points.size < 2) return

    val path = Path().apply {
        moveTo(0f, size.height)
        lineTo(points[0].x, points[0].y)

        for (i in 0 until points.size - 1) {
            val current = points[i]
            val next = points[i + 1]
            val controlX = (current.x + next.x) / 2
            val controlY = (current.y + next.y) / 2
            quadraticBezierTo(current.x, current.y, controlX, controlY)
        }

        lineTo(points.last().x, points.last().y)
        lineTo(size.width, size.height)
        close()
    }

    drawPath(path, brush = gradient)

    val strokePath = Path().apply {
        moveTo(points[0].x, points[0].y)
        for (i in 0 until points.size - 1) {
            val current = points[i]
            val next = points[i + 1]
            val controlX = (current.x + next.x) / 2
            val controlY = (current.y + next.y) / 2
            quadraticBezierTo(current.x, current.y, controlX, controlY)
        }
        lineTo(points.last().x, points.last().y)
    }

    drawPath(
        path = strokePath,
        color = accentColor.copy(alpha = 0.56f),
        style = androidx.compose.ui.graphics.drawscope.Stroke(width = 2f),
    )
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawWave(
    waveformData: FloatArray,
    accentColor: Color,
    boost: Float,
) {
    if (waveformData.isEmpty()) return

    val bufferLength = waveformData.size
    val sliceWidth = size.width / bufferLength
    val centerY = size.height / 2

    val path = Path()

    for (i in 0 until bufferLength) {
        val deviation = (waveformData[i] - 128) * boost
        val y = centerY - (deviation / 128) * centerY
        val x = i * sliceWidth

        if (i == 0) {
            path.moveTo(x, y)
        } else {
            path.lineTo(x, y)
        }
    }

    drawPath(
        path = path,
        color = accentColor.copy(alpha = 0.5f),
        style = androidx.compose.ui.graphics.drawscope.Stroke(width = 2f),
    )
}
