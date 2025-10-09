<script setup lang="ts">
  import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

  interface VisualizerProps {
    analyserNode: AnalyserNode | null
    isPlaying:    boolean
    style?:       'bars' | 'bars-mirror' | 'curve' | 'wave'
  }

  const props = withDefaults(defineProps<VisualizerProps>(), {
    style: 'bars-mirror',
  })

  const canvasRef = ref<HTMLCanvasElement | null>(null)
  const animationFrameId = ref<null | number>(null)

  let dataArray: null | Uint8Array = null
  let bufferLength = 0

  const initializeVisualizer = (): void => {
    if (!props.analyserNode) return

    // Configure analyser for better frequency resolution - we don't mutate the prop, just use it
    const analyser = props.analyserNode
    analyser.fftSize = 256 // 128 frequency bins
    bufferLength = analyser.frequencyBinCount
    dataArray = new Uint8Array(bufferLength) as Uint8Array
  }

  const drawBars = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!props.analyserNode || !dataArray) return

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    props.analyserNode.getByteFrequencyData(dataArray as any)

    ctx.clearRect(0, 0, width, height)

    const barCount = 64 // Use fewer bars for cleaner look
    const barWidth = width / barCount
    const heightScale = height / 255

    for (let i = 0; i < barCount; i++) {
      // Sample data array evenly
      const dataIndex = Math.floor((i * bufferLength) / barCount)
      const barHeight = dataArray[dataIndex] * heightScale

      // Create gradient from accent color
      const gradient = ctx.createLinearGradient(0, height, 0, height - barHeight)
      const accentColor = getComputedStyle(document.documentElement)
        .getPropertyValue('--color-accent')
        .trim() || '#3b82f6'

      gradient.addColorStop(0, `${accentColor}80`) // 50% opacity
      gradient.addColorStop(1, `${accentColor}20`) // 12% opacity

      ctx.fillStyle = gradient
      ctx.fillRect(i * barWidth, height - barHeight, barWidth - 2, barHeight)
    }
  }

  const drawBarsMirror = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!props.analyserNode || !dataArray) return

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    props.analyserNode.getByteFrequencyData(dataArray as any)

    ctx.clearRect(0, 0, width, height)

    const barCount = 64
    const barWidth = width / barCount
    const centerY = height / 2
    const heightScale = centerY / 255

    for (let i = 0; i < barCount; i++) {
      const dataIndex = Math.floor((i * bufferLength) / barCount)
      const barHeight = dataArray[dataIndex] * heightScale

      const accentColor = getComputedStyle(document.documentElement)
        .getPropertyValue('--color-accent')
        .trim() || '#3b82f6'

      // Top half (upward)
      const gradientTop = ctx.createLinearGradient(0, centerY, 0, centerY - barHeight)
      gradientTop.addColorStop(0, `${accentColor}60`)
      gradientTop.addColorStop(1, `${accentColor}10`)

      ctx.fillStyle = gradientTop
      ctx.fillRect(i * barWidth, centerY - barHeight, barWidth - 2, barHeight)

      // Bottom half (downward - mirror)
      const gradientBottom = ctx.createLinearGradient(0, centerY, 0, centerY + barHeight)
      gradientBottom.addColorStop(0, `${accentColor}60`)
      gradientBottom.addColorStop(1, `${accentColor}10`)

      ctx.fillStyle = gradientBottom
      ctx.fillRect(i * barWidth, centerY, barWidth - 2, barHeight)
    }
  }

  const drawCircular = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!props.analyserNode || !dataArray) return

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    props.analyserNode.getByteFrequencyData(dataArray as any)

    ctx.clearRect(0, 0, width, height)

    const sampleCount = 64
    const stepX = width / (sampleCount - 1)
    const heightScale = height / 255

    const accentColor = getComputedStyle(document.documentElement)
      .getPropertyValue('--color-accent')
      .trim() || '#3b82f6'

    // Get frequency data points
    const points: Array<{ x: number, y: number }> = []
    for (let i = 0; i < sampleCount; i++) {
      const dataIndex = Math.floor((i * bufferLength) / sampleCount)
      const value = dataArray[dataIndex] * heightScale
      points.push({
        x: i * stepX,
        y: height - value,
      })
    }

    // Draw smooth curve using bezier curves
    ctx.beginPath()
    ctx.moveTo(0, height)
    ctx.lineTo(points[0].x, points[0].y)

    // Use quadratic curves for smooth interpolation
    for (let i = 0; i < points.length - 1; i++) {
      const current = points[i]
      const next = points[i + 1]

      // Calculate control point (midpoint between current and next)
      const controlX = (current.x + next.x) / 2
      const controlY = (current.y + next.y) / 2

      ctx.quadraticCurveTo(current.x, current.y, controlX, controlY)
    }

    // Complete the last segment
    const lastPoint = points[points.length - 1]
    ctx.lineTo(lastPoint.x, lastPoint.y)

    // Close the path to create filled area
    ctx.lineTo(width, height)
    ctx.closePath()

    // Create gradient fill
    const gradient = ctx.createLinearGradient(0, height, 0, 0)
    gradient.addColorStop(0, `${accentColor}70`)
    gradient.addColorStop(0.5, `${accentColor}40`)
    gradient.addColorStop(1, `${accentColor}15`)

    ctx.fillStyle = gradient
    ctx.fill()

    // Draw stroke for the curve
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

    ctx.strokeStyle = `${accentColor}90`
    ctx.lineWidth = 2
    ctx.stroke()
  }

  const drawWave = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!props.analyserNode || !dataArray) return

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    props.analyserNode.getByteTimeDomainData(dataArray as any)

    ctx.clearRect(0, 0, width, height)

    const accentColor = getComputedStyle(document.documentElement)
      .getPropertyValue('--color-accent')
      .trim() || '#3b82f6'

    ctx.strokeStyle = `${accentColor}80`
    ctx.lineWidth = 2
    ctx.beginPath()

    const sliceWidth = width / bufferLength
    let x = 0

    for (let i = 0; i < bufferLength; i++) {
      const v = dataArray[i] / 128.0
      const y = (v * height) / 2

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

    // Match canvas resolution to display size for crisp rendering
    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()

    canvas.width = rect.width * dpr
    canvas.height = rect.height * dpr

    ctx.scale(dpr, dpr)

    const width = rect.width
    const height = rect.height

    // Only draw if playing
    if (props.isPlaying && props.analyserNode) {
      switch (props.style) {
        case 'bars':
          drawBars(ctx, width, height)
          break
        case 'bars-mirror':
          drawBarsMirror(ctx, width, height)
          break
        case 'curve':
          drawCircular(ctx, width, height)
          break
        case 'wave':
          drawWave(ctx, width, height)
          break
      }
    } else {
      // Clear canvas when not playing
      ctx.clearRect(0, 0, width, height)
    }

    animationFrameId.value = requestAnimationFrame(animate)
  }

  const startAnimation = (): void => {
    if (animationFrameId.value !== null) return
    initializeVisualizer()
    animate()
  }

  const stopAnimation = (): void => {
    if (animationFrameId.value !== null) {
      cancelAnimationFrame(animationFrameId.value)
      animationFrameId.value = null
    }

    // Clear canvas
    if (canvasRef.value) {
      const ctx = canvasRef.value.getContext('2d')
      if (ctx)
        ctx.clearRect(0, 0, canvasRef.value.width, canvasRef.value.height)
    }
  }

  // Handle canvas resize
  const handleResize = (): void => {
    if (canvasRef.value && props.isPlaying)
      animate()
  }

  onMounted(() => {
    startAnimation()
    window.addEventListener('resize', handleResize)
  })

  onBeforeUnmount(() => {
    stopAnimation()
    window.removeEventListener('resize', handleResize)
  })

  // Restart animation when analyser changes
  watch(() => props.analyserNode, () => {
    stopAnimation()
    startAnimation()
  })

  // Update when playing state changes
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
  />
</template>
