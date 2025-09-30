import { clearActivity, isRunning, setActivity, start, stop } from 'tauri-plugin-drpc'
import { Activity, ActivityType, Assets, Timestamps } from 'tauri-plugin-drpc/activity'
import { onBeforeUnmount, ref, type Ref, watch } from 'vue'

import type { Song } from '@/bindings'

import { presenceLogger } from '@/lib/logger'
import { usePlayerStore } from '@/stores'

const DISCORD_APP_ID =
  import.meta.env.VITE_DISCORD_APP_ID
  || ''

const POSITION_UPDATE_THRESHOLD = 10
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

  const ensureThread = async (): Promise<boolean> => {
    if (hasStartedThread)
      return true

    try {
      if (!(await isRunning())) {
        await start(DISCORD_APP_ID)
        presenceLogger.debug('Started Discord RPC thread')
      }

      hasStartedThread = true
      return true
    } catch (error) {
      presenceLogger.error('Failed to start Discord RPC thread', error)
      return false
    }
  }

  const stopThread = async (): Promise<void> => {
    try {
      await clearActivity()
    } catch (error) {
      presenceLogger.debug('Failed to clear Discord activity on shutdown', error)
    }

    try {
      await stop()
      presenceLogger.debug('Stopped Discord RPC thread')
    } catch (error) {
      presenceLogger.debug('Failed to stop Discord RPC thread', error)
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

      await clearActivity()
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

    const assets = new Assets()

    if (song.albumArtUrl)
      assets
        .setLargeImage(song.albumArtUrl)
        .setLargeText(`Listening to ${song.name}`)

    if (artistImageUrl)
      assets
        .setSmallImage(artistImageUrl)
        .setSmallText(artists)

    const activity = new Activity()
      .setActivity(ActivityType.Listening)
      .setDetails(song.name)
      .setAssets(assets)

    if (isPlaying) {
      activity.setState(artists)

      const now = Date.now()
      const startAt = Math.max(0, now - Math.round(position * 1000))

      if (duration > 0) {
        const remaining = Math.max(0, duration - position)
        const endAt = startAt + Math.round(remaining * 1000)
        activity.setTimestamps(new Timestamps(startAt, endAt))
      } else {
        activity.setTimestamps(new Timestamps(startAt))
      }
    } else {
      activity.setState(`Paused — ${artists}`)
    }

    presenceLogger.debug('Updated Discord activity', {
      artists,
      durationSeconds: duration ? Math.round(duration) : null,
      positionSeconds: Math.round(position),
      state:           isPlaying ? 'playing' : 'paused',
      title:           song.name,
    })

    await setActivity(activity)
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
      lastProgressPosition = playerStore.currentTime
      lastSignature = ''
      void updatePresence()
    }, { immediate: true }),
    watch(() => playerStore.isPlaying, () => {
      lastSignature = ''
      void updatePresence()
    }, { immediate: true }),
    watch(() => playerStore.currentTime, position => {
      if (!playerStore.currentSong || !playerStore.isPlaying)
        return

      if (Math.abs(position - lastProgressPosition) < POSITION_UPDATE_THRESHOLD)
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
