/**
 * Composable for audio visualizer data from Rust backend
 *
 * Provides real-time spectrum and waveform data via Tauri events.
 * The FFT analysis is performed in Rust for optimal performance.
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

export const useVisualizerData = (): UseVisualizerDataReturn => {
  // Reactive data buffers - pre-allocate for performance
  const frequencyData = ref<Uint8Array>(new Uint8Array(FREQUENCY_BIN_COUNT))
  const timeDomainData = ref<Uint8Array>(new Uint8Array(FFT_SIZE))
  const isEnabled = ref(false)

  // Event listener cleanup
  let eventUnlisten: null | UnlistenFn = null

  // Setup event listener for spectrum data
  const setupEventListener = async (): Promise<void> => {
    // Clean up existing listener
    if (eventUnlisten) {
      eventUnlisten()
      eventUnlisten = null
    }

    eventUnlisten = await listen<SpectrumEvent>('audio:spectrum', event => {
      const { frequencyData: freqData, timeDomainData: timeData } = event.payload

      // Update reactive refs with new data
      frequencyData.value = new Uint8Array(freqData)
      timeDomainData.value = new Uint8Array(timeData)
    })

    logger.debug('Spectrum event listener registered')
  }

  // Set analyzer enabled state in backend
  const setEnabled = async (enabled: boolean): Promise<void> => {
    try {
      const result = await commands.audioSetAnalyzerEnabled(enabled)
      if (result.status === 'error') {
        logger.error('Failed to set analyzer enabled:', result.error)
        return
      }
      isEnabled.value = enabled

      if (enabled && !eventUnlisten) {
        await setupEventListener()
      } else if (!enabled && eventUnlisten) {
        eventUnlisten()
        eventUnlisten = null
        // Clear data when disabled
        frequencyData.value = new Uint8Array(FREQUENCY_BIN_COUNT)
        timeDomainData.value = new Uint8Array(FFT_SIZE)
      }

      logger.debug(`Spectrum analyzer ${enabled ? 'enabled' : 'disabled'}`)
    } catch (error) {
      logger.error('Failed to set analyzer enabled:', error)
    }
  }

  // Auto-cleanup listener based on enabled state
  watch(isEnabled, async enabled => {
    if (enabled && !eventUnlisten) {
      await setupEventListener()
    }
  })

  // Cleanup on unmount
  onUnmounted(() => {
    if (eventUnlisten) {
      eventUnlisten()
      eventUnlisten = null
    }
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
