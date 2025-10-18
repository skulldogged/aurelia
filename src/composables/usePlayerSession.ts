import { onUnmounted, watch } from 'vue'

import { logger } from '@/lib/logger'
import { useAuthStore, usePlayerStore } from '@/stores'

import { useSession } from './useSession'

// Track current playback state to avoid duplicate reports
const lastReportedState = {
  currentSongId: '',
  isPlaying:     false,
  position:      0,
}

// Track which song has already been marked as played to avoid duplicates
let lastMarkedPlayedSongId = ''

let progressReportTimer: null | ReturnType<typeof setInterval> = null
let isInitialized = false

const initialize = async (sessionManager: ReturnType<typeof useSession>): Promise<void> => {
  if (isInitialized) {
    logger.debug('Player session already initialized, skipping')
    return
  }

  logger.info('Initializing player session...')
  await sessionManager.initializeSession()
  await sessionManager.registerCapabilities()
  isInitialized = true

  logger.info('Player session integration initialized')
}

const startProgressReporting = (
  playerStore: ReturnType<typeof usePlayerStore>,
  sessionManager: ReturnType<typeof useSession>,
): void => {
  if (progressReportTimer) return

  progressReportTimer = setInterval(async () => {
    if (!playerStore.currentSong || !playerStore.isPlaying) return

    // Only report progress if position has changed significantly (>5 seconds)
    const currentPosition = playerStore.currentTime
    if (Math.abs(currentPosition - lastReportedState.position) < 5) return

    lastReportedState.position = currentPosition

    await sessionManager.reportPlaybackProgress(
      playerStore.currentSong.id,
      currentPosition,
      'TimeUpdate',
      false,
    )
  }, 30000) // Report every 30 seconds
}

const stopProgressReporting = (): void => {
  if (progressReportTimer) {
    clearInterval(progressReportTimer)
    progressReportTimer = null
  }
}

const setupWatchers = (
  playerStore: ReturnType<typeof usePlayerStore>,
  authStore: ReturnType<typeof useAuthStore>,
  sessionManager: ReturnType<typeof useSession>,
): (() => void) => {
  const unwatchCurrentSong = watch(
    () => playerStore.currentSong,
    async (newSong, oldSong) => {
      if (oldSong && playerStore.isPlaying)
        await sessionManager.reportPlaybackStop(oldSong.id, playerStore.currentTime)

      if (newSong && playerStore.isPlaying) {
        await sessionManager.reportPlaybackStart(
          newSong.id,
          0,
        )

        lastReportedState.currentSongId = newSong.id
        lastMarkedPlayedSongId = ''
        startProgressReporting(playerStore, sessionManager)
      }
    },
    { immediate: true },
  )

  const unwatchIsPlaying = watch(
    () => playerStore.isPlaying,
    async (isPlaying, wasPlaying) => {
      if (!playerStore.currentSong)
        return

      if (isPlaying === lastReportedState.isPlaying)
        return

      lastReportedState.isPlaying = isPlaying

      if (isPlaying && !wasPlaying) {
        await sessionManager.reportPlaybackStart(
          playerStore.currentSong.id,
          playerStore.currentTime,
        )
        startProgressReporting(playerStore, sessionManager)
      } else if (!isPlaying && wasPlaying) {
        await sessionManager.reportPlaybackStop(
          playerStore.currentSong.id,
          playerStore.currentTime,
        )
        stopProgressReporting()
      }
    },
    { immediate: true },
  )

  const unwatchTimeAndDuration = watch(
    [() => playerStore.currentTime, () => playerStore.duration],
    async ([currentTime, duration]) => {
      if (!playerStore.currentSong || !playerStore.isPlaying) return

      // If song is near completion (within 1 second), mark as played
      // Only mark once per song to avoid duplicate calls
      if (duration > 0 && currentTime >= duration - 1 && playerStore.currentSong.id !== lastMarkedPlayedSongId) {
        lastMarkedPlayedSongId = playerStore.currentSong.id
        await sessionManager.markItemPlayed(playerStore.currentSong.id)
      }
    },
  )

  const unwatchAuth = watch(
    () => authStore.isAuthenticated(),
    async isAuthenticated => {
      logger.debug('Auth status changed:', isAuthenticated)
      if (isAuthenticated)
        await initialize(sessionManager)
    },
    { immediate: true },
  )

  // Return cleanup function
  return () => {
    unwatchCurrentSong()
    unwatchIsPlaying()
    unwatchTimeAndDuration()
    unwatchAuth()
  }
}

export interface PlayerSession {
  initialize:     () => Promise<void>
  sessionManager: ReturnType<typeof useSession>
}

export const usePlayerSession = (): PlayerSession => {
  const playerStore = usePlayerStore()
  const authStore = useAuthStore()
  const sessionManager = useSession()

  // Session initialization is handled by the auth status watcher below

  // Set up watchers
  const cleanupWatchers = setupWatchers(playerStore, authStore, sessionManager)

  // Set up cleanup on unmount
  onUnmounted(async () => {
    stopProgressReporting()

    if (playerStore.currentSong && playerStore.isPlaying)
      await sessionManager.reportPlaybackStop(
        playerStore.currentSong.id,
        playerStore.currentTime,
      )

    cleanupWatchers()
  })

  return {
    initialize: () => initialize(sessionManager),
    sessionManager,
  }
}
