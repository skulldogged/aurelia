import { onBeforeUnmount, ref, type Ref, watch } from 'vue'

import type { RpcActivity } from '../generated'
import type { Song } from '../lib/api/types'

import { getApiClient } from '../index'
import { logger } from '../lib/logger'
import { isDesktop } from '../lib/platform'
import { usePlayerStore } from '../stores'

const DISCORD_APP_ID =
  import.meta.env.VITE_DISCORD_APP_ID
  || '1422099270340837419'
const hasTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const activitySignature = (song: null | Song, isPlaying: boolean): string => {
  if (!song)
    return 'idle'

  const artists = song.artists?.join(', ') ?? 'Unknown Artist'

  const segments = [
    song.id,
    song.name,
    artists,
    isPlaying ? 'playing' : 'paused',
  ]

  return segments.join('|')
}

export const useDiscordPresence = (): {
  isEnabled: Ref<boolean>
  refresh:   () => Promise<void>
  stop:      () => Promise<void>
} => {
  const playerStore = usePlayerStore()

  const isEnabled = ref(hasTauri && isDesktop() && DISCORD_APP_ID.length > 0)

  if (!isEnabled.value) {
    const noop = async (): Promise<void> => {}
    const reason = !hasTauri
      ? 'Tauri runtime not detected'
      : !isDesktop()
        ? 'Not running on desktop platform'
        : 'Discord application ID not configured'
    logger.info('Discord Rich Presence disabled', { reason })

    return {
      isEnabled,
      refresh: noop,
      stop:    noop,
    }
  }

  logger.info(`Discord Rich Presence enabled with app ID: ${DISCORD_APP_ID}`)

  let hasStartedThread = false
  let lastSignature = ''
  let isUpdating = false
  let pendingUpdate = false
  let lastSuccessfulUpdate = Date.now()
  let currentSongStartTime: null | number = null

  const sleep = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms))

  const ensureThread = async (): Promise<boolean> => {
    if (hasStartedThread) {
      const timeSinceLastSuccess = Date.now() - lastSuccessfulUpdate
      if (timeSinceLastSuccess > 60000) {
        const seconds = Math.round(timeSinceLastSuccess / 1000)
        logger.warn(`No successful updates in ${seconds}s, attempting reconnection`)
        hasStartedThread = false
      } else {
        return true
      }
    }

    try {
      const result = await getApiClient().discordRpcIsRunning()
      if (result.status === 'error') {
        logger.error('Failed to check Discord RPC status:', result.error)
        return false
      }
      const running = result.data
      logger.debug('Discord RPC thread status before start:', { running })

      if (!running) {
        logger.info('Starting Discord RPC thread with app ID:', DISCORD_APP_ID)
        const startResult = await getApiClient().discordRpcStart(DISCORD_APP_ID)
        if (startResult.status === 'error') {
          logger.error('Failed to start Discord RPC:', startResult.error)
          return false
        }
        logger.info('Discord RPC thread started successfully')
        await sleep(500)
      } else {
        logger.debug('Discord RPC thread already running')
      }

      hasStartedThread = true
      return true
    } catch (error) {
      logger.error('Failed to start Discord RPC thread', error)
      hasStartedThread = false
      return false
    }
  }

  const stopThread = async (): Promise<void> => {
    const clearResult = await getApiClient().discordRpcClearActivity()
    if (clearResult.status === 'error') {
      logger.debug('Failed to clear Discord activity on shutdown', clearResult.error)
    }

    const stopResult = await getApiClient().discordRpcStop()
    if (stopResult.status === 'error') {
      logger.debug('Failed to stop Discord RPC thread', stopResult.error)
    } else {
      logger.debug('Stopped Discord RPC thread')
    }

    hasStartedThread = false
    lastSignature = ''
  }

  const pushActivity = async (): Promise<void> => {
    const song = playerStore.currentSong
    const isPlaying = playerStore.isPlaying
    const position = playerStore.currentTime

    const signature = activitySignature(song, isPlaying)
    if (signature === lastSignature)
      return

    lastSignature = signature

    if (!song) {
      if (!await ensureThread())
        return

      const clearResult = await getApiClient().discordRpcClearActivity()
      if (clearResult.status === 'error') {
        logger.error('Failed to clear Discord activity', clearResult.error)
        hasStartedThread = false
        return
      }
      lastSuccessfulUpdate = Date.now()
      logger.debug('Cleared Discord activity (no active song)')
      return
    }

    if (!await ensureThread())
      return

    const artists = song.artists?.join(', ') ?? 'Unknown Artist'
    const duration = song.duration ?? 0

    let artistImageUrl: null | string = null
    if (song.artistIds?.length && song.albumArtUrl)
      artistImageUrl = `${song.albumArtUrl.split('/Items/')[0]}/Items/${song.artistIds[0]}/Images/Primary`

    const activity: RpcActivity = {
      details:        song.name,
      endTimestamp:   null,
      largeImageKey:  song.albumArtUrl ?? null,
      largeImageText: song.album ?? 'Unknown Album',
      smallImageKey:  null,
      smallImageText: null,
      startTimestamp: null,
      state:          null,
    }

    if (artistImageUrl) {
      activity.smallImageKey = artistImageUrl
      activity.smallImageText = artists
    }

    if (isPlaying) {
      activity.state = artists

      const songStartTime = currentSongStartTime ?? (Date.now() - (position * 1000))
      const startAt = Math.max(0, songStartTime)

      activity.startTimestamp = Math.floor(startAt / 1000)

      if (duration > 0) {
        const endAt = startAt + (duration * 1000)
        activity.endTimestamp = Math.floor(endAt / 1000)
      }
    } else {
      activity.state = artists

      if (duration > 0) {
        const now = Date.now()
        const positionMs = Math.round(position * 1000)
        const startAt = Math.max(0, now - positionMs)

        activity.startTimestamp = Math.floor(startAt / 1000)
        activity.endTimestamp = null
      }
    }

    logger.debug('Updated Discord activity', {
      artists,
      durationSeconds: duration ? Math.round(duration) : null,
      positionSeconds: Math.round(position),
      state:           isPlaying ? 'playing' : 'paused',
      title:           song.name,
    })

    logger.debug('Activity payload details', {
      details:       song.name,
      hasLargeImage: !!song.albumArtUrl,
      hasSmallImage: !!artistImageUrl,
      largeImageUrl: song.albumArtUrl?.substring(0, 100),
      largeText:     song.album ?? 'Unknown Album',
      smallImageUrl: artistImageUrl?.substring(0, 100),
      state:         artists,
    })

    const activityResult = await getApiClient().discordRpcSetActivity(activity)
    if (activityResult.status === 'error') {
      logger.error('Failed to set Discord activity', activityResult.error)
      hasStartedThread = false
      return
    }
    lastSuccessfulUpdate = Date.now()
    logger.debug('Discord activity set successfully')
  }

  const updatePresence = async (): Promise<void> => {
    if (isUpdating) {
      pendingUpdate = true
      return
    }

    isUpdating = true
    try {
      do {
        pendingUpdate = false
        await pushActivity()
      } while (pendingUpdate)
    } catch (error) {
      logger.error('Failed to update Discord activity', error)
    } finally {
      isUpdating = false
    }
  }

  const stopWatchers: Array<() => void> = [
    watch(() => playerStore.currentSong?.id, (newId, oldId) => {
      if (newId === oldId) return
      currentSongStartTime = null
      if (playerStore.isPlaying) {
        lastSignature = ''
        void updatePresence()
      }
    }, { immediate: true }),
    watch(() => playerStore.isPlaying, isPlaying => {
      if (!playerStore.currentSong)
        return

      if (isPlaying) {
        currentSongStartTime = Date.now() - (playerStore.currentTime * 1000)
      }
      lastSignature = ''
      void updatePresence()
    }, { immediate: true }),
    watch(() => playerStore.isSeeking, isSeeking => {
      if (isSeeking && playerStore.currentSong && playerStore.isPlaying) {
        currentSongStartTime = Date.now() - (playerStore.currentTime * 1000)
        logger.debug('Detected seeking via isSeeking flag, updated song start time', {
          newStartTime: currentSongStartTime,
          position:     playerStore.currentTime,
        })
        lastSignature = ''
        void updatePresence()
      }
    }),
  ]

  onBeforeUnmount(async () => {
    stopWatchers.forEach(stopWatcher => stopWatcher())
    await stopThread()
  })

  return {
    isEnabled,
    refresh: updatePresence,
    stop:    stopThread,
  }
}
