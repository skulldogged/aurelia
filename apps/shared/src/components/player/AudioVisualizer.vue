<script setup lang="ts">
  import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

  interface VisualizerProps {
    /** Amplitude boost multiplier (1.0 = normal, 2.0 = double height) */
    boost?:         number
    /** Frequency domain data from Rust FFT (0-255 per bin) */
    frequencyData:  Uint8Array
    /** Whether audio is currently playing */
    isPlaying:      boolean
    /** Visualization style */
    style?:         'bars' | 'curve' | 'wave'
    /** Time domain waveform data (0-255, centered at 128) */
    timeDomainData: Uint8Array
  }

  const props = withDefaults(defineProps<VisualizerProps>(), {
    boost: 1.0,
    style: 'bars',
  })

  const canvasRef = ref<HTMLCanvasElement | null>(null)
  const animationFrameId = ref<null | number>(null)

  // Performance: Cache accent color and gradients
  const accentColor = ref<string>('#3b82f6')
  const accentRgb = ref<{ b: number; g: number; r: number; }>({ b: 246, g: 130, r: 59 })
  const gradientBars = ref<CanvasGradient | null>(null)
  const gradientMirrorTop = ref<CanvasGradient | null>(null)
  const gradientMirrorBottom = ref<CanvasGradient | null>(null)
  const gradientCurve = ref<CanvasGradient | null>(null)

  const updateAccentColor = (): void => {
    const accentColorHex = getComputedStyle(document.documentElement)
      .getPropertyValue('--color-accent')
      .trim() || '#3b82f6'

    // Convert hex color to RGB values for proper alpha handling
    const hex = accentColorHex.replace('#', '')
    const r = parseInt(hex.substr(0, 2), 16)
    const g = parseInt(hex.substr(2, 2), 16)
    const b = parseInt(hex.substr(4, 2), 16)

    accentColor.value = `rgb(${r}, ${g}, ${b})`

    // Store RGB values for reuse
    accentRgb.value = { b, g, r }

    // Re-create gradients when color changes
    if (canvasRef.value) {
      const ctx = canvasRef.value.getContext('2d')
      if (ctx) {
        const height = canvasRef.value.height
        const centerY = height / 2

        // For drawBars
        gradientBars.value = ctx.createLinearGradient(0, height, 0, 0)
        gradientBars.value.addColorStop(0, `rgba(${r}, ${g}, ${b}, 0.5)`)
        gradientBars.value.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0.13)`)

        // For drawBarsMirror
        gradientMirrorTop.value = ctx.createLinearGradient(0, centerY, 0, 0)
        gradientMirrorTop.value.addColorStop(0, `rgba(${r}, ${g}, ${b}, 0.38)`)
        gradientMirrorTop.value.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0.06)`)

        gradientMirrorBottom.value = ctx.createLinearGradient(0, centerY, 0, height)
        gradientMirrorBottom.value.addColorStop(0, `rgba(${r}, ${g}, ${b}, 0.38)`)
        gradientMirrorBottom.value.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0.06)`)

        // For drawCircular
        gradientCurve.value = ctx.createLinearGradient(0, height, 0, 0)
        gradientCurve.value.addColorStop(0, `rgba(${r}, ${g}, ${b}, 0.44)`)
        gradientCurve.value.addColorStop(0.5, `rgba(${r}, ${g}, ${b}, 0.25)`)
        gradientCurve.value.addColorStop(1, `rgba(${r}, ${g}, ${b}, 0.09)`)
      }
    }
  }

  const drawBars = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!gradientBars.value || props.frequencyData.length === 0) return

    ctx.clearRect(0, 0, width, height)

    const barCount = 64
    const barWidth = width / barCount
    const heightScale = (height / 255) * props.boost
    const dataStep = props.frequencyData.length / barCount

    ctx.fillStyle = gradientBars.value

    for (let i = 0; i < barCount; i++) {
      const dataIndex = Math.floor(i * dataStep)
      const barHeight = Math.min(props.frequencyData[dataIndex] * heightScale, height)
      ctx.fillRect(i * barWidth, height - barHeight, barWidth - 2, barHeight)
    }
  }

  const drawCircular = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!gradientCurve.value || props.frequencyData.length === 0) return

    ctx.clearRect(0, 0, width, height)

    const sampleCount = 64
    const stepX = width / (sampleCount - 1)
    const heightScale = (height / 255) * props.boost
    const dataStep = props.frequencyData.length / sampleCount

    const points: Array<{ x: number, y: number }> = []
    for (let i = 0; i < sampleCount; i++) {
      const dataIndex = Math.floor(i * dataStep)
      const value = Math.min(props.frequencyData[dataIndex] * heightScale, height)
      points.push({ x: i * stepX, y: height - value })
    }

    ctx.beginPath()
    ctx.moveTo(0, height)
    ctx.lineTo(points[0].x, points[0].y)

    for (let i = 0; i < points.length - 1; i++) {
      const current = points[i]
      const next = points[i + 1]
      const controlX = (current.x + next.x) / 2
      const controlY = (current.y + next.y) / 2
      ctx.quadraticCurveTo(current.x, current.y, controlX, controlY)
    }

    const lastPoint = points[points.length - 1]
    ctx.lineTo(lastPoint.x, lastPoint.y)
    ctx.lineTo(width, height)
    ctx.closePath()

    ctx.fillStyle = gradientCurve.value
    ctx.fill()

    ctx.beginPath()
    ctx.moveTo(points[0].x, points[0].y)

    for (let i = 0; i < points.length - 1; i++) {
      const current = points[i]
      const next = points[i + 1]
      const controlX = (current.x + next.x) / 2
      const controlY = (current.y + next.y) / 2
      ctx.quadraticCurveTo(current.x, current.y, controlX, controlY)
    }

    ctx.lineTo(lastPoint.x, lastPoint.y)
    const rgb = accentRgb.value
    ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.56)`
    ctx.lineWidth = 2
    ctx.stroke()
  }

  const drawWave = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (props.timeDomainData.length === 0) return

    ctx.clearRect(0, 0, width, height)

    ctx.strokeStyle = `rgba(${accentRgb.value.r}, ${accentRgb.value.g}, ${accentRgb.value.b}, 0.5)`
    ctx.lineWidth = 2
    ctx.beginPath()

    const bufferLength = props.timeDomainData.length
    const sliceWidth = width / bufferLength
    const centerY = height / 2
    let x = 0

    for (let i = 0; i < bufferLength; i++) {
      // Apply boost to deviation from center (128)
      const deviation = (props.timeDomainData[i] - 128) * props.boost
      const y = centerY - (deviation / 128) * centerY
      if (i === 0)
        ctx.moveTo(x, y)
      else
        ctx.lineTo(x, y)
      x += sliceWidth
    }

    ctx.stroke()
  }

  const animate = (): void => {
    if (!canvasRef.value) return

    const canvas = canvasRef.value
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()
    if (canvas.width !== rect.width * dpr || canvas.height !== rect.height * dpr) {
      canvas.width = rect.width * dpr
      canvas.height = rect.height * dpr
      ctx.scale(dpr, dpr)
      updateAccentColor() // Re-create gradients for new canvas size
    }

    const width = rect.width
    const height = rect.height

    if (props.isPlaying && props.frequencyData.length > 0) {
      switch (props.style) {
        case 'bars':
          drawBars(ctx, width, height)
          break
        case 'curve':
          drawCircular(ctx, width, height)
          break
        case 'wave':
          drawWave(ctx, width, height)
          break
      }
    } else {
      ctx.clearRect(0, 0, width, height)
    }

    animationFrameId.value = requestAnimationFrame(animate)
  }

  const startAnimation = (): void => {
    if (animationFrameId.value !== null) return
    updateAccentColor()
    animate()
  }

  const stopAnimation = (): void => {
    if (animationFrameId.value !== null) {
      cancelAnimationFrame(animationFrameId.value)
      animationFrameId.value = null
    }
    if (canvasRef.value) {
      const ctx = canvasRef.value.getContext('2d')
      if (ctx)
        ctx.clearRect(0, 0, canvasRef.value.width, canvasRef.value.height)
    }
  }

  const handleResize = (): void => {
    if (canvasRef.value && props.isPlaying) {
      const canvas = canvasRef.value
      const ctx = canvas.getContext('2d')
      if (ctx) {
        const dpr = window.devicePixelRatio || 1
        const rect = canvas.getBoundingClientRect()
        canvas.width = rect.width * dpr
        canvas.height = rect.height * dpr
        ctx.scale(dpr, dpr)
        updateAccentColor()
      }
    }
  }

  onMounted(() => {
    startAnimation()
    window.addEventListener('resize', handleResize)
    // Watch for theme changes to update accent color
    const observer = new MutationObserver(updateAccentColor)
    observer.observe(document.documentElement, {
      attributeFilter: ['style', 'class'],
      attributes:      true,
    })
    onBeforeUnmount(() => observer.disconnect())
  })

  onBeforeUnmount(() => {
    stopAnimation()
    window.removeEventListener('resize', handleResize)
  })

  watch(() => props.isPlaying, isPlaying => {
    if (!isPlaying && canvasRef.value) {
      const ctx = canvasRef.value.getContext('2d')
      if (ctx)
        ctx.clearRect(0, 0, canvasRef.value.width, canvasRef.value.height)
    }
  })
</script>

<template>
  <canvas
    ref='canvasRef'
    aria-hidden='true'
    class='absolute inset-0 w-full h-full pointer-events-none'
    style='will-change: transform;'
  />
</template>
