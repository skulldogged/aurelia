import { playerLogger } from '@/lib/logger'
import { usePlayerStore } from '@/stores/player'

export interface EQBand {
  frequency: number,
  gain:      number,
  Q:         number,
  type:      BiquadFilterType,
}

export interface EQPreset {
  bands: EQBand[],
  name:  string,
}

export interface WebAudioPlayer {
  applyEQPreset:       (presetName: string) => boolean
  cleanup:             () => void
  getCurrentTime:      () => number
  getDuration:         () => number
  getEQBandGain:       (bandIndex: number) => number
  getEQBands:          () => EQBand[]
  getEQEnabled:        () => boolean
  getEQPresets:        () => EQPreset[]
  getIsPlaying:        () => boolean
  getIsReady:          () => boolean
  initializeWebAudio:  () => Promise<boolean>
  isWebAudioAvailable: () => boolean
  loadAudio:           (url: string) => Promise<boolean>
  loadStoredEQBands:   () => boolean
  pause:               () => boolean
  play:                () => Promise<boolean>
  resetEQ:             () => boolean
  seek:                (time: number) => Promise<boolean>
  setEQBandGain:       (bandIndex: number, gain: number) => boolean
  setEQEnabled:        (enabled: boolean) => boolean
  setOnDurationChange: (callback: (duration: number) => void) => void
  setVolume:           (volume: number) => boolean
  stop:                () => boolean
}

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
  { frequency: 60, gain: 0, Q: 1.414, type: 'lowshelf' },
  { frequency: 250, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 1000, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 4000, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 16000, gain: 0, Q: 1.414, type: 'highshelf' },
]

// EQ Presets
const EQ_PRESETS : EQPreset[] = [
  {
    bands: DEFAULT_EQ_BANDS.map(band => ({ ...band, gain: 0 })),
    name:  'Flat',
  },
  {
    bands: [
      { frequency: 60, gain: 3, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 2, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Rock',
  },
  {
    bands: [
      { frequency: 60, gain: -2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 3, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 1, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Pop',
  },
  {
    bands: [
      { frequency: 60, gain: 2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 3, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 0, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Jazz',
  },
  {
    bands: [
      { frequency: 60, gain: 1, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 3, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Classical',
  },
  {
    bands: [
      { frequency: 60, gain: 4, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: 3, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: -2, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 0, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: -1, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Hip Hop',
  },
  {
    bands: [
      { frequency: 60, gain: 2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -3, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 4, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 3, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Electronic',
  },
  {
    bands: [
      { frequency: 60, gain: -2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 4, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 2, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Vocal',
  },
]

const initializeWebAudio = async (): Promise<boolean> => {
  try {
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

const initializeEQ = (): boolean => {
  try {
    if (!audioContext) {
      playerLogger.error('Cannot initialize EQ: AudioContext not available')
      return false
    }

    if (eqNodes)
      eqNodes.forEach(node => node.disconnect())

    eqNodes = DEFAULT_EQ_BANDS.map(band => {
      const filter = audioContext!.createBiquadFilter()
      filter.type = band.type as BiquadFilterType
      filter.frequency.value = band.frequency
      filter.gain.value = band.gain
      filter.Q.value = band.Q

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

const loadStoredEQBands = (): boolean => {
  try {
    if (!eqNodes) {
      playerLogger.error('Cannot load EQ bands: EQ not initialized')
      return false
    }

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

    let currentNode: AudioNode = mediaSource

    currentNode.connect(gainNode)
    currentNode = gainNode

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

    currentNode.connect(analyserNode)
    analyserNode.connect(audioContext.destination)

    return true
  } catch (error) {
    playerLogger.error('Failed to update audio graph:', error)
    return false
  }
}

const loadAudio = async (url: string): Promise<boolean> => {
  try {
    if (!audioContext)
      throw new Error('AudioContext not initialized')

    stop()

    playerLogger.debug(`Loading streaming audio via WebAudio API: ${url}`)

    if (!mediaElement) {
      mediaElement = new Audio()
      mediaElement.crossOrigin = 'anonymous'
      mediaElement.preload = 'metadata' // Load metadata but not the full audio
    }

    mediaElement.src = url

    if (!mediaSource) {
      mediaSource = audioContext.createMediaElementSource(mediaElement)

      gainNode = audioContext.createGain()

      analyserNode = audioContext.createAnalyser()

      initializeEQ()

      loadStoredEQBands()

      const playerStore = usePlayerStore()
      eqEnabled = playerStore.eqEnabled

      gainNode.gain.value = pendingVolume

      updateAudioGraph()

      setupEventListeners()
    }

    return new Promise(resolve => {
      const onMetadataLoaded = (): void => {
        mediaElement?.removeEventListener('loadedmetadata', onMetadataLoaded)
        mediaElement?.removeEventListener('error', onError)
        playerLogger.debug(`Streaming audio metadata loaded, duration: ${mediaElement?.duration || 0}s`)
        resolve(true)
      }

      const onError = (e: Event): void => {
        mediaElement?.removeEventListener('loadedmetadata', onMetadataLoaded)
        mediaElement?.removeEventListener('error', onError)
        playerLogger.error('Failed to load streaming audio metadata:', e)
        resolve(false)
      }

      if (mediaElement && mediaElement.readyState >= 1) {
        playerLogger.debug(`Streaming audio metadata already loaded, duration: ${mediaElement.duration}s`)
        resolve(true)
      } else {
        mediaElement?.addEventListener('loadedmetadata', onMetadataLoaded)
        mediaElement?.addEventListener('error', onError)

        mediaElement?.load()
      }
    })
  } catch (error) {
    playerLogger.error('Failed to load streaming audio via WebAudio API:', error)
    return false
  }
}

const play = async (): Promise<boolean> => {
  try {
    if (!mediaElement)
      throw new Error('Audio not loaded')

    if (audioContext?.state === 'suspended')
      await audioContext.resume()

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

const seek = async (time: number): Promise<boolean> => {
  try {
    if (!mediaElement)
      throw new Error('Audio not loaded')

    const clampedTime = Math.max(0, Math.min(time, mediaElement.duration || 0))

    mediaElement.currentTime = clampedTime

    playerLogger.debug(`WebAudio streaming seeked to ${clampedTime}s`)
    return true
  } catch (error) {
    playerLogger.error('Failed to seek WebAudio streaming playback:', error)
    return false
  }
}

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

const getCurrentTime = (): number => mediaElement?.currentTime || 0

const getDuration = (): number => {
  const duration = mediaElement?.duration
  return (duration && !isNaN(duration) && isFinite(duration) && duration > 0) ? duration : 0
}

const getIsPlaying = (): boolean =>
  isPlaying && mediaElement
    ? !mediaElement.paused && !mediaElement.ended
    : false

const getIsReady = (): boolean =>
  mediaElement
    ? mediaElement.readyState >= 2
    : false

const setupEventListeners = (): void => {
  if (!mediaElement) return

  mediaElement.addEventListener('ended', () => {
    isPlaying = false
    playerLogger.debug('Streaming audio ended, advancing to next song')

    setTimeout(() => {
      if (typeof window !== 'undefined') {
        const w = window as typeof window & { advanceToNextSong?: () => void }
        if (w.advanceToNextSong)
          w.advanceToNextSong()
      }
    }, 100)
  })

  mediaElement.addEventListener('play', () => {
    isPlaying = true
    playerLogger.debug('Streaming audio started playing')
  })

  mediaElement.addEventListener('pause', () => {
    isPlaying = false
    playerLogger.debug('Streaming audio paused')
  })

  mediaElement.addEventListener('loadedmetadata', () => {
    if (mediaElement)
      playerLogger.debug(`Metadata loaded, duration: ${mediaElement.duration}s`)
  })

  mediaElement.addEventListener('durationchange', () => {
    if (mediaElement) {
      const duration = mediaElement.duration
      if (duration && !isNaN(duration) && isFinite(duration) && duration > 0) {
        playerLogger.debug(`Duration updated to: ${duration}s`)
        if (onDurationChange)
          onDurationChange(duration)
      }
    }
  })

  mediaElement.addEventListener('error', e => {
    playerLogger.error('Streaming audio error:', e)
    isPlaying = false
  })
}

const setEQEnabled = (enabled: boolean): boolean => {
  try {
    eqEnabled = enabled

    const playerStore = usePlayerStore()
    if (playerStore.eqEnabled !== enabled)
      playerStore.setEQEnabled(enabled)

    if (enabled) {
      if (!eqNodes && !initializeEQ())
        return false

      loadStoredEQBands()

      if (!updateAudioGraph())
        return false
    } else if (!updateAudioGraph()){
      playerLogger.error('Failed to update audio graph when disabling EQ')
    }

    return true
  } catch (error) {
    playerLogger.error('Failed to toggle EQ:', error)
    return false
  }
}

const getEQEnabled = (): boolean => eqEnabled

const setEQBandGain = (bandIndex: number, gain: number): boolean => {
  try {
    if (!eqNodes || bandIndex < 0 || bandIndex >= eqNodes.length) {
      playerLogger.error(`Invalid EQ band index: ${bandIndex}`)
      return false
    }

    const clampedGain = Math.max(-20, Math.min(20, gain)) // Clamp to -20dB to +20dB
    eqNodes[bandIndex].gain.value = clampedGain

    const playerStore = usePlayerStore()
    playerStore.setEQBandGain(bandIndex, clampedGain)

    return true
  } catch (error) {
    playerLogger.error(`Failed to set EQ band ${bandIndex} gain:`, error)
    return false
  }
}

const getEQBandGain = (bandIndex: number): number => {
  if (!eqNodes || bandIndex < 0 || bandIndex >= eqNodes.length)
    return 0

  return eqNodes[bandIndex].gain.value
}

const getEQBands = (): EQBand[] =>
  DEFAULT_EQ_BANDS.map((band, index) => ({
    ...band,
    gain: getEQBandGain(index),
  }))

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

const getEQPresets = (): EQPreset[] => EQ_PRESETS

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

const setOnDurationChange = (callback: (duration: number) => void): void => {
  onDurationChange = callback
}

const cleanup = (): void => {
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

    if (audioContext && audioContext.state !== 'closed')
      audioContext.close()

    audioContext = null
    pendingVolume = 1.0
    eqEnabled = false
    onDurationChange = null
    playerLogger.debug('WebAudio streaming resources cleaned up')
  } catch (error) {
    playerLogger.error('Failed to cleanup WebAudio streaming resources:', error)
  }
}

export const useWebAudioPlayer = (): WebAudioPlayer => ({
  applyEQPreset,
  cleanup,
  getCurrentTime,

  getDuration,
  getEQBandGain,
  getEQBands,
  getEQEnabled,
  getEQPresets,
  getIsPlaying,

  getIsReady,
  initializeWebAudio,
  isWebAudioAvailable,
  loadAudio,

  loadStoredEQBands,
  pause,
  play,
  resetEQ,
  seek: seek as (time: number) => Promise<boolean>,
  setEQBandGain,
  setEQEnabled,
  setOnDurationChange,

  setVolume,

  stop,
})

