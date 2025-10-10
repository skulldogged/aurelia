import { onBeforeUnmount, ref, type Ref, watch } from 'vue'

import type { RpcActivity, Song } from '@/bindings'

import { commands } from '@/bindings'
import { presenceLogger } from '@/lib/logger'
import { usePlayerStore } from '@/stores'

const DISCORD_APP_ID =
  import.meta.env.VITE_DISCORD_APP_ID
  || '1422099270340837419'

const POSITION_UPDATE_THRESHOLD = 5
const hasTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const activitySignature = (song: null | Song, isPlaying: boolean, position: number): string => {
  if (!song)
    return 'idle'

  const duration = song.duration ?? 0
  const artists = song.artists?.join(', ') ?? 'Unknown Artist'

  const segments = [
    song.id,
    song.name,
    artists,
    isPlaying ? 'playing' : 'paused',
    Math.round(position).toString(),
    Math.round(duration).toString(),
  ]

  return segments.join('|')
}

export const useDiscordPresence = (): {
  isEnabled: Ref<boolean>
  refresh:   () => Promise<void>
  stop:      () => Promise<void>
} => {
  const playerStore = usePlayerStore()

  const isEnabled = ref(hasTauri && DISCORD_APP_ID.length > 0)

  if (!isEnabled.value) {
    const noop = async (): Promise<void> => {}
    const reason = !hasTauri
      ? 'Tauri runtime not detected'
      : 'Discord application ID not configured'
    presenceLogger.info('Discord Rich Presence disabled', { reason })

    return {
      isEnabled,
      refresh: noop,
      stop:    noop,
    }
  }

  presenceLogger.info('Discord Rich Presence enabled with app ID: ' + DISCORD_APP_ID)

  let hasStartedThread = false
  let lastSignature = ''
  let lastProgressPosition = -POSITION_UPDATE_THRESHOLD
  let isUpdating = false
  let pendingUpdate = false
  let lastSuccessfulUpdate = Date.now() // Initialize to now to avoid false reconnection on startup
  let currentSongStartTime: null | number = null // Track when the current song started playing

  const sleep = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms))

  const ensureThread = async (): Promise<boolean> => {
    if (hasStartedThread) {
      // Check if it's been a while since last successful update - might need reconnection
      const timeSinceLastSuccess = Date.now() - lastSuccessfulUpdate
      if (timeSinceLastSuccess > 60000) { // 1 minute
        const seconds = Math.round(timeSinceLastSuccess / 1000)
        presenceLogger.warn(`No successful updates in ${seconds}s, attempting reconnection`)
        hasStartedThread = false
      } else {
        return true
      }
    }

    try {
      const result = await commands.discordRpcIsRunning()
      if (result.status === 'error') {
        presenceLogger.error('Failed to check Discord RPC status:', result.error)
        return false
      }
      const running = result.data
      presenceLogger.debug('Discord RPC thread status before start:', { running })

      if (!running) {
        presenceLogger.info('Starting Discord RPC thread with app ID:', DISCORD_APP_ID)
        const startResult = await commands.discordRpcStart(DISCORD_APP_ID)
        if (startResult.status === 'error') {
          presenceLogger.error('Failed to start Discord RPC:', startResult.error)
          return false
        }
        presenceLogger.info('Discord RPC thread started successfully')
        await sleep(500) // Give Discord a moment to establish the connection
      } else {
        presenceLogger.debug('Discord RPC thread already running')
      }

      hasStartedThread = true
      return true
    } catch (error) {
      presenceLogger.error('Failed to start Discord RPC thread', error)
      hasStartedThread = false
      return false
    }
  }

  const stopThread = async (): Promise<void> => {
    const clearResult = await commands.discordRpcClearActivity()
    if (clearResult.status === 'error') {
      presenceLogger.debug('Failed to clear Discord activity on shutdown', clearResult.error)
    }

    const stopResult = await commands.discordRpcStop()
    if (stopResult.status === 'error') {
      presenceLogger.debug('Failed to stop Discord RPC thread', stopResult.error)
    } else {
      presenceLogger.debug('Stopped Discord RPC thread')
    }

    hasStartedThread = false
    lastSignature = ''
    lastProgressPosition = -POSITION_UPDATE_THRESHOLD
  }

  const pushActivity = async (): Promise<void> => {
    const song = playerStore.currentSong
    const isPlaying = playerStore.isPlaying
    const position = playerStore.currentTime

    const signature = activitySignature(song, isPlaying, position)
    if (signature === lastSignature)
      return

    lastSignature = signature

    if (!song) {
      if (!await ensureThread())
        return

      const clearResult = await commands.discordRpcClearActivity()
      if (clearResult.status === 'error') {
        presenceLogger.error('Failed to clear Discord activity', clearResult.error)
        hasStartedThread = false
        return
      }
      lastSuccessfulUpdate = Date.now()
      presenceLogger.debug('Cleared Discord activity (no active song)')
      return
    }

    if (!await ensureThread())
      return

    const artists = song.artists?.join(', ') ?? 'Unknown Artist'
    const duration = song.duration ?? 0

    // Build artist image URL if we have an artist ID
    let artistImageUrl: null | string = null

    if (song.artistIds?.length && song.albumArtUrl)
      artistImageUrl = `${song.albumArtUrl.split('/Items/')[0]}/Items/${song.artistIds[0]}/Images/Primary`

    const activity: RpcActivity = {
      buttons:         null,
      details:         song.name,
      end_timestamp:   null,
      large_image:     song.albumArtUrl ?? null,
      large_text:      song.album ?? 'Unknown Album',
      small_image:     null,
      small_text:      null,
      start_timestamp: null,
      state:           null,
    }

    if (artistImageUrl) {
      activity.small_image = artistImageUrl
      activity.small_text = artists
    }

    if (isPlaying) {
      activity.state = artists

      // Use the recorded song start time, or calculate it if not available
      const songStartTime = currentSongStartTime ?? (Date.now() - (position * 1000))
      const startAt = Math.max(0, songStartTime)

      presenceLogger.debug('Discord timestamp calculation', {
        currentSongStartTime,
        duration,
        position,
        songStartTime,
        startAt,
      })

      // Discord expects timestamps in seconds, not milliseconds
      activity.start_timestamp = Math.floor(startAt / 1000)

      if (duration > 0) {
        const endAt = startAt + (duration * 1000)
        activity.end_timestamp = Math.floor(endAt / 1000)

        presenceLogger.debug('Discord end timestamp', {
          duration,
          endAt,
        })
      }
    } else {
      // When paused, show static position without progress bar animation
      activity.state = artists

      if (duration > 0) {
        // Set start_timestamp to represent current position, don't set end_timestamp
        // This shows elapsed time without animated progress bar
        const now = Date.now()
        const positionMs = Math.round(position * 1000)
        const startAt = Math.max(0, now - positionMs)

        activity.start_timestamp = Math.floor(startAt / 1000)
        // Don't set end_timestamp for paused songs to avoid progress bar animation
        activity.end_timestamp = null
      }
    }

    presenceLogger.debug('Updated Discord activity', {
      artists,
      durationSeconds: duration ? Math.round(duration) : null,
      positionSeconds: Math.round(position),
      state:           isPlaying ? 'playing' : 'paused',
      title:           song.name,
    })

    // Log the actual activity payload being sent to Discord
    presenceLogger.debug('Activity payload details', {
      details:       song.name,
      hasLargeImage: !!song.albumArtUrl,
      hasSmallImage: !!artistImageUrl,
      largeImageUrl: song.albumArtUrl?.substring(0, 100), // Truncate for readability
      largeText:     song.album ?? 'Unknown Album',
      smallImageUrl: artistImageUrl?.substring(0, 100),
      state:         artists,
    })

    const activityResult = await commands.discordRpcSetActivity(activity)
    if (activityResult.status === 'error') {
      presenceLogger.error('Failed to set Discord activity', activityResult.error)
      // Reset the thread state so it will retry on next update
      hasStartedThread = false
      return
    }
    lastSuccessfulUpdate = Date.now()
    presenceLogger.debug('Discord activity set successfully')
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
      presenceLogger.error('Failed to update Discord activity', error)
    } finally {
      isUpdating = false
    }
  }

  const stopWatchers: Array<() => void> = [
    watch(() => playerStore.currentSong, () => {
      // Reset start time when song changes
      currentSongStartTime = null
      lastProgressPosition = playerStore.currentTime
      lastSignature = ''
      void updatePresence()
    }, { immediate: true }),
    watch(() => playerStore.isPlaying, isPlaying => {
      if (isPlaying && playerStore.currentSong) {
        // When playback starts, record the start time
        currentSongStartTime = Date.now() - (playerStore.currentTime * 1000)
      }
      lastSignature = ''
      void updatePresence()
    }, { immediate: true }),
    watch(() => playerStore.currentTime, position => {
      if (!playerStore.currentSong || !playerStore.isPlaying)
        return

      const positionChange = Math.abs(position - lastProgressPosition)

      // Detect seeking: if position changed by more than 10 seconds at once
      if (positionChange > 10 && lastProgressPosition >= 0) {
        // User seeked, update the song start time to maintain correct progress bar
        currentSongStartTime = Date.now() - (position * 1000)
        presenceLogger.debug('Detected seeking, updated song start time', {
          newStartTime: currentSongStartTime,
          position,
          positionChange,
        })
      }

      if (positionChange < POSITION_UPDATE_THRESHOLD)
        return

      lastProgressPosition = position
      lastSignature = ''
      void updatePresence()
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
