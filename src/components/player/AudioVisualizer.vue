<script setup lang="ts">
  import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

  interface VisualizerProps {
    analyserNode: AnalyserNode | null
    isPlaying:    boolean
    style?:       'bars' | 'curve' | 'wave'
  }

  const props = withDefaults(defineProps<VisualizerProps>(), {
    style: 'bars',
  })

  const canvasRef = ref<HTMLCanvasElement | null>(null)
  const animationFrameId = ref<null | number>(null)

  let dataArray: null | Uint8Array<ArrayBuffer> = null
  let bufferLength = 0

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

  const initializeVisualizer = (): void => {
    if (!props.analyserNode) return

    const analyser = props.analyserNode
    analyser.fftSize = 256
    bufferLength = analyser.frequencyBinCount
    dataArray = new Uint8Array(bufferLength)
  }

  const drawBars = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!props.analyserNode || !dataArray || !gradientBars.value) return

    props.analyserNode.getByteFrequencyData(dataArray)
    ctx.clearRect(0, 0, width, height)

    const barCount = 64
    const barWidth = width / barCount
    const heightScale = height / 255
    const dataStep = bufferLength / barCount

    ctx.fillStyle = gradientBars.value

    for (let i = 0; i < barCount; i++) {
      const dataIndex = Math.floor(i * dataStep)
      const barHeight = dataArray[dataIndex] * heightScale
      ctx.fillRect(i * barWidth, height - barHeight, barWidth - 2, barHeight)
    }
  }

  const drawCircular = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void => {
    if (!props.analyserNode || !dataArray || !gradientCurve.value) return

    props.analyserNode.getByteFrequencyData(dataArray)
    ctx.clearRect(0, 0, width, height)

    const sampleCount = 64
    const stepX = width / (sampleCount - 1)
    const heightScale = height / 255
    const dataStep = bufferLength / sampleCount

    const points: Array<{ x: number, y: number }> = []
    for (let i = 0; i < sampleCount; i++) {
      const dataIndex = Math.floor(i * dataStep)
      const value = dataArray[dataIndex] * heightScale
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
    if (!props.analyserNode || !dataArray) return

    props.analyserNode.getByteTimeDomainData(dataArray)
    ctx.clearRect(0, 0, width, height)

    ctx.strokeStyle = `rgba(${accentRgb.value.r}, ${accentRgb.value.g}, ${accentRgb.value.b}, 0.5)`
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

    if (props.isPlaying && props.analyserNode) {
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
    initializeVisualizer()
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

  watch(() => props.analyserNode, () => {
    stopAnimation()
    startAnimation()
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
