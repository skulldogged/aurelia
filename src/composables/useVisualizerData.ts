/**
 * Composable for audio visualizer data from Rust backend
 *
 * Provides real-time spectrum and waveform data.
 * - Desktop: Uses Tauri events from Rust FFT analysis
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onUnmounted, ref, type Ref, watch } from 'vue'

import { commands } from '@/bindings'
import { logger } from '@/lib/logger'

interface SpectrumEvent {
  frequencyData:   number[]
  timeDomainData:  number[]
}

interface UseVisualizerDataReturn {
  /** Raw frequency data from FFT (0-255 per bin, 128 bins) */
  frequencyData:   Ref<Uint8Array>
  /** Whether visualizer is currently enabled */
  isEnabled:       Ref<boolean>
  /** Set visualizer enabled state (controls whether backend emits data) */
  setEnabled:      (enabled: boolean) => Promise<void>
  /** Raw time domain waveform data (0-255, 256 samples) */
  timeDomainData:  Ref<Uint8Array>
}

/** FFT size used by the Rust analyzer */
const FFT_SIZE = 256

/** Number of frequency bins (FFT_SIZE / 2) */
const FREQUENCY_BIN_COUNT = 128

/**
 * Smoothing parameters for visualizer animation.
 * Creates responsive but fluid motion: fast attack, slow decay.
 */
const SMOOTHING = {
  /** How quickly values rise to new peaks (0-1, higher = faster) */
  attack: 0.8,
  /** How quickly values fall from peaks (0-1, higher = faster) */
  decay: 0.15,
}


export const useVisualizerData = (): UseVisualizerDataReturn => {
  // Reactive data buffers - pre-allocate for performance
  const frequencyData = ref<Uint8Array>(new Uint8Array(FREQUENCY_BIN_COUNT))
  const timeDomainData = ref<Uint8Array>(new Uint8Array(FFT_SIZE))
  const isEnabled = ref(false)

  // Smoothing buffers (Float32 for precision during interpolation)
  let smoothedFrequency = new Float32Array(FREQUENCY_BIN_COUNT)
  let smoothedTimeDomain = new Float32Array(FFT_SIZE)

  // Event listener cleanup (desktop)
  let eventUnlisten: null | UnlistenFn = null

  /**
   * Apply temporal smoothing to spectrum data.
   * Fast attack (responds to beats), slow decay (smooth falloff).
   */
  const applySmoothingAndUpdate = (
    rawFreq: Uint8Array,
    rawTime: Uint8Array,
  ): void => {
    // Smooth frequency data
    for (let i = 0; i < rawFreq.length; i++) {
      const raw = rawFreq[i]
      const current = smoothedFrequency[i]
      // Use attack rate when rising, decay rate when falling
      const rate = raw > current ? SMOOTHING.attack : SMOOTHING.decay
      smoothedFrequency[i] = current + (raw - current) * rate
    }

    // Smooth time domain data
    for (let i = 0; i < rawTime.length; i++) {
      const raw = rawTime[i]
      const current = smoothedTimeDomain[i]
      const rate = raw > current ? SMOOTHING.attack : SMOOTHING.decay
      smoothedTimeDomain[i] = current + (raw - current) * rate
    }

    // Convert to Uint8Array for output
    const freqOut = new Uint8Array(smoothedFrequency.length)
    for (let i = 0; i < smoothedFrequency.length; i++) {
      freqOut[i] = Math.round(smoothedFrequency[i])
    }

    const timeOut = new Uint8Array(smoothedTimeDomain.length)
    for (let i = 0; i < smoothedTimeDomain.length; i++) {
      timeOut[i] = Math.round(smoothedTimeDomain[i])
    }

    frequencyData.value = freqOut
    timeDomainData.value = timeOut
  }

  /**
   * Desktop: Setup event listener for spectrum data from Rust.
   */
  const setupEventListener = async (): Promise<void> => {
    // Clean up existing listener
    if (eventUnlisten) {
      eventUnlisten()
      eventUnlisten = null
    }

    eventUnlisten = await listen<SpectrumEvent>('audio:spectrum', event => {
      const { frequencyData: freqData, timeDomainData: timeData } = event.payload

      // Apply smoothing and update reactive refs
      applySmoothingAndUpdate(new Uint8Array(freqData), new Uint8Array(timeData))
    })

    logger.debug('Spectrum event listener registered')
  }

  /**
   * Start the data source.
   */
  const startDataSource = async (): Promise<void> => {
    if (!eventUnlisten) {
      await setupEventListener()
    }
  }

  /**
   * Stop the data source and cleanup.
   */
  const stopDataSource = (): void => {
    if (eventUnlisten) {
      eventUnlisten()
      eventUnlisten = null
    }

    // Clear data and reset smoothing buffers
    frequencyData.value = new Uint8Array(FREQUENCY_BIN_COUNT)
    timeDomainData.value = new Uint8Array(FFT_SIZE)
    smoothedFrequency = new Float32Array(FREQUENCY_BIN_COUNT)
    smoothedTimeDomain = new Float32Array(FFT_SIZE)
  }

  // Set analyzer enabled state in backend
  const setEnabled = async (enabled: boolean): Promise<void> => {
    try {
      logger.debug(`setEnabled called: enabled=${enabled}`)

      const result = await commands.audioSetAnalyzerEnabled(enabled)
      if (result.status === 'error') {
        logger.error('Failed to set analyzer enabled:', result.error)
        return
      }

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

  // Auto-cleanup listener based on enabled state
  watch(isEnabled, async enabled => {
    if (enabled) {
      await startDataSource()
    }
  })

  // Cleanup on unmount
  onUnmounted(() => {
    stopDataSource()
    // Disable analyzer when component unmounts to save CPU
    commands.audioSetAnalyzerEnabled(false).catch(() => {})
  })

  return {
    frequencyData,
    isEnabled,
    setEnabled,
    timeDomainData,
  }
}
