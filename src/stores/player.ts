import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Song } from '@/bindings'
import { playerLogger } from '@/lib/logger'

export interface EQBand {
  frequency: number
  gain:      number
  type:      BiquadFilterType
  Q:         number
}

export interface PlayerState {
  isPlaying:   boolean
  currentTime: number
  duration:    number
  isShuffled:  boolean
  repeatMode:  'none' | 'all' | 'one'
  hasPrevious: boolean
  hasNext:     boolean
}

const STORAGE_KEYS = {
  VOLUME:       'player-volume',
  MUTED_VOLUME: 'player-muted-volume',
  IS_MUTED:     'player-muted',
  REPEAT_MODE:  'player-repeat-mode',
  IS_SHUFFLED:  'player-shuffled',
  EQ_ENABLED:   'player-eq-enabled',
  EQ_BANDS:     'player-eq-bands',
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
  { frequency: 60, gain: 0, type: 'lowshelf', Q: 0.707 },
  { frequency: 250, gain: 0, type: 'peaking', Q: 1.414 },
  { frequency: 1000, gain: 0, type: 'peaking', Q: 1.414 },
  { frequency: 4000, gain: 0, type: 'peaking', Q: 1.414 },
  { frequency: 16000, gain: 0, type: 'highshelf', Q: 0.707 },
]

const getStoredEQBands = (): EQBand[] => {
  try {
    const stored = localStorage.getItem(STORAGE_KEYS.EQ_BANDS)
    if (stored) {
      const parsed = JSON.parse(stored)
      if (Array.isArray(parsed) && parsed.length === 5) {
        return parsed as EQBand[]
      }
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
  const repeatMode = ref<'none' | 'one' | 'all'>(getStoredValue(STORAGE_KEYS.REPEAT_MODE, 'none'))

  const eqEnabled = ref(getStoredValue(STORAGE_KEYS.EQ_ENABLED, false))
  const eqBands = ref(getStoredEQBands())
  const hasPrevious = ref(false)
  const hasNext = ref(false)

  const currentSong = ref<Song | null>(null)
  const playlist = ref<Song[]>([])
  const currentIndex = ref(-1)
  const audioReady = ref(false)
  const isBuffering = ref(false)

  const progress = computed(() => duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0)

  const formattedCurrentTime = computed(() => formatTime(currentTime.value))

  const formattedDuration = computed(() => formatTime(duration.value))

  const play = () => {
    isPlaying.value = true
  }

  const pause = () => {
    isPlaying.value = false
  }

  const togglePlay = () => {
    isPlaying.value = !isPlaying.value
  }

  const setCurrentTime = (time: number) => {
    currentTime.value = time
  }

  const setDuration = (time: number) => {
    duration.value = time
  }

  const setVolume = (vol: number) => {
    const clampedVolume = Math.max(0, Math.min(1, vol))
    volume.value = clampedVolume
    setStoredValue(STORAGE_KEYS.VOLUME, clampedVolume)
  }

  const toggleMute = () => {
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

  const toggleShuffle = () => {
    isShuffled.value = !isShuffled.value
    setStoredValue(STORAGE_KEYS.IS_SHUFFLED, isShuffled.value)
  }

  const setRepeatMode = (mode: 'none' | 'all' | 'one') => {
    repeatMode.value = mode
    setStoredValue(STORAGE_KEYS.REPEAT_MODE, mode)
  }

  const cycleRepeatMode = () => {
    const modes: Array<'none' | 'one' | 'all'> = ['none', 'one', 'all']
    const currentModeIndex = modes.indexOf(repeatMode.value)
    repeatMode.value = modes[(currentModeIndex + 1) % modes.length]
    setStoredValue(STORAGE_KEYS.REPEAT_MODE, repeatMode.value)
  }

  const setEQEnabled = (enabled: boolean) => {
    eqEnabled.value = enabled
    setStoredValue(STORAGE_KEYS.EQ_ENABLED, enabled)
  }

  const setEQBands = (bands: EQBand[]) => {
    eqBands.value = bands
    localStorage.setItem(STORAGE_KEYS.EQ_BANDS, JSON.stringify(bands))
  }

  const setEQBandGain = (bandIndex: number, gain: number) => {
    if (bandIndex >= 0 && bandIndex < eqBands.value.length) {
      eqBands.value[bandIndex].gain = gain
      setEQBands(eqBands.value)
    }
  }

  const resetEQ = () => {
    eqBands.value = [...DEFAULT_EQ_BANDS]
    setEQBands(eqBands.value)
  }

  const setCurrentSong = (song: Song | null) => {
    currentSong.value = song
    // Reset time values when song changes to prevent stale seekbar data
    if (song) {
      currentTime.value = 0
      duration.value = song.duration || 0
    } else {
      currentTime.value = 0
      duration.value = 0
    }
  }

  const setPlaylist = (songs: Song[]) => {
    playlist.value = songs
  }

  const setCurrentIndex = (index: number) => {
    currentIndex.value = index
    if (index >= 0 && index < playlist.value.length) {
      currentSong.value = playlist.value[index]
    } else {
      currentSong.value = null
    }
  }

  const setAudioReady = (ready: boolean) => {
    audioReady.value = ready
  }

  const setBuffering = (buffering: boolean) => {
    isBuffering.value = buffering
  }

  const nextSong = () => {
    if (playlist.value.length === 0) return

    let nextIndex: number
    if (isShuffled.value) {
      // Random next song (excluding current)
      const availableIndices = playlist.value
        .map((_, i) => i)
        .filter(i => i !== currentIndex.value)
      nextIndex = availableIndices[Math.floor(Math.random() * availableIndices.length)] ?? 0
    } else {
      nextIndex = (currentIndex.value + 1) % playlist.value.length
    }

    setCurrentIndex(nextIndex)
  }

  const previousSong = () => {
    if (playlist.value.length === 0) return

    let prevIndex: number
    if (isShuffled.value) {
      // Random previous song (excluding current)
      const availableIndices = playlist.value
        .map((_, i) => i)
        .filter(i => i !== currentIndex.value)
      prevIndex = availableIndices[Math.floor(Math.random() * availableIndices.length)] ?? 0
    } else {
      prevIndex = currentIndex.value <= 0 ? playlist.value.length - 1 : currentIndex.value - 1
    }

    setCurrentIndex(prevIndex)
  }

  const playSongAtIndex = (index: number) => {
    if (index >= 0 && index < playlist.value.length) {
      setCurrentIndex(index)
      play()
    }
  }

  const setHasPrevious = (value: boolean) => {
    hasPrevious.value = value
  }

  const setHasNext = (value: boolean) => {
    hasNext.value = value
  }

  const reset = () => {
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

  return {
    isPlaying,
    currentTime,
    duration,
    volume,
    mutedVolume,
    isMuted,
    isShuffled,
    repeatMode,
    hasPrevious,
    hasNext,
    currentSong,
    playlist,
    currentIndex,
    audioReady,
    isBuffering,
    eqEnabled,
    eqBands,
    progress,
    formattedCurrentTime,
    formattedDuration,
    play,
    pause,
    togglePlay,
    setCurrentTime,
    setDuration,
    setVolume,
    toggleMute,
    toggleShuffle,
    setRepeatMode,
    cycleRepeatMode,
    setHasPrevious,
    setHasNext,
    reset,
    setCurrentSong,
    setPlaylist,
    setCurrentIndex,
    setAudioReady,
    setBuffering,
    nextSong,
    previousSong,
    playSongAtIndex,
    setEQEnabled,
    setEQBands,
    setEQBandGain,
    resetEQ,
  }
})