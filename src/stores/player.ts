import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { Song } from '@/bindings'

import { playerLogger } from '@/lib/logger'

export interface EQBand {
  frequency: number
  gain:      number
  Q:         number
  type:      BiquadFilterType
}

export interface PlayerState {
  currentTime: number
  duration:    number
  hasNext:     boolean
  hasPrevious: boolean
  isPlaying:   boolean
  isShuffled:  boolean
  repeatMode:  RepeatMode
}

export type RepeatMode = 'all' | 'none' | 'one'

const STORAGE_KEYS = {
  EQ_BANDS:           'player-eq-bands',
  EQ_ENABLED:         'player-eq-enabled',
  IS_MUTED:           'player-muted',
  IS_SHUFFLED:        'player-shuffled',
  MUTED_VOLUME:       'player-muted-volume',
  REPEAT_MODE:        'player-repeat-mode',
  VISUALIZER_ENABLED: 'player-visualizer-enabled',
  VISUALIZER_STYLE:   'player-visualizer-style',
  VOLUME:             'player-volume',
}

const getStoredValue = <T>(key: string, defaultValue: T): T => {
  try {
    const stored = localStorage.getItem(key)
    if (stored === null) {
      playerLogger.debug(`No stored value for ${key}, using default:`, defaultValue)
      return defaultValue
    }

    if (typeof defaultValue === 'boolean') {
      const result = (stored === 'true') as T
      playerLogger.debug(`Loaded ${key} from localStorage:`, result)
      return result
    }

    if (typeof defaultValue === 'number') {
      const parsed = parseFloat(stored)
      const result = isNaN(parsed) ? defaultValue : parsed as T
      playerLogger.debug(`Loaded ${key} from localStorage:`, result)
      return result
    }

    if (typeof defaultValue === 'string') {
      playerLogger.debug(`Loaded ${key} from localStorage:`, stored)
      return stored as T
    }

    return defaultValue
  } catch (error) {
    playerLogger.warn(`Failed to load ${key} from localStorage:`, error)
    return defaultValue
  }
}

const setStoredValue = <T>(key: string, value: T): void => {
  try {
    playerLogger.debug(`Saving ${key} to localStorage:`, value)
    localStorage.setItem(key, String(value))
  } catch (error) {
    playerLogger.warn(`Failed to save ${key} to localStorage:`, error)
  }
}

const DEFAULT_EQ_BANDS: EQBand[] = [
  { frequency: 60, gain: 0, Q: 0.707, type: 'lowshelf' },
  { frequency: 250, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 1000, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 4000, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 16000, gain: 0, Q: 0.707, type: 'highshelf' },
]

const getStoredEQBands = (): EQBand[] => {
  try {
    const stored = localStorage.getItem(STORAGE_KEYS.EQ_BANDS)
    if (stored) {
      const parsed = JSON.parse(stored)
      if (Array.isArray(parsed) && parsed.length === 5)
        return parsed as EQBand[]
    }
  } catch (error) {
    playerLogger.warn('Failed to load EQ bands from localStorage:', error)
  }

  return DEFAULT_EQ_BANDS
}

export const usePlayerStore = defineStore('player', () => {
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)
  const volume = ref(getStoredValue(STORAGE_KEYS.VOLUME, 1))
  const mutedVolume = ref(getStoredValue(STORAGE_KEYS.MUTED_VOLUME, 0.5)) // Store volume before muting
  const isMuted = ref(getStoredValue(STORAGE_KEYS.IS_MUTED, false))
  const isShuffled = ref(getStoredValue(STORAGE_KEYS.IS_SHUFFLED, false))
  const repeatMode = ref<'all' | 'none' | 'one'>(getStoredValue(STORAGE_KEYS.REPEAT_MODE, 'none'))

  const eqEnabled = ref(getStoredValue(STORAGE_KEYS.EQ_ENABLED, false))
  const eqBands = ref(getStoredEQBands())
  const hasPrevious = ref(false)
  const hasNext = ref(false)
  const hasLyrics = ref(false)

  // Visualizer settings
  const visualizerEnabled = ref(getStoredValue(STORAGE_KEYS.VISUALIZER_ENABLED, true))
  const visualizerStyle = ref<'bars' | 'bars-mirror' | 'curve' | 'wave'>(
    getStoredValue(STORAGE_KEYS.VISUALIZER_STYLE, 'bars-mirror') as 'bars' | 'bars-mirror' | 'curve' | 'wave',
  )

  const currentSong = ref<null | Song>(null)
  const playlist = ref<Song[]>([])
  const currentIndex = ref(-1)
  const audioReady = ref(false)
  const isBuffering = ref(false)

  const progress = computed(() => duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0)

  const formattedCurrentTime = computed(() => formatTime(currentTime.value))

  const formattedDuration = computed(() => formatTime(duration.value))

  const play = (): void => {
    isPlaying.value = true
  }

  const pause = (): void => {
    isPlaying.value = false
  }

  const togglePlay = (): void => {
    isPlaying.value = !isPlaying.value
  }

  const setCurrentTime = (time: number): void => {
    currentTime.value = time
  }

  const setDuration = (time: number): void => {
    duration.value = time
  }

  const setVolume = (vol: number): void => {
    const clampedVolume = Math.max(0, Math.min(1, vol))
    volume.value = clampedVolume
    setStoredValue(STORAGE_KEYS.VOLUME, clampedVolume)
  }

  const toggleMute = (): void => {
    if (isMuted.value) {
      // Unmuting - restore the previous volume
      volume.value = mutedVolume.value
      setStoredValue(STORAGE_KEYS.VOLUME, volume.value)
    } else {
      // Muting - store current volume and mute
      mutedVolume.value = volume.value
      setStoredValue(STORAGE_KEYS.MUTED_VOLUME, mutedVolume.value)
      volume.value = 0
      setStoredValue(STORAGE_KEYS.VOLUME, 0)
    }
    isMuted.value = !isMuted.value
    setStoredValue(STORAGE_KEYS.IS_MUTED, isMuted.value)
  }

  const toggleShuffle = (): void => {
    isShuffled.value = !isShuffled.value
    setStoredValue(STORAGE_KEYS.IS_SHUFFLED, isShuffled.value)
  }

  const setRepeatMode = (mode: 'all' | 'none' | 'one'): void => {
    repeatMode.value = mode
    setStoredValue(STORAGE_KEYS.REPEAT_MODE, mode)
  }

  const cycleRepeatMode = (): void => {
    const modes: Array<'all' | 'none' | 'one'> = ['none', 'one', 'all']
    const currentModeIndex = modes.indexOf(repeatMode.value)
    repeatMode.value = modes[(currentModeIndex + 1) % modes.length]
    setStoredValue(STORAGE_KEYS.REPEAT_MODE, repeatMode.value)
  }

  const setEQEnabled = (enabled: boolean): void => {
    eqEnabled.value = enabled
    setStoredValue(STORAGE_KEYS.EQ_ENABLED, enabled)
  }

  const setEQBands = (bands: EQBand[]): void => {
    eqBands.value = bands
    localStorage.setItem(STORAGE_KEYS.EQ_BANDS, JSON.stringify(bands))
  }

  const setEQBandGain = (bandIndex: number, gain: number): void => {
    if (bandIndex >= 0 && bandIndex < eqBands.value.length) {
      eqBands.value[bandIndex].gain = gain
      setEQBands(eqBands.value)
    }
  }

  const resetEQ = (): void => {
    eqBands.value = [...DEFAULT_EQ_BANDS]
    setEQBands(eqBands.value)
  }

  const setCurrentSong = (song: null | Song): void => {
    currentSong.value = song
    // Reset time values when song changes to prevent stale seekbar data
    if (song) {
      currentTime.value = 0
      duration.value = song.duration || 0
      // Reset lyrics availability when song changes
      hasLyrics.value = false
    } else {
      currentTime.value = 0
      duration.value = 0
      hasLyrics.value = false
    }
  }

  const setHasLyrics = (value: boolean): void => {
    hasLyrics.value = value
  }

  const setPlaylist = (songs: Song[]): void => {
    playlist.value = songs
  }

  const setCurrentIndex = (index: number): void => {
    currentIndex.value = index
    if (index >= 0 && index < playlist.value.length) {
      const newSong = playlist.value[index]
      currentSong.value = newSong
      // Also update duration when song changes via index
      if (newSong) {
        currentTime.value = 0
        duration.value = newSong.duration || 0
      }
    } else {
      currentSong.value = null
      currentTime.value = 0
      duration.value = 0
    }
  }

  const setAudioReady = (ready: boolean): void => {
    audioReady.value = ready
  }

  const setBuffering = (buffering: boolean): void => {
    isBuffering.value = buffering
  }

  const nextSong = (): void => {
    if (playlist.value.length === 0) return

    let nextIndex: number
    if (isShuffled.value) {
      const availableIndices = playlist.value
        .map((_, i) => i)
        .filter(i => i !== currentIndex.value)
      nextIndex = availableIndices[Math.floor(Math.random() * availableIndices.length)] ?? 0
    } else {
      nextIndex = (currentIndex.value + 1) % playlist.value.length
    }

    setCurrentIndex(nextIndex)
  }

  const previousSong = (): void => {
    if (playlist.value.length === 0) return

    let prevIndex: number
    if (isShuffled.value) {
      const availableIndices = playlist.value
        .map((_, i) => i)
        .filter(i => i !== currentIndex.value)
      prevIndex = availableIndices[Math.floor(Math.random() * availableIndices.length)] ?? 0
    } else {
      prevIndex = currentIndex.value <= 0 ? playlist.value.length - 1 : currentIndex.value - 1
    }

    setCurrentIndex(prevIndex)
  }

  const playSongAtIndex = (index: number): void => {
    if (index >= 0 && index < playlist.value.length) {
      setCurrentIndex(index)
      play()
    }
  }

  const setHasPrevious = (value: boolean): void => {
    hasPrevious.value = value
  }

  const setHasNext = (value: boolean): void => {
    hasNext.value = value
  }

  const reset = (): void => {
    isPlaying.value = false
    currentTime.value = 0
    duration.value = 0
    isShuffled.value = false
    repeatMode.value = 'none'
    hasPrevious.value = false
    hasNext.value = false
  }

  const formatTime = (seconds: number): string =>
    `${Math.floor(seconds / 60)}:${(Math.floor(seconds % 60)).toString().padStart(2, '0')}`

  const setVisualizerEnabled = (enabled: boolean): void => {
    visualizerEnabled.value = enabled
    setStoredValue(STORAGE_KEYS.VISUALIZER_ENABLED, enabled)
  }

  const setVisualizerStyle = (style: 'bars' | 'bars-mirror' | 'curve' | 'wave'): void => {
    visualizerStyle.value = style
    setStoredValue(STORAGE_KEYS.VISUALIZER_STYLE, style)
  }

  return {
    audioReady,
    currentIndex,
    currentSong,
    currentTime,
    cycleRepeatMode,
    duration,
    eqBands,
    eqEnabled,
    formattedCurrentTime,
    formattedDuration,
    hasLyrics,
    hasNext,
    hasPrevious,
    isBuffering,
    isMuted,
    isPlaying,
    isShuffled,
    mutedVolume,
    nextSong,
    pause,
    play,
    playlist,
    playSongAtIndex,
    previousSong,
    progress,
    repeatMode,
    reset,
    resetEQ,
    setAudioReady,
    setBuffering,
    setCurrentIndex,
    setCurrentSong,
    setCurrentTime,
    setDuration,
    setEQBandGain,
    setEQBands,
    setEQEnabled,
    setHasLyrics,
    setHasNext,
    setHasPrevious,
    setPlaylist,
    setRepeatMode,
    setVisualizerEnabled,
    setVisualizerStyle,
    setVolume,
    toggleMute,
    togglePlay,
    toggleShuffle,
    visualizerEnabled,
    visualizerStyle,
    volume,
  }
})