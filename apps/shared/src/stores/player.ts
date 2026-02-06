import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { Song } from '../lib/api/types'

import { setIn } from '../lib/immutable'
import { logger } from '../lib/logger'
import { fromNullable, map, unwrapOr } from '../lib/option'

export interface EQBand {
  frequency: number
  gain:      number
  Q:         number
  type:      BiquadFilterType
}

export interface PlayerState {
  currentSong: null | Song
  currentTime: number
  duration:    number
  hasNext:     boolean
  hasPrevious: boolean
  isMuted:     boolean
  isPlaying:   boolean
  isShuffled:  boolean
  playlist:    Song[]
  progress:    number
  repeatMode:  RepeatMode
  volume:      number
}

export type RepeatMode = 'all' | 'none' | 'one'

const STORAGE_KEYS = {
  CURRENT_INDEX:      'player-current-index',
  CURRENT_SONG:       'player-current-song',
  EQ_BANDS:           'player-eq-bands',
  EQ_ENABLED:         'player-eq-enabled',
  IS_MUTED:           'player-muted',
  IS_SHUFFLED:        'player-shuffled',
  MUTED_VOLUME:       'player-muted-volume',
  PLAYLIST:           'player-playlist',
  REPEAT_MODE:        'player-repeat-mode',
  VISUALIZER_ENABLED: 'player-visualizer-enabled',
  VISUALIZER_STYLE:   'player-visualizer-style',
  VOLUME:             'player-volume',
}

const getStoredValue = <T>(key: string, defaultValue: T): T => {
  try {
    const stored = localStorage.getItem(key)
    return unwrapOr(
      map(fromNullable(stored), value => {
        if (typeof defaultValue === 'boolean') {
          return (value === 'true') as T
        }
        if (typeof defaultValue === 'number') {
          const parsed = parseFloat(value)
          return isNaN(parsed) ? defaultValue : parsed as T
        }
        if (typeof defaultValue === 'string') {
          return value as T
        }
        return defaultValue
      }),
      defaultValue,
    )
  } catch (error) {
    logger.warn(`Failed to load ${key} from localStorage:`, error)
    return defaultValue
  }
}

const setStoredValue = <T>(key: string, value: T): void => {
  try {
    logger.debug(`Saving ${key} to localStorage:`, value)
    localStorage.setItem(key, String(value))
  } catch (error) {
    logger.warn(`Failed to save ${key} to localStorage:`, error)
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
    logger.warn('Failed to load EQ bands from localStorage:', error)
  }

  return DEFAULT_EQ_BANDS
}

// Session state helpers
const getStoredSession = (): { currentIndex: number; currentSong: null | Song; playlist: Song[] } => {
  try {
    const songStr = localStorage.getItem(STORAGE_KEYS.CURRENT_SONG)
    const playlistStr = localStorage.getItem(STORAGE_KEYS.PLAYLIST)
    const indexStr = localStorage.getItem(STORAGE_KEYS.CURRENT_INDEX)

    const currentSong = songStr ? JSON.parse(songStr) as Song : null
    const playlist = playlistStr ? JSON.parse(playlistStr) as Song[] : []
    const currentIndex = indexStr ? parseInt(indexStr, 10) : -1

    return { currentIndex, currentSong, playlist }
  } catch (error) {
    logger.warn('Failed to load session from localStorage:', error)
    return { currentIndex: -1, currentSong: null, playlist: [] }
  }
}

const saveSessionState = (currentSong: null | Song, playlist: Song[], currentIndex: number): void => {
  try {
    if (currentSong) {
      localStorage.setItem(STORAGE_KEYS.CURRENT_SONG, JSON.stringify(currentSong))
    } else {
      localStorage.removeItem(STORAGE_KEYS.CURRENT_SONG)
    }

    localStorage.setItem(STORAGE_KEYS.CURRENT_INDEX, String(currentIndex))
  } catch (error) {
    logger.warn('Failed to save session to localStorage:', error)
  }
}

// Debounced playlist save to avoid serializing large playlists on every change
let playlistSaveTimeout: null | ReturnType<typeof setTimeout> = null
const PLAYLIST_SAVE_DEBOUNCE_MS = 500

const savePlaylistDebounced = (playlist: Song[]): void => {
  if (playlistSaveTimeout) {
    clearTimeout(playlistSaveTimeout)
  }
  playlistSaveTimeout = setTimeout(() => {
    try {
      if (playlist.length > 0) {
        localStorage.setItem(STORAGE_KEYS.PLAYLIST, JSON.stringify(playlist))
      } else {
        localStorage.removeItem(STORAGE_KEYS.PLAYLIST)
      }
    } catch (error) {
      logger.warn('Failed to save playlist to localStorage:', error)
    }
    playlistSaveTimeout = null
  }, PLAYLIST_SAVE_DEBOUNCE_MS)
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
  const isSeeking = ref(false)

  // Visualizer settings
  const visualizerEnabled = ref(getStoredValue(STORAGE_KEYS.VISUALIZER_ENABLED, true))
  const visualizerStyle = ref<'bars'  | 'curve' | 'wave'>(
    getStoredValue(STORAGE_KEYS.VISUALIZER_STYLE, 'bars') as 'bars' | 'curve' | 'wave',
  )

  // Restore session state from localStorage
  const storedSession = getStoredSession()
  const currentSong = ref<null | Song>(storedSession.currentSong)
  const playlist = ref<Song[]>(storedSession.playlist)
  const currentIndex = ref(storedSession.currentIndex)
  const shuffleOrder = ref<number[]>([])
  const shuffleOrderPosition = ref(-1)
  const audioReady = ref(!!storedSession.currentSong)
  const isBuffering = ref(false)
  const needsReload = ref(false) // Set when audio stream dies and needs to be recreated

  const progress = computed(() => duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0)

  const formattedCurrentTime = computed(() => formatTime(currentTime.value))

  const formattedDuration = computed(() => formatTime(duration.value))

  const shuffleIndicesInPlace = (indices: number[]): void => {
    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[indices[i], indices[j]] = [indices[j], indices[i]]
    }
  }

  const clearShuffleOrder = (): void => {
    if (shuffleOrder.value.length === 0 && shuffleOrderPosition.value === -1) {
      return
    }
    shuffleOrder.value = []
    shuffleOrderPosition.value = -1
  }

  const rebuildShuffleOrder = (): void => {
    if (!isShuffled.value || playlist.value.length === 0 || currentIndex.value < 0 || currentIndex.value >= playlist.value.length) {
      clearShuffleOrder()
      return
    }

    const remaining = playlist.value
      .map((_, index) => index)
      .filter(index => index !== currentIndex.value)
    shuffleIndicesInPlace(remaining)
    shuffleOrder.value = [currentIndex.value, ...remaining]
    shuffleOrderPosition.value = 0
  }

  const syncShufflePositionWithCurrentIndex = (): void => {
    if (!isShuffled.value) {
      clearShuffleOrder()
      return
    }

    if (playlist.value.length === 0 || currentIndex.value < 0 || currentIndex.value >= playlist.value.length) {
      clearShuffleOrder()
      return
    }

    const normalizedOrder: number[] = []
    const seen = new Set<number>()
    for (const index of shuffleOrder.value) {
      if (index >= 0 && index < playlist.value.length && !seen.has(index)) {
        normalizedOrder.push(index)
        seen.add(index)
      }
    }

    const missing = playlist.value
      .map((_, index) => index)
      .filter(index => !seen.has(index))
    shuffleIndicesInPlace(missing)
    const nextOrder = [...normalizedOrder, ...missing]
    const orderChanged = nextOrder.length !== shuffleOrder.value.length
      || nextOrder.some((index, idx) => index !== shuffleOrder.value[idx])
    if (orderChanged) {
      shuffleOrder.value = nextOrder
    }

    const position = shuffleOrder.value.indexOf(currentIndex.value)
    if (position === -1) {
      rebuildShuffleOrder()
      return
    }
    shuffleOrderPosition.value = position
  }

  const getNextSongIndex = (includeRepeat = false): number => {
    if (playlist.value.length === 0 || currentIndex.value < 0 || currentIndex.value >= playlist.value.length) {
      return -1
    }

    if (isShuffled.value) {
      const nextPosition = shuffleOrderPosition.value + 1
      if (nextPosition < shuffleOrder.value.length) {
        return shuffleOrder.value[nextPosition] ?? -1
      }
      return includeRepeat ? (shuffleOrder.value[0] ?? -1) : -1
    }

    const nextIndex = currentIndex.value + 1
    if (nextIndex < playlist.value.length) {
      return nextIndex
    }
    return includeRepeat ? 0 : -1
  }

  const getPreviousSongIndex = (includeRepeat = false): number => {
    if (playlist.value.length === 0 || currentIndex.value < 0 || currentIndex.value >= playlist.value.length) {
      return -1
    }

    if (isShuffled.value) {
      const previousPosition = shuffleOrderPosition.value - 1
      if (previousPosition >= 0) {
        return shuffleOrder.value[previousPosition] ?? -1
      }
      return includeRepeat ? (shuffleOrder.value[shuffleOrder.value.length - 1] ?? -1) : -1
    }

    const previousIndex = currentIndex.value - 1
    if (previousIndex >= 0) {
      return previousIndex
    }
    return includeRepeat ? playlist.value.length - 1 : -1
  }

  const canGoNext = (): boolean => getNextSongIndex(false) !== -1
  const canGoPrevious = (): boolean => getPreviousSongIndex(false) !== -1

  if (isShuffled.value) {
    rebuildShuffleOrder()
  }

  const play = (): void => {
    isPlaying.value = true
  }

  const pause = (): void => {
    isPlaying.value = false
  }

  const togglePlay = (): void => {
    isPlaying.value = !isPlaying.value
  }

  const setCurrentTime = (time: number, isSeek = false): void => {
    currentTime.value = time
    if (isSeek) {
      isSeeking.value = true
    }
  }

  const setIsSeeking = (seeking: boolean): void => {
    isSeeking.value = seeking
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
    if (isShuffled.value) {
      rebuildShuffleOrder()
    } else {
      clearShuffleOrder()
    }
  }

  const setRepeatMode = (mode: 'all' | 'none' | 'one'): void => {
    repeatMode.value = mode
    setStoredValue(STORAGE_KEYS.REPEAT_MODE, mode)
  }

  const cycleRepeatMode = (): void => {
    const modes: Array<'all' | 'none' | 'one'> = ['none', 'all', 'one']
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
      eqBands.value = setIn(eqBands.value, [bandIndex, 'gain'], gain)
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
    // Persist session state (playlist is saved separately with debounce)
    saveSessionState(currentSong.value, playlist.value, currentIndex.value)
  }

  const setHasLyrics = (value: boolean): void => {
    hasLyrics.value = value
  }

  const setPlaylist = (songs: Song[]): void => {
    playlist.value = songs
    if (isShuffled.value) {
      rebuildShuffleOrder()
    } else {
      clearShuffleOrder()
    }
    // Persist session state - playlist saved with debounce to avoid expensive serialization
    saveSessionState(currentSong.value, playlist.value, currentIndex.value)
    savePlaylistDebounced(songs)
  }

  const setCurrentIndex = (index: number): void => {
    const indexChanged = currentIndex.value !== index
    currentIndex.value = index
    if (index >= 0 && index < playlist.value.length) {
      const newSong = playlist.value[index]
      currentSong.value = newSong
      // Only reset time/duration when switching to a different song
      if (indexChanged && newSong) {
        currentTime.value = 0
        duration.value = newSong.duration || 0
      }
    } else {
      currentSong.value = null
      currentTime.value = 0
      duration.value = 0
    }
    if (isShuffled.value) {
      syncShufflePositionWithCurrentIndex()
    }
    // Persist session state (playlist is saved separately with debounce)
    saveSessionState(currentSong.value, playlist.value, currentIndex.value)
  }

  const setAudioReady = (ready: boolean): void => {
    audioReady.value = ready
  }

  const setBuffering = (buffering: boolean): void => {
    isBuffering.value = buffering
  }

  const nextSong = (): void => {
    logger.debug(
      `[Store] nextSong called. playlist.length=${playlist.value.length}, currentIndex=${currentIndex.value}`,
    )
    const nextIndex = getNextSongIndex(repeatMode.value === 'all')
    if (nextIndex === -1) return

    logger.debug(`[Store] nextSong: setting index to ${nextIndex}`)
    setCurrentIndex(nextIndex)
    logger.debug(`[Store] nextSong: currentSong is now ${currentSong.value?.name}`)
  }

  const previousSong = (): void => {
    const prevIndex = getPreviousSongIndex(false)
    if (prevIndex === -1) return
    setCurrentIndex(prevIndex)
  }

  const playSongAtIndex = (index: number): void => {
    if (index >= 0 && index < playlist.value.length) {
      setCurrentIndex(index)
      setCurrentTime(0)
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
    clearShuffleOrder()
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

  const setVisualizerStyle = (style: 'bars' | 'curve' | 'wave'): void => {
    visualizerStyle.value = style
    setStoredValue(STORAGE_KEYS.VISUALIZER_STYLE, style)
  }

  const setNeedsReload = (value: boolean): void => {
    needsReload.value = value
  }

  const updateSongFavorite = (songId: string, isFavorite: boolean): void => {
    // Update current song if it matches
    if (currentSong.value?.id === songId) {
      currentSong.value = { ...currentSong.value, isFavorite }
      saveSessionState(currentSong.value, playlist.value, currentIndex.value)
    }

    // Update song in playlist if present
    const playlistIndex = playlist.value.findIndex(s => s.id === songId)
    if (playlistIndex !== -1) {
      playlist.value = [
        ...playlist.value.slice(0, playlistIndex),
        { ...playlist.value[playlistIndex], isFavorite },
        ...playlist.value.slice(playlistIndex + 1),
      ]
      savePlaylistDebounced(playlist.value)
    }
  }

  return {
    audioReady,
    canGoNext,
    canGoPrevious,
    currentIndex,
    currentSong,
    currentTime,
    cycleRepeatMode,
    duration,
    eqBands,
    eqEnabled,
    formattedCurrentTime,
    formattedDuration,
    getNextSongIndex,
    getPreviousSongIndex,
    hasLyrics,
    hasNext,
    hasPrevious,
    isBuffering,
    isMuted,
    isPlaying,
    isSeeking,
    isShuffled,
    mutedVolume,
    needsReload,
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
    setIsSeeking,
    setNeedsReload,
    setPlaylist,
    setRepeatMode,
    setVisualizerEnabled,
    setVisualizerStyle,
    setVolume,
    toggleMute,
    togglePlay,
    toggleShuffle,
    updateSongFavorite,
    visualizerEnabled,
    visualizerStyle,
    volume,
  }
})
