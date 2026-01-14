/**
 * Composable for audio visualizer data from Rust backend
 *
 * Provides real-time spectrum and waveform data.
 * - Desktop: Uses Tauri events from Rust FFT analysis
 * - Android: Uses high-performance polling via JavascriptInterface at display refresh rate
 */
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onUnmounted, ref, type Ref, watch } from 'vue'

import { commands } from '@/bindings'
import { logger } from '@/lib/logger'
import { getPlatform, Platform } from '@/lib/platform'

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

// Type for Android's JavascriptInterface
interface AureliaSpectrum {
  getData: () => string
  getVersion: () => number
}

// Declare the global interface injected by Android
declare global {
  interface Window {
    AureliaSpectrum?: AureliaSpectrum
  }
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

  // Animation frame ID for Android polling
  let animationFrameId: null | number = null
  let lastVersion = -1

  // Platform detection
  const currentPlatform = getPlatform()
  const isAndroid = currentPlatform === Platform.Android

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
   * Android: Poll spectrum data at display refresh rate using requestAnimationFrame.
   * This provides smooth 60/90/120fps visualization matching the device's display.
   */
  const pollAndroidSpectrum = (): void => {
    if (!isEnabled.value) {
      animationFrameId = null
      return
    }

    const spectrum = window.AureliaSpectrum
    if (spectrum) {
      // Check version first to avoid parsing if data hasn't changed
      const version = spectrum.getVersion()
      if (version !== lastVersion) {
        lastVersion = version
        const data = spectrum.getData()
        if (data) {
          parseSpectrumData(data)
        }
      }
    }

    // Continue polling at display refresh rate
    animationFrameId = requestAnimationFrame(pollAndroidSpectrum)
  }

  /**
   * Parse the compact spectrum data format from Android.
   * Format: "version,freq0,freq1,...|time0,time1,..."
   */
  const parseSpectrumData = (data: string): void => {
    const pipeIndex = data.indexOf('|')
    if (pipeIndex === -1) return

    const freqPart = data.substring(data.indexOf(',') + 1, pipeIndex)
    const timePart = data.substring(pipeIndex + 1)

    // Parse frequency data
    const freqValues = freqPart.split(',')
    const rawFreqData = new Uint8Array(freqValues.length)
    for (let i = 0; i < freqValues.length; i++) {
      rawFreqData[i] = parseInt(freqValues[i], 10)
    }

    // Parse time domain data
    const timeValues = timePart.split(',')
    const rawTimeData = new Uint8Array(timeValues.length)
    for (let i = 0; i < timeValues.length; i++) {
      rawTimeData[i] = parseInt(timeValues[i], 10)
    }

    // Apply smoothing and update reactive refs
    applySmoothingAndUpdate(rawFreqData, rawTimeData)
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
   * Start the appropriate data source based on platform.
   */
  const startDataSource = async (): Promise<void> => {
    if (isAndroid) {
      // Android: Start polling loop
      if (animationFrameId === null) {
        lastVersion = -1
        animationFrameId = requestAnimationFrame(pollAndroidSpectrum)
        logger.debug('Android spectrum polling started')
      }
    } else {
      // Desktop: Use Tauri events
      if (!eventUnlisten) {
        await setupEventListener()
      }
    }
  }

  /**
   * Stop the data source and cleanup.
   */
  const stopDataSource = (): void => {
    if (isAndroid) {
      // Android: Stop polling
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId)
        animationFrameId = null
        logger.debug('Android spectrum polling stopped')
      }
    } else {
      // Desktop: Remove event listener
      if (eventUnlisten) {
        eventUnlisten()
        eventUnlisten = null
      }
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
      logger.debug(`setEnabled called: enabled=${enabled}, platform=${currentPlatform}`)

      // On Android, check and request RECORD_AUDIO permission first
      if (enabled && isAndroid) {
        const hasPermission = await checkAndRequestRecordPermission()
        if (!hasPermission) {
          logger.warn('RECORD_AUDIO permission denied, cannot enable visualizer')
          isEnabled.value = false
          return
        }
      }

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

  // Check and request RECORD_AUDIO permission on Android
  const checkAndRequestRecordPermission = async (): Promise<boolean> => {
    try {
      // First check if we already have permission
      const checkResult = await invoke<boolean>('audio_check_record_permission')
      logger.debug(`RECORD_AUDIO permission check: ${checkResult}`)

      if (checkResult) {
        return true
      }

      // Request permission
      logger.info('Requesting RECORD_AUDIO permission for visualizer...')
      const requestResult = await invoke<boolean>('audio_request_record_permission')
      logger.debug(`RECORD_AUDIO permission request result: ${requestResult}`)

      return requestResult
    } catch (error) {
      logger.error('Failed to check/request RECORD_AUDIO permission:', error)
      return false
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
