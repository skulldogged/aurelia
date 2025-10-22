import { watchThrottled } from '@vueuse/core'
import { storeToRefs } from 'pinia'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { commands, type Song } from '@/bindings'
import { usePlayerControls } from '@/composables/usePlayerControls'
import { type AndroidNowPlayingPayload, AndroidNowPlayingService } from '@/lib/androidForegroundService'
import { logger } from '@/lib/logger'
import { getPlatform, Platform } from '@/lib/platform'
import { useAuthStore, usePlayerStore } from '@/stores'

const roundTo = (value: number, precision: number): number => {
  const factor = 10 ** precision
  return Math.round(value * factor) / factor
}

const sanitizeNumber = (value: unknown): null | number => {
  if (typeof value !== 'number') return null
  if (!Number.isFinite(value)) return null
  return value
}

export const useAndroidNowPlayingService = (): { isSupported: boolean } => {
  const isSupported = getPlatform() === Platform.Android
  if (!isSupported)
    return { isSupported: false }

  const authStore = useAuthStore()
  const playerStore = usePlayerStore()

  const { serverUrl, token } = storeToRefs(authStore)

  const {
    handleNextSong,
    handlePreviousSong,
    handleTogglePlayPause,
  } = usePlayerControls()

  const {
    currentIndex,
    currentSong,
    currentTime,
    duration,
    isPlaying,
    isShuffled,
    playlist,
    repeatMode,
  } = storeToRefs(playerStore)

  const artworkPath = ref<null | string>(null)
  let lastSignature = ''
  let pendingSignature: null | string = null
  let suppressUpdates = false

  const hasNext = computed(() =>
    playlist.value.length > 0
    && currentIndex.value >= 0
    && currentIndex.value < playlist.value.length - 1,
  )

  const hasPrevious = computed(() =>
    playlist.value.length > 0
    && currentIndex.value > 0,
  )

  const baseMetadata = computed(() => {
    const song = currentSong.value
    if (!song)
      return null

    const explicitDuration = sanitizeNumber(song.duration)
    const runtimeDuration = sanitizeNumber(duration.value)

    return {
      album:           song.album ?? null,
      artists:         song.artists ?? [],
      artworkPath:     artworkPath.value,
      artworkUrl:      song.albumArtUrl ?? null,
      durationSeconds: explicitDuration ?? runtimeDuration,
      hasNext:         hasNext.value,
      hasPrevious:     hasPrevious.value,
      id:              song.id,
      isPlaying:       isPlaying.value,
      isShuffled:      isShuffled.value,
      repeatMode:      repeatMode.value ?? null,
      title:           song.name,
    }
  })

  const buildPayload = (positionOverride?: number): AndroidNowPlayingPayload | null => {
    const metadata = baseMetadata.value
    if (!metadata)
      return null

    const position = sanitizeNumber(positionOverride ?? currentTime.value) ?? 0
    const roundedPosition = roundTo(position, 2)

    return {
      ...metadata,
      positionSeconds: roundedPosition,
    }
  }

  const sendUpdate = (positionOverride?: number): void => {
    const payload = buildPayload(positionOverride)
    if (!payload) {
      pendingSignature = null
      if (lastSignature !== '') {
        lastSignature = ''
        void AndroidNowPlayingService.clear()
      }
      return
    }

    if (suppressUpdates && !payload.isPlaying) {
      return
    }

    if (payload.isPlaying) {
      suppressUpdates = false
    }

    const signature = JSON.stringify(payload)
    if (signature === lastSignature || signature === pendingSignature) return

    pendingSignature = signature

    void AndroidNowPlayingService.update(payload).then(success => {
      if (success) {
        lastSignature = signature
      }

      if (pendingSignature === signature) {
        pendingSignature = null
      }
    })
  }

  const resolveArtworkPath = async (song: Song): Promise<void> => {
    if (!authStore.isAuthenticated()) {
      artworkPath.value = null
      return
    }

    try {
      const imageId = song.albumId ?? song.id
      const result = await commands.getImage(imageId, 'Primary', serverUrl.value, token.value)
      if (result.status === 'ok')
        artworkPath.value = result.data ?? null
      else
        artworkPath.value = null
    } catch (error) {
      logger.warn('Failed to resolve artwork path for Android service:', error)
      artworkPath.value = null
    }
  }

  const unwatchers: Array<() => void> = []
  const cleanupListeners: Array<() => void> = []

  if (typeof window !== 'undefined') {
    const handleNativeControl = (event: Event): void => {
      const detail = (event as CustomEvent<{ action?: string }>).detail
      const action = detail?.action
      if (!action)
        return

      switch (action) {
        case 'next':
          handleNextSong()
          break
        case 'pause':
          if (isPlaying.value)
            handleTogglePlayPause()
          break
        case 'play':
          if (!isPlaying.value)
            handleTogglePlayPause()
          break
        case 'previous':
          handlePreviousSong()
          break
        case 'stop':
          if (isPlaying.value)
            handleTogglePlayPause()
          suppressUpdates = true
          lastSignature = ''
          pendingSignature = null
          artworkPath.value = null
          void AndroidNowPlayingService.clear()
          break
        case 'toggle':
          handleTogglePlayPause()
          break
        default:
          logger.debug('Received unknown Android control action', {
            action,
          })
      }
    }

    window.addEventListener('android-now-playing-control', handleNativeControl as EventListener)
    cleanupListeners.push(() => window.removeEventListener('android-now-playing-control', handleNativeControl as EventListener))
  }

  unwatchers.push(watch(currentSong, async song => {
    if (!song) {
      artworkPath.value = null
      lastSignature = ''
      pendingSignature = null
      void AndroidNowPlayingService.clear()
      return
    }

    lastSignature = ''
    pendingSignature = null
    await resolveArtworkPath(song)
    sendUpdate()
  }, { immediate: true }))

  unwatchers.push(watch(baseMetadata, metadata => {
    if (!metadata) return
    sendUpdate()
  }))

  unwatchers.push(watchThrottled(() => currentTime.value, position => {
    sendUpdate(position)
  }, { leading: true, throttle: 1000, trailing: true }))

  onBeforeUnmount(() => {
    unwatchers.forEach(unwatch => unwatch())
    cleanupListeners.forEach(cleanup => cleanup())
    lastSignature = ''
    pendingSignature = null
    void AndroidNowPlayingService.clear()
  })

  return { isSupported: true }
}
