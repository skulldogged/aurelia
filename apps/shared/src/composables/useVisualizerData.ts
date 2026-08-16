/**
 * Composable for audio visualizer data
 *
 * Electron uses FFT snapshots from the Rust backend over WebSocket.
 * The web client uses the browser AnalyserNode.
 */
import { onUnmounted, ref, type Ref } from 'vue'

import { getAudioPlayer } from '../audio'
import { runAureliaEffect } from '../effect'
import { audioSetAnalyzerEnabledEffect } from '../effect/services/api'
import { subscribeBackendEvents } from '../lib/backend-events'
import { logger } from '../lib/logger'
import { isElectron } from '../lib/platform'

interface UseVisualizerDataReturn {
  /** Raw frequency data from FFT (0-255 per bin, 128 bins) */
  frequencyData:  Ref<Uint8Array>
  /** Whether visualizer is currently enabled */
  isEnabled:      Ref<boolean>
  /** Set visualizer enabled state (controls whether backend emits data) */
  setEnabled:     (enabled: boolean) => Promise<void>
  /** Raw time domain waveform data (0-255, 256 samples) */
  timeDomainData: Ref<Uint8Array>
}

/** FFT size used by the analyzer */
const FFT_SIZE = 256

/** Number of frequency bins (FFT_SIZE / 2) */
const FREQUENCY_BIN_COUNT = 128

/** Rendering analyzer data above 30 FPS adds churn without a visible benefit. */
const FRAME_INTERVAL_MS = 1000 / 30

/**
 * Smoothing parameters for visualizer animation.
 * Creates responsive but fluid motion: fast attack, slow decay.
 */
const SMOOTHING = {
  /** How quickly values rise to new peaks (0-1, higher = faster) */
  attack: 0.8,
  /** How quickly values fall from peaks (0-1, higher = faster) */
  decay:  0.15,
}

export const useVisualizerData = (): UseVisualizerDataReturn => {
  // Reusable conversion buffers keep smoothing work allocation-free. Publishing is
  // capped below so Vue receives a new identity only for frames it will render.
  const frequencyOutput = new Uint8Array(FREQUENCY_BIN_COUNT)
  const timeDomainOutput = new Uint8Array(FFT_SIZE)
  const frequencyData = ref<Uint8Array>(frequencyOutput)
  const timeDomainData = ref<Uint8Array>(timeDomainOutput)
  const isEnabled = ref(false)

  // Smoothing buffers (Float32 for precision during interpolation)
  let smoothedFrequency = new Float32Array(FREQUENCY_BIN_COUNT)
  let smoothedTimeDomain = new Float32Array(FFT_SIZE)
  let lastFrameTime = -FRAME_INTERVAL_MS

  // Event listener cleanup (desktop)
  let eventUnlisten: (() => void) | null = null

  // Animation frame ID (web)
  let animationFrameId: null | number = null
  let analyserRetryId: null | ReturnType<typeof setInterval> = null

  /**
   * Apply temporal smoothing to spectrum data.
   * Fast attack (responds to beats), slow decay (smooth falloff).
   */
  const applySmoothingAndUpdate = (
    rawFreq: ArrayLike<number>,
    rawTime: ArrayLike<number>,
  ): void => {
    const now = performance.now()
    if (now - lastFrameTime < FRAME_INTERVAL_MS) return
    lastFrameTime = now

    const frequencyLength = Math.min(rawFreq.length, smoothedFrequency.length)
    for (let i = 0; i < frequencyLength; i++) {
      const raw = rawFreq[i]
      const current = smoothedFrequency[i]
      const rate = raw > current ? SMOOTHING.attack : SMOOTHING.decay
      smoothedFrequency[i] = current + (raw - current) * rate
    }

    const timeDomainLength = Math.min(rawTime.length, smoothedTimeDomain.length)
    for (let i = 0; i < timeDomainLength; i++) {
      const raw = rawTime[i]
      const current = smoothedTimeDomain[i]
      const rate = raw > current ? SMOOTHING.attack : SMOOTHING.decay
      smoothedTimeDomain[i] = current + (raw - current) * rate
    }

    for (let i = 0; i < smoothedFrequency.length; i++)
      frequencyOutput[i] = Math.round(smoothedFrequency[i])

    for (let i = 0; i < smoothedTimeDomain.length; i++)
      timeDomainOutput[i] = Math.round(smoothedTimeDomain[i])

    frequencyData.value = frequencyOutput.slice()
    timeDomainData.value = timeDomainOutput.slice()
  }

  /**
   * Electron: listen for FFT snapshots from the local Rust backend.
   */
  const setupDesktopEventListener = (): void => {
    if (eventUnlisten) {
      eventUnlisten()
      eventUnlisten = null
    }

    eventUnlisten = subscribeBackendEvents(event => {
      if (event.type !== 'AudioSpectrum') return
      applySmoothingAndUpdate(
        event.data.frequencyData,
        event.data.timeDomainData,
      )
    })

    logger.debug('Spectrum event listener registered (electron)')
  }

  /**
   * Web: Setup Web Audio API analyzer polling
   */
  const clearAnalyserRetry = (): void => {
    if (analyserRetryId === null) return
    clearInterval(analyserRetryId)
    analyserRetryId = null
  }

  const setupWebAnalyzer = (): void => {
    const attach = (): boolean => {
      const audioPlayer = getAudioPlayer()
      const analyserNode = audioPlayer.getAnalyserNode()

      if (!analyserNode) return false

      analyserNode.fftSize = FFT_SIZE
      analyserNode.smoothingTimeConstant = 0.8

      const frequencyBuffer = new Uint8Array(analyserNode.frequencyBinCount)
      const timeDomainBuffer = new Uint8Array(analyserNode.fftSize)

      const update = (): void => {
        if (!isEnabled.value) return

        analyserNode.getByteFrequencyData(frequencyBuffer)
        analyserNode.getByteTimeDomainData(timeDomainBuffer)

        applySmoothingAndUpdate(frequencyBuffer, timeDomainBuffer)

        animationFrameId = requestAnimationFrame(update)
      }

      animationFrameId = requestAnimationFrame(update)
      logger.debug('Web Audio analyzer started')
      return true
    }

    if (attach()) return

    logger.debug('Web Audio analyzer node not ready yet, retrying')
    clearAnalyserRetry()
    analyserRetryId = setInterval(() => {
      if (!isEnabled.value) {
        clearAnalyserRetry()
        return
      }
      if (attach()) clearAnalyserRetry()
    }, 200)
  }

  /**
   * Start the data source based on platform
   */
  const startDataSource = async (): Promise<void> => {
    if (isElectron()) {
      if (!eventUnlisten) {
        setupDesktopEventListener()
      }
    } else {
      setupWebAnalyzer()
    }
  }

  /**
   * Stop the data source and cleanup
   */
  const stopDataSource = (): void => {
    // Desktop cleanup
    if (eventUnlisten) {
      eventUnlisten()
      eventUnlisten = null
    }

    // Web cleanup
    clearAnalyserRetry()
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId)
      animationFrameId = null
    }

    // Clear data and reset smoothing buffers
    frequencyOutput.fill(0)
    timeDomainOutput.fill(0)
    frequencyData.value = new Uint8Array(FREQUENCY_BIN_COUNT)
    timeDomainData.value = new Uint8Array(FFT_SIZE)
    smoothedFrequency = new Float32Array(FREQUENCY_BIN_COUNT)
    smoothedTimeDomain = new Float32Array(FFT_SIZE)
    lastFrameTime = -FRAME_INTERVAL_MS
  }

  // Set analyzer enabled state
  const setEnabled = async (enabled: boolean): Promise<void> => {
    try {
      logger.debug(`setEnabled called: enabled=${enabled}`)

      // For desktop, notify backend
      if (isElectron())
        await runAureliaEffect(audioSetAnalyzerEnabledEffect(enabled))

      isEnabled.value = enabled

      if (enabled) {
        await startDataSource()
      } else {
        stopDataSource()
      }

      logger.debug(`Spectrum analyzer ${enabled ? 'enabled' : 'disabled'}`)
    } catch (error) {
      logger.error('Failed to set analyzer enabled:', error)
    }
  }

  // Cleanup on unmount
  onUnmounted(() => {
    stopDataSource()
    // Disable analyzer when component unmounts to save CPU
    if (isElectron()) {
      runAureliaEffect(audioSetAnalyzerEnabledEffect(false)).catch(() => {})
    }
  })

  return {
    frequencyData,
    isEnabled,
    setEnabled,
    timeDomainData,
  }
}
