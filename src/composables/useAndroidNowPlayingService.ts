import { watchThrottled } from '@vueuse/core'
import { storeToRefs } from 'pinia'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import type { Song } from '@/bindings'
import { AndroidNowPlayingService, type AndroidNowPlayingPayload } from '@/lib/androidForegroundService'
import { logger } from '@/lib/logger'
import { getPlatform, Platform } from '@/lib/platform'
import { commands } from '@/bindings'
import { useAuthStore, usePlayerStore } from '@/stores'

const roundTo = (value: number, precision: number): number => {
  const factor = 10 ** precision
  return Math.round(value * factor) / factor
}

const sanitizeNumber = (value: unknown): number | null => {
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
      if (lastSignature !== '') {
        lastSignature = ''
        void AndroidNowPlayingService.clear()
      }
      return
    }

    const signature = JSON.stringify(payload)
    if (signature === lastSignature) return
    lastSignature = signature

    void AndroidNowPlayingService.update(payload)
  }

  const resolveArtworkPath = async (song: Song): Promise<void> => {
    if (!authStore.isAuthenticated()) {
      artworkPath.value = null
      return
    }

    try {
      const imageId = song.albumId ?? song.id
      const result = await commands.getImage(imageId, 'Primary', authStore.serverUrl.value, authStore.token.value)
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

  unwatchers.push(watch(currentSong, async song => {
    if (!song) {
      artworkPath.value = null
      lastSignature = ''
      void AndroidNowPlayingService.clear()
      return
    }

    lastSignature = ''
    await resolveArtworkPath(song)
    sendUpdate()
  }, { immediate: true }))

  unwatchers.push(watch(baseMetadata, metadata => {
    if (!metadata) return
    sendUpdate()
  }))

  unwatchers.push(watchThrottled(() => currentTime.value, position => {
    sendUpdate(position)
  }, { throttle: 1000, trailing: true, leading: true }))

  onBeforeUnmount(() => {
    unwatchers.forEach(unwatch => unwatch())
    void AndroidNowPlayingService.clear()
  })

  return { isSupported: true }
}
