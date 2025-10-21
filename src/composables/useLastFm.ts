import { ref, type Ref, watch } from 'vue'

import type { LastFmCredentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { logger } from '@/lib/logger'
import { useLastFmStore, usePlayerStore } from '@/stores'

const SCROBBLE_THRESHOLD_SECONDS = 240 // 4 minutes
const SCROBBLE_PERCENTAGE = 0.5 // 50% of track

const hasTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export const useLastFm = (): {
  authenticate:   (apiKey: string, apiSecret: string, token: string) => Promise<LastFmCredentials>
  clearSession:   () => Promise<void>
  isEnabled:      Ref<boolean>
  setCredentials: (credentials: LastFmCredentials) => Promise<void>
} => {
  const lastfmStore = useLastFmStore()
  const playerStore = usePlayerStore()

  const isEnabled = ref(hasTauri && lastfmStore.isAuthenticated())

  if (!hasTauri) {
    logger.info('Last.fm disabled: Tauri runtime not detected')
    const noop = async (): Promise<void> => {}
    const noopAuth = async (): Promise<LastFmCredentials> => ({
      api_key:     '',
      api_secret:  '',
      session_key: null,
      username:    null,
    })

    return {
      authenticate:   noopAuth,
      clearSession:   noop,
      isEnabled,
      setCredentials: noop,
    }
  }

  // Restore credentials to Rust backend on init
  if (lastfmStore.credentials) {
    void commands.lastfmSetCredentials(lastfmStore.credentials).then(result => {
      if (result.status === 'error') {
        logger.error('Failed to restore credentials to backend:', result.error)
      } else {
        logger.debug('Loaded Last.fm credentials from localStorage')
      }
    })
  }

  let hasScrobbled = false
  let trackStartTimestamp = 0 // Unix timestamp when current track started playing

  const shouldScrobble = (song: Song, currentTime: number): boolean => {
    const duration = song.duration ?? 0
    if (duration === 0)
      return false

    // Scrobble when 50% played or after 4 minutes, whichever comes first
    const timeThreshold = Math.min(duration * SCROBBLE_PERCENTAGE, SCROBBLE_THRESHOLD_SECONDS)
    return currentTime >= timeThreshold
  }

  const scrobbleTrack = async (song: Song): Promise<void> => {
    if (!lastfmStore.isEnabled || !lastfmStore.isScrobblingEnabled)
      return

    if (hasScrobbled) {
      logger.debug('Track already scrobbled, skipping')
      return
    }

    // Set flag immediately to prevent duplicate scrobbles
    hasScrobbled = true

    const artist = song.artists?.[0] ?? 'Unknown Artist'
    const track = song.name
    const album = song.album ?? null
    const duration = song.duration ? Math.floor(song.duration) : null

    // Use the timestamp when the track started, not when we're scrobbling
    const timestamp = trackStartTimestamp

    const scrobble = {
      album,
      artist,
      duration,
      timestamp,
      track,
    }

    try {
      logger.info('Scrobbling track:', { artist, timestamp, track })
      const result = await commands.lastfmScrobble(scrobble)
      if (result.status === 'error') {
        logger.error('Failed to scrobble track:', result.error)
        // Reset flag on error so we can retry
        hasScrobbled = false
      } else {
        logger.debug('Successfully scrobbled track')
      }
    } catch (error) {
      logger.error('Failed to scrobble track:', error)
      // Reset flag on error so we can retry
      hasScrobbled = false
    }
  }

  const updateNowPlaying = async (song: Song): Promise<void> => {
    if (!lastfmStore.isEnabled)
      return

    const artist = song.artists?.[0] ?? 'Unknown Artist'
    const track = song.name
    const album = song.album ?? undefined

    const result = await commands.lastfmUpdateNowPlaying(artist, track, album ?? null)
    if (result.status === 'error') {
      logger.warn('Failed to update now playing:', result.error)
    } else {
      logger.debug('Successfully updated now playing')
    }
  }

  // Watch for song changes
  watch(
    () => playerStore.currentSong,
    (newSong, oldSong) => {
      if (!newSong || !lastfmStore.isEnabled)
        return

      // Reset scrobble state when song changes
      if (oldSong?.id !== newSong.id) {
        hasScrobbled = false

        // Calculate when this track actually started playing
        // If currentTime > 0, the track was already playing (e.g., after page reload)
        const currentTime = playerStore.currentTime
        const secondsAgo = Math.floor(currentTime)
        trackStartTimestamp = Math.floor(Date.now() / 1000) - secondsAgo

        // Update now playing
        void updateNowPlaying(newSong)
      }
    },
    { immediate: true },
  )

  // Watch for playback progress to trigger scrobble
  watch(
    () => playerStore.currentTime,
    currentTime => {
      const song = playerStore.currentSong
      if (!song || !playerStore.isPlaying || !lastfmStore.isEnabled)
        return

      // Check if we should scrobble
      if (shouldScrobble(song, currentTime) && !hasScrobbled)
        void scrobbleTrack(song)
    },
  )

  const authenticate = async (
    apiKey: string,
    apiSecret: string,
    token: string,
  ): Promise<LastFmCredentials> => {
    try {
      logger.info('Authenticating with Last.fm')
      const result = await commands.lastfmAuthenticate(apiKey, apiSecret, token)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      const credentials = result.data

      lastfmStore.setCredentials(credentials)
      lastfmStore.setEnabled(true)
      isEnabled.value = true

      logger.info('Successfully authenticated with Last.fm')
      return credentials
    } catch (error) {
      logger.error('Failed to authenticate with Last.fm:', error)
      throw error
    }
  }

  const setCredentials = async (credentials: LastFmCredentials): Promise<void> => {
    try {
      const result = await commands.lastfmSetCredentials(credentials)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      lastfmStore.setCredentials(credentials)
      lastfmStore.setEnabled(true)
      isEnabled.value = true
      logger.info('Last.fm credentials set')
    } catch (error) {
      logger.error('Failed to set Last.fm credentials:', error)
      throw error
    }
  }

  const clearSession = async (): Promise<void> => {
    try {
      const result = await commands.lastfmClearCredentials()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      lastfmStore.clearCredentials()
      isEnabled.value = false
      hasScrobbled = false
      logger.info('Last.fm session cleared')
    } catch (error) {
      logger.error('Failed to clear Last.fm session:', error)
      throw error
    }
  }

  // Initialize credentials on load if they exist
  if (lastfmStore.credentials) {
    void setCredentials(lastfmStore.credentials).catch(error => {
      logger.warn('Failed to initialize Last.fm credentials:', error)
    })
  }

  return {
    authenticate,
    clearSession,
    isEnabled,
    setCredentials,
  }
}
