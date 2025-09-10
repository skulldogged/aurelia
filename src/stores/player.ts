import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Song } from '@/bindings'

export interface PlayerState {
  isPlaying:   boolean
  currentTime: number
  duration:    number
  isShuffled:  boolean
  repeatMode:  'none' | 'all' | 'one'
  hasPrevious: boolean
  hasNext:     boolean
}

// Local storage keys
const STORAGE_KEYS = {
  VOLUME:       'player-volume',
  MUTED_VOLUME: 'player-muted-volume',
  IS_MUTED:     'player-muted',
  REPEAT_MODE:  'player-repeat-mode',
  IS_SHUFFLED:  'player-shuffled',
}

// Helper functions for localStorage
const getStoredValue = <T>(key: string, defaultValue: T): T => {
  try {
    const stored = localStorage.getItem(key)
    if (stored === null) {
      console.log(`[PlayerStore] No stored value for ${key}, using default:`, defaultValue)
      return defaultValue
    }

    // Handle boolean values stored as strings
    if (typeof defaultValue === 'boolean') {
      const result = (stored === 'true') as T
      console.log(`[PlayerStore] Loaded ${key} from localStorage:`, result)
      return result
    }

    // Handle numeric values
    if (typeof defaultValue === 'number') {
      const parsed = parseFloat(stored)
      const result = isNaN(parsed) ? defaultValue : parsed as T
      console.log(`[PlayerStore] Loaded ${key} from localStorage:`, result)
      return result
    }

    // Handle string values
    if (typeof defaultValue === 'string') {
      console.log(`[PlayerStore] Loaded ${key} from localStorage:`, stored)
      return stored as T
    }

    return defaultValue
  } catch (error) {
    console.warn(`Failed to load ${key} from localStorage:`, error)
    return defaultValue
  }
}

const setStoredValue = <T>(key: string, value: T): void => {
  try {
    console.log(`[PlayerStore] Saving ${key} to localStorage:`, value)
    localStorage.setItem(key, String(value))
  } catch (error) {
    console.warn(`Failed to save ${key} to localStorage:`, error)
  }
}

export const usePlayerStore = defineStore('player', () => {
  // State - Initialize with persisted values
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)
  const volume = ref(getStoredValue(STORAGE_KEYS.VOLUME, 1))
  const mutedVolume = ref(getStoredValue(STORAGE_KEYS.MUTED_VOLUME, 0.5)) // Store volume before muting
  const isMuted = ref(getStoredValue(STORAGE_KEYS.IS_MUTED, false))
  const isShuffled = ref(getStoredValue(STORAGE_KEYS.IS_SHUFFLED, false))
  const repeatMode = ref<'none' | 'one' | 'all'>(getStoredValue(STORAGE_KEYS.REPEAT_MODE, 'none'))
  const hasPrevious = ref(false)
  const hasNext = ref(false)

  // New centralized state
  const currentSong = ref<Song | null>(null)
  const playlist = ref<Song[]>([])
  const currentIndex = ref(-1)
  const audioReady = ref(false)
  const isBuffering = ref(false)

  // Getters
  const progress = computed(() => duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0)

  const formattedCurrentTime = computed(() => formatTime(currentTime.value))

  const formattedDuration = computed(() => formatTime(duration.value))

  // Actions
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

  // New actions for centralized state
  const setCurrentSong = (song: Song | null) => {
    currentSong.value = song
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

  // Helper function
  const formatTime = (seconds: number): string =>
    `${Math.floor(seconds / 60)}:${(Math.floor(seconds % 60)).toString().padStart(2, '0')}`

  return {
    // State
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
    // Getters
    progress,
    formattedCurrentTime,
    formattedDuration,
    // Actions
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
  }
})