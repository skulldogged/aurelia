import { invoke } from '@tauri-apps/api/core'
import { ref, type Ref, watch } from 'vue'

import type { ListenBrainzCredentials, Song } from '@/bindings'

import { listenbrainzLogger as logger } from '@/lib/logger'
import { useListenBrainzStore, usePlayerStore } from '@/stores'

const SCROBBLE_THRESHOLD_SECONDS = 240
const SCROBBLE_PERCENTAGE = 0.5

const hasTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export const useListenBrainz = (): {
  clearSession:   () => Promise<void>
  isEnabled:      Ref<boolean>
  setCredentials: (credentials: ListenBrainzCredentials) => Promise<void>
  validateToken:  (userToken: string) => Promise<ListenBrainzCredentials>
} => {
  const listenbrainzStore = useListenBrainzStore()
  const playerStore = usePlayerStore()

  const isEnabled = ref(hasTauri && listenbrainzStore.isAuthenticated())

  if (!hasTauri) {
    const noop = async (): Promise<void> => {}
    const noopValidate = async (): Promise<ListenBrainzCredentials> => ({
      user_token: '',
      username:   null,
    })

    return {
      clearSession:   noop,
      isEnabled,
      setCredentials: noop,
      validateToken:  noopValidate,
    }
  }

  if (listenbrainzStore.credentials) {
    void invoke('listenbrainz_set_credentials', {
      credentials: listenbrainzStore.credentials,
    }).catch(err => {
      logger.error('Failed to restore credentials:', err)
    })
  }

  let hasScrobbled = false
  let trackStartTimestamp = 0

  const shouldScrobble = (song: Song, currentTime: number): boolean => {
    const duration = song.duration ?? 0
    if (duration === 0)
      return false

    const timeThreshold = Math.min(duration * SCROBBLE_PERCENTAGE, SCROBBLE_THRESHOLD_SECONDS)
    return currentTime >= timeThreshold
  }

  const submitListen = async (song: Song): Promise<void> => {
    if (!listenbrainzStore.isEnabled || !listenbrainzStore.isScrobblingEnabled || hasScrobbled)
      return

    hasScrobbled = true

    const artist = song.artists?.[0] ?? 'Unknown Artist'
    const track = song.name
    const album = song.album ?? null
    const duration = song.duration ? Math.floor(song.duration) : null

    const listen = { album, artist, duration, track }
    const timestamp = trackStartTimestamp

    try {
      logger.info('Submitting listen:', { artist, track })
      await invoke('listenbrainz_submit_listen', { listen, timestamp })
    } catch (error) {
      logger.error('Failed to submit listen:', error)
      hasScrobbled = false
    }
  }

  const updatePlayingNow = async (song: Song): Promise<void> => {
    if (!listenbrainzStore.isEnabled)
      return

    const artist = song.artists?.[0] ?? 'Unknown Artist'
    const track = song.name
    const album = song.album ?? null

    try {
      await invoke('listenbrainz_playing_now', { album, artist, track })
    } catch (error) {
      logger.warn('Failed to update playing now:', error)
    }
  }

  watch(
    () => playerStore.currentSong,
    (newSong, oldSong) => {
      if (!newSong || !listenbrainzStore.isEnabled)
        return

      if (oldSong?.id !== newSong.id) {
        hasScrobbled = false
        const currentTime = playerStore.currentTime
        const secondsAgo = Math.floor(currentTime)
        trackStartTimestamp = Math.floor(Date.now() / 1000) - secondsAgo
        void updatePlayingNow(newSong)
      }
    },
    { immediate: true },
  )

  watch(
    () => playerStore.currentTime,
    currentTime => {
      const song = playerStore.currentSong
      if (!song || !playerStore.isPlaying || !listenbrainzStore.isEnabled)
        return

      if (shouldScrobble(song, currentTime) && !hasScrobbled)
        void submitListen(song)
    },
  )

  const validateToken = async (userToken: string): Promise<ListenBrainzCredentials> => {
    try {
      const credentials = await invoke<ListenBrainzCredentials>('listenbrainz_validate_token', {
        userToken,
      })

      listenbrainzStore.setCredentials(credentials)
      listenbrainzStore.setEnabled(true)
      isEnabled.value = true

      logger.info('Successfully validated token')
      return credentials
    } catch (error) {
      logger.error('Failed to validate token:', error)
      throw error
    }
  }

  const setCredentials = async (credentials: ListenBrainzCredentials): Promise<void> => {
    try {
      await invoke('listenbrainz_set_credentials', { credentials })
      listenbrainzStore.setCredentials(credentials)
      listenbrainzStore.setEnabled(true)
      isEnabled.value = true
    } catch (error) {
      logger.error('Failed to set credentials:', error)
      throw error
    }
  }

  const clearSession = async (): Promise<void> => {
    try {
      await invoke('listenbrainz_clear_credentials')
      listenbrainzStore.clearCredentials()
      isEnabled.value = false
      hasScrobbled = false
    } catch (error) {
      logger.error('Failed to clear session:', error)
      throw error
    }
  }

  return {
    clearSession,
    isEnabled,
    setCredentials,
    validateToken,
  }
}
