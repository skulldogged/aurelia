import { playerLogger } from '@/lib/logger'
import { usePlayerStore } from '@/stores/player'

// EQ Band interface
export interface EQBand {
  frequency: number,
  gain:      number,
  type:      BiquadFilterType,
  Q:         number,
}

// EQ Preset interface
export interface EQPreset {
  name:  string,
  bands: EQBand[],
}

// Web Audio API player implementation with streaming support
let audioContext: AudioContext | null = null
let mediaElement: HTMLAudioElement | null = null
let mediaSource: MediaElementAudioSourceNode | null = null
let gainNode: GainNode | null = null
let analyserNode: AnalyserNode | null = null
let eqNodes: BiquadFilterNode[] | null = null
let eqEnabled: boolean = false
let pendingVolume: number = 1.0 // Store volume to apply when gain node is created
let isPlaying: boolean = false
let onDurationChange: ((duration: number) => void) | null = null

// Standard EQ bands (5-band EQ)
const DEFAULT_EQ_BANDS: EQBand[] = [
  { frequency: 60, gain: 0, type: 'lowshelf', Q: 1.414 },     // Bass
  { frequency: 250, gain: 0, type: 'peaking', Q: 1.414 },    // Low Mids
  { frequency: 1000, gain: 0, type: 'peaking', Q: 1.414 },   // Mids
  { frequency: 4000, gain: 0, type: 'peaking', Q: 1.414 },   // High Mids
  { frequency: 16000, gain: 0, type: 'highshelf', Q: 1.414 },  // Treble
]

// EQ Presets
const EQ_PRESETS : EQPreset[] = [
  {
    name:  'Flat',
    bands: DEFAULT_EQ_BANDS.map(band => ({ ...band, gain: 0 })),
  },
  {
    name:  'Rock',
    bands: [
      { frequency: 60, gain: 3, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: 2, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: -1, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 1, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: 2, type: 'highshelf', Q: 1.414 },
    ],
  },
  {
    name:  'Pop',
    bands: [
      { frequency: 60, gain: -2, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: 1, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: 3, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 2, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: 1, type: 'highshelf', Q: 1.414 },
    ],
  },
  {
    name:  'Jazz',
    bands: [
      { frequency: 60, gain: 2, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: -1, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: 2, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 3, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: 0, type: 'highshelf', Q: 1.414 },
    ],
  },
  {
    name:  'Classical',
    bands: [
      { frequency: 60, gain: 1, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: -1, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: 1, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 2, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: 3, type: 'highshelf', Q: 1.414 },
    ],
  },
  {
    name:  'Hip Hop',
    bands: [
      { frequency: 60, gain: 4, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: 3, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: -2, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 0, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: -1, type: 'highshelf', Q: 1.414 },
    ],
  },
  {
    name:  'Electronic',
    bands: [
      { frequency: 60, gain: 2, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: -3, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: 4, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 2, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: 3, type: 'highshelf', Q: 1.414 },
    ],
  },
  {
    name:  'Vocal',
    bands: [
      { frequency: 60, gain: -2, type: 'lowshelf', Q: 1.414 },
      { frequency: 250, gain: -1, type: 'peaking', Q: 1.414 },
      { frequency: 1000, gain: 4, type: 'peaking', Q: 1.414 },
      { frequency: 4000, gain: 1, type: 'peaking', Q: 1.414 },
      { frequency: 16000, gain: 2, type: 'highshelf', Q: 1.414 },
    ],
  },
]

// Initialize Web Audio API context with streaming support
const initializeWebAudio = async (): Promise<boolean> => {
  try {
    // Create audio context if it doesn't exist
    if (!audioContext) {
      const AudioContextClass =
        window.AudioContext ||
        (window as typeof window & { webkitAudioContext: typeof AudioContext }).webkitAudioContext
      audioContext = new AudioContextClass()

      // Resume context if it's suspended (required by some browsers)
      if (audioContext.state === 'suspended') {
        await audioContext.resume()
      }

      playerLogger.debug('WebAudio API initialized successfully')
    }
    return true
  } catch (error) {
    playerLogger.error('Failed to initialize WebAudio API:', error)
    return false
  }
}

// Check if Web Audio API is available
const isWebAudioAvailable = (): boolean => {
  try {
    const AudioContextClass =
      window.AudioContext ||
      (window as typeof window & { webkitAudioContext: typeof AudioContext }).webkitAudioContext
    const testContext = new AudioContextClass()
    testContext.close() // Clean up test context
    return true
  } catch {
    return false
  }
}

// Initialize EQ nodes
const initializeEQ = (): boolean => {
  try {
    if (!audioContext) {
      playerLogger.error('Cannot initialize EQ: AudioContext not available')
      return false
    }

    // Clean up existing EQ nodes
    if (eqNodes) {
      eqNodes.forEach(node => node.disconnect())
    }

    // Create EQ filter nodes for each band
    eqNodes = DEFAULT_EQ_BANDS.map(band => {
      const filter = audioContext!.createBiquadFilter()
      filter.type = band.type as BiquadFilterType
      filter.frequency.value = band.frequency
      filter.gain.value = band.gain
      filter.Q.value = band.Q

      // Log the filter configuration
      playerLogger.debug(`Created EQ filter: ${band.type} @ ${band.frequency}Hz, Q=${band.Q}, gain=${band.gain}dB`)

      return filter
    })

    playerLogger.debug('EQ initialized with 5 bands')
    return true
  } catch (error) {
    playerLogger.error('Failed to initialize EQ:', error)
    return false
  }
}

// Load stored EQ bands into WebAudio nodes
const loadStoredEQBands = (): boolean => {
  try {
    if (!eqNodes) {
      playerLogger.error('Cannot load EQ bands: EQ not initialized')
      return false
    }

    // Get stored EQ bands from localStorage
    const stored = localStorage.getItem('player-eq-bands')
    if (!stored) {
      playerLogger.debug('No stored EQ bands found, using defaults')
      return true
    }

    const parsed = JSON.parse(stored) as EQBand[]
    if (!Array.isArray(parsed) || parsed.length !== 5) {
      playerLogger.warn('Invalid stored EQ bands format, using defaults')
      return true
    }

    // Apply stored values to WebAudio nodes
    parsed.forEach((band: { gain?: number }, index: number) => {
      if (index < eqNodes!.length && typeof band.gain === 'number') {
        const clampedGain = Math.max(-20, Math.min(20, band.gain))
        eqNodes![index].gain.value = clampedGain
      }
    })

    return true
  } catch (error) {
    playerLogger.error('Failed to load stored EQ bands:', error)
    return false
  }
}

// Update EQ connections in audio graph
const updateAudioGraph = (): boolean => {
  try {
    if (!audioContext || !mediaSource || !gainNode || !analyserNode) {
      playerLogger.error('Cannot update audio graph: missing nodes')
      return false
    }

    // Disconnect everything first
    mediaSource.disconnect()
    gainNode.disconnect()
    if (eqNodes) eqNodes.forEach(node => node.disconnect())
    analyserNode.disconnect()

    // Reconnect the audio graph
    let currentNode: AudioNode = mediaSource

    // Connect source to gain
    currentNode.connect(gainNode)
    currentNode = gainNode

    // Connect EQ if enabled
    if (eqEnabled && eqNodes) {
      eqNodes.forEach((eqNode, index) => {
        try {
          currentNode.connect(eqNode)
          currentNode = eqNode
        } catch (error) {
          playerLogger.error(`Failed to connect EQ band ${index}:`, error)
        }
      })
    }

    // Connect to analyser and destination
    currentNode.connect(analyserNode)
    analyserNode.connect(audioContext.destination)

    return true
  } catch (error) {
    playerLogger.error('Failed to update audio graph:', error)
    return false
  }
}

// Load audio from URL using streaming approach
const loadAudio = async (url: string): Promise<boolean> => {
  try {
    if (!audioContext) {
      throw new Error('AudioContext not initialized')
    }

    // Stop any current playback
    stop()

    playerLogger.debug(`Loading streaming audio via WebAudio API: ${url}`)

    // Create HTML5 audio element for streaming
    if (!mediaElement) {
      mediaElement = new Audio()
      mediaElement.crossOrigin = 'anonymous'
      mediaElement.preload = 'metadata' // Load metadata but not the full audio
    }

    // Set the source URL
    mediaElement.src = url

    // Create MediaElementAudioSourceNode if it doesn't exist
    if (!mediaSource) {
      mediaSource = audioContext.createMediaElementSource(mediaElement)

      // Create gain node for volume control
      gainNode = audioContext.createGain()

      // Create analyser node
      analyserNode = audioContext.createAnalyser()

      // Initialize EQ
      initializeEQ()

      // Load stored EQ bands into the nodes
      loadStoredEQBands()

      // Sync EQ enabled state with player store
      const playerStore = usePlayerStore()
      eqEnabled = playerStore.eqEnabled

      // Apply any pending volume
      gainNode.gain.value = pendingVolume

      // Setup audio graph
      updateAudioGraph()

      // Setup event listeners for playback state tracking
      setupEventListeners()
    }

    // Wait for metadata to load so we have duration information
    return new Promise(resolve => {
      const onMetadataLoaded = () => {
        mediaElement?.removeEventListener('loadedmetadata', onMetadataLoaded)
        mediaElement?.removeEventListener('error', onError)
        playerLogger.debug(`Streaming audio metadata loaded, duration: ${mediaElement?.duration || 0}s`)
        resolve(true)
      }

      const onError = (e: Event) => {
        mediaElement?.removeEventListener('loadedmetadata', onMetadataLoaded)
        mediaElement?.removeEventListener('error', onError)
        playerLogger.error('Failed to load streaming audio metadata:', e)
        resolve(false)
      }

      if (mediaElement && mediaElement.readyState >= 1) {
        // Metadata already loaded
        playerLogger.debug(`Streaming audio metadata already loaded, duration: ${mediaElement.duration}s`)
        resolve(true)
      } else {
        // Wait for metadata
        mediaElement?.addEventListener('loadedmetadata', onMetadataLoaded)
        mediaElement?.addEventListener('error', onError)

        // Load the audio to trigger metadata loading
        mediaElement?.load()
      }
    })
  } catch (error) {
    playerLogger.error('Failed to load streaming audio via WebAudio API:', error)
    return false
  }
}

// Play audio using HTML5 element with Web Audio processing
const play = async (): Promise<boolean> => {
  try {
    if (!mediaElement) {
      throw new Error('Audio not loaded')
    }

    // Ensure audio context is running
    if (audioContext?.state === 'suspended') {
      await audioContext.resume()
    }

    await mediaElement.play()
    isPlaying = true

    playerLogger.debug('WebAudio streaming playback started')
    return true
  } catch (error) {
    playerLogger.error('Failed to start WebAudio streaming playback:', error)
    isPlaying = false
    return false
  }
}

// Pause audio
const pause = (): boolean => {
  try {
    if (mediaElement && !mediaElement.paused) {
      mediaElement.pause()
      isPlaying = false
      playerLogger.debug(`WebAudio streaming playback paused at ${mediaElement.currentTime}s`)
      return true
    }
    return false
  } catch (error) {
    playerLogger.error('Failed to pause WebAudio streaming playback:', error)
    isPlaying = false
    return false
  }
}

// Stop audio
const stop = (): boolean => {
  try {
    if (mediaElement) {
      mediaElement.pause()
      mediaElement.currentTime = 0
    }
    isPlaying = false
    playerLogger.debug('WebAudio streaming playback stopped')
    return true
  } catch (error) {
    playerLogger.error('Failed to stop WebAudio streaming playback:', error)
    isPlaying = false
    return false
  }
}

// Seek to position
const seek = async (time: number): Promise<boolean> => {
  try {
    if (!mediaElement) {
      throw new Error('Audio not loaded')
    }

    const clampedTime = Math.max(0, Math.min(time, mediaElement.duration || 0))
    const _wasPlaying = !mediaElement.paused

    mediaElement.currentTime = clampedTime

    // If it was playing before, it should continue playing after seek
    // The HTML5 audio element handles this automatically

    playerLogger.debug(`WebAudio streaming seeked to ${clampedTime}s`)
    return true
  } catch (error) {
    playerLogger.error('Failed to seek WebAudio streaming playback:', error)
    return false
  }
}

// Set volume (0-1)
const setVolume = (volume: number): boolean => {
  try {
    const clampedVolume = Math.max(0, Math.min(1, volume))
    pendingVolume = clampedVolume

    if (gainNode) {
      gainNode.gain.value = clampedVolume
      playerLogger.debug(`WebAudio volume set to ${clampedVolume}`)
      return true
    }

    // Gain node not ready yet, volume will be applied when it's created
    playerLogger.debug(`WebAudio volume stored for later: ${clampedVolume}`)
    return true
  } catch (error) {
    playerLogger.error('Failed to set WebAudio volume:', error)
    return false
  }
}

// Get current playback time
const getCurrentTime = (): number => mediaElement?.currentTime || 0

// Get duration
const getDuration = (): number => {
  const duration = mediaElement?.duration
  return (duration && !isNaN(duration) && isFinite(duration) && duration > 0) ? duration : 0
}

// Check if audio is currently playing
const getIsPlaying = (): boolean => isPlaying && mediaElement ? !mediaElement.paused && !mediaElement.ended : false

// Check if audio is loaded and ready
const getIsReady = (): boolean => {
  return mediaElement ? mediaElement.readyState >= 2 : false // HAVE_CURRENT_DATA or higher
}

// Setup event listeners for HTML5 audio element
const setupEventListeners = () => {
  if (!mediaElement) return

  // Handle playback end to trigger next song
  mediaElement.addEventListener('ended', () => {
    isPlaying = false
    playerLogger.debug('Streaming audio ended, advancing to next song')

    // Trigger next song advancement
    setTimeout(() => {
      if (typeof window !== 'undefined') {
        const w = window as typeof window & { advanceToNextSong?: () => void }
        if (w.advanceToNextSong)
          w.advanceToNextSong()
      }
    }, 100)
  })

  // Update playing state
  mediaElement.addEventListener('play', () => {
    isPlaying = true
    playerLogger.debug('Streaming audio started playing')
  })

  mediaElement.addEventListener('pause', () => {
    isPlaying = false
    playerLogger.debug('Streaming audio paused')
  })

  // Handle metadata loading
  mediaElement.addEventListener('loadedmetadata', () => {
    if (mediaElement) {
      playerLogger.debug(`Metadata loaded, duration: ${mediaElement.duration}s`)
    }
  })

  // Handle duration changes
  mediaElement.addEventListener('durationchange', () => {
    if (mediaElement) {
      const duration = mediaElement.duration
      if (duration && !isNaN(duration) && isFinite(duration) && duration > 0) {
        playerLogger.debug(`Duration updated to: ${duration}s`)
        if (onDurationChange) {
          onDurationChange(duration)
        }
      }
    }
  })

  // Handle errors
  mediaElement.addEventListener('error', e => {
    playerLogger.error('Streaming audio error:', e)
    isPlaying = false
  })
}

// EQ Control Functions

// Enable/Disable EQ
const setEQEnabled = (enabled: boolean): boolean => {
  try {
    eqEnabled = enabled

    // Sync with player store
    const playerStore = usePlayerStore()
    if (playerStore.eqEnabled !== enabled) {
      playerStore.setEQEnabled(enabled)
    }

    if (enabled) {
      // Make sure EQ nodes exist before enabling
      if (!eqNodes) {
        if (!initializeEQ()) {
          return false
        }
      }

      // Always load stored bands when enabling EQ
      loadStoredEQBands()

      // Force update the audio graph
      if (!updateAudioGraph())
        return false
    } else {
      if (!updateAudioGraph())
        playerLogger.error('Failed to update audio graph when disabling EQ')
    }
    return true
  } catch (error) {
    playerLogger.error('Failed to toggle EQ:', error)
    return false
  }
}

// Get EQ enabled state
const getEQEnabled = (): boolean => eqEnabled

// Set EQ band gain
const setEQBandGain = (bandIndex: number, gain: number): boolean => {
  try {
    if (!eqNodes || bandIndex < 0 || bandIndex >= eqNodes.length) {
      playerLogger.error(`Invalid EQ band index: ${bandIndex}`)
      return false
    }

    const clampedGain = Math.max(-20, Math.min(20, gain)) // Clamp to -20dB to +20dB
    eqNodes[bandIndex].gain.value = clampedGain

    // Sync with player store
    const playerStore = usePlayerStore()
    playerStore.setEQBandGain(bandIndex, clampedGain)

    return true
  } catch (error) {
    playerLogger.error(`Failed to set EQ band ${bandIndex} gain:`, error)
    return false
  }
}

// Get EQ band gain
const getEQBandGain = (bandIndex: number): number => {
  if (!eqNodes || bandIndex < 0 || bandIndex >= eqNodes.length) {
    return 0
  }
  return eqNodes[bandIndex].gain.value
}

// Get all EQ bands
const getEQBands = (): EQBand[] => {
  return DEFAULT_EQ_BANDS.map((band, index) => ({
    ...band,
    gain: getEQBandGain(index),
  }))
}

// Apply EQ preset
const applyEQPreset = (presetName: string): boolean => {
  try {
    const preset = EQ_PRESETS.find(p => p.name === presetName)
    if (!preset) {
      playerLogger.error(`EQ preset not found: ${presetName}`)
      return false
    }

    preset.bands.forEach((band, index) => {
      setEQBandGain(index, band.gain)
    })

    playerLogger.debug(`EQ preset applied: ${presetName}`)
    return true
  } catch (error) {
    playerLogger.error(`Failed to apply EQ preset ${presetName}:`, error)
    return false
  }
}

// Get available EQ presets
const getEQPresets = (): EQPreset[] => EQ_PRESETS

// Reset EQ to flat
const resetEQ = (): boolean => {
  try {
    DEFAULT_EQ_BANDS.forEach((band, index) => {
      setEQBandGain(index, 0)
    })
    playerLogger.debug('EQ reset to flat')
    return true
  } catch (error) {
    playerLogger.error('Failed to reset EQ:', error)
    return false
  }
}

// Set callback for duration changes
const setOnDurationChange = (callback: (duration: number) => void) => {
  onDurationChange = callback
}

// Cleanup Web Audio resources
const cleanup = () => {
  try {
    stop()
    if (mediaElement) {
      mediaElement.src = ''
      mediaElement = null
    }
    if (mediaSource) {
      mediaSource.disconnect()
      mediaSource = null
    }
    if (gainNode) {
      gainNode.disconnect()
      gainNode = null
    }
    if (eqNodes) {
      eqNodes.forEach(node => node.disconnect())
      eqNodes = null
    }
    if (analyserNode) {
      analyserNode.disconnect()
      analyserNode = null
    }
    if (audioContext && audioContext.state !== 'closed') {
      audioContext.close()
    }
    audioContext = null
    pendingVolume = 1.0
    eqEnabled = false
    onDurationChange = null
    playerLogger.debug('WebAudio streaming resources cleaned up')
  } catch (error) {
    playerLogger.error('Failed to cleanup WebAudio streaming resources:', error)
  }
}

export const useWebAudioPlayer = () => ({
  // Initialization
  initializeWebAudio,
  isWebAudioAvailable,
  cleanup,

  // Audio loading and playback
  loadAudio,
  play,
  pause,
  stop,
  seek: seek as (time: number) => Promise<boolean>,
  setVolume,

  // State getters
  getCurrentTime,
  getDuration,
  getIsPlaying,
  getIsReady,

  // EQ Controls
  setEQEnabled,
  getEQEnabled,
  setEQBandGain,
  getEQBandGain,
  getEQBands,
  applyEQPreset,
  getEQPresets,
  resetEQ,

  // EQ Utilities
  loadStoredEQBands,

  // Callbacks
  setOnDurationChange,
})

