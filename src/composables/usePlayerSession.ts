import { watch, onMounted, onUnmounted } from 'vue'
import { usePlayerStore, useAuthStore } from '@/stores'
import { useSession } from './useSession'
import { logger } from '@/lib/logger'

export const usePlayerSession = () => {
  const playerStore = usePlayerStore()
  const authStore = useAuthStore()
  const sessionManager = useSession()

  // Track current playback state to avoid duplicate reports
  const lastReportedState = {
    isPlaying:     false,
    currentSongId: '',
    position:      0,
  }

  // Track which song has already been marked as played to avoid duplicates
  let lastMarkedPlayedSongId = ''

  let progressReportTimer: ReturnType<typeof setInterval> | null = null
  let isInitialized = false

  // Initialize session management
  const initialize = async () => {
    if (isInitialized) return

    await sessionManager.initializeSession()
    await sessionManager.registerCapabilities()
    isInitialized = true

    logger.info('Player session integration initialized')
  }

  // Start progress reporting timer
  const startProgressReporting = () => {
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
        'TimeUpdate', // event name for time update
        false, // not paused
      )
    }, 30000) // Report every 30 seconds
  }

  // Stop progress reporting timer
  const stopProgressReporting = () => {
    if (progressReportTimer) {
      clearInterval(progressReportTimer)
      progressReportTimer = null
    }
  }

  // Watch for song changes
  watch(
    () => playerStore.currentSong,
    async (newSong, oldSong) => {
      // If there was a previous song playing, report it as stopped
      if (oldSong && playerStore.isPlaying) {
        await sessionManager.reportPlaybackStop(oldSong.id, playerStore.currentTime)
      }

      // If there's a new song and we're playing, report playback start
      if (newSong && playerStore.isPlaying) {
        await sessionManager.reportPlaybackStart(
          newSong.id,
          0, // Start from beginning
        )

        lastReportedState.currentSongId = newSong.id
        // Reset the marked played tracking for the new song
        lastMarkedPlayedSongId = ''
        startProgressReporting()
      }
    },
    { immediate: true },
  )

  // Watch for play/pause state changes
  watch(
    () => playerStore.isPlaying,
    async (isPlaying, wasPlaying) => {
      if (!playerStore.currentSong) {
        return
      }

      // Avoid duplicate reports
      if (isPlaying === lastReportedState.isPlaying) {
        return
      }
      lastReportedState.isPlaying = isPlaying

      if (isPlaying && !wasPlaying) {
        // Started playing
        await sessionManager.reportPlaybackStart(
          playerStore.currentSong.id,
          playerStore.currentTime,
        )
        startProgressReporting()
      } else if (!isPlaying && wasPlaying) {
        // Stopped playing
        await sessionManager.reportPlaybackStop(
          playerStore.currentSong.id,
          playerStore.currentTime,
        )
        stopProgressReporting()
      }
    },
    { immediate: true },
  )

  // Watch for song completion (when position reaches duration)
  watch(
    [() => playerStore.currentTime, () => playerStore.duration],
    async ([currentTime, duration]) => {
      if (!playerStore.currentSong || !playerStore.isPlaying) return

      // If song is near completion (within 1 second), mark as played
      // Only mark once per song to avoid duplicate calls
      if (duration > 0 && currentTime >= duration - 1 && playerStore.currentSong.id !== lastMarkedPlayedSongId) {
        lastMarkedPlayedSongId = playerStore.currentSong.id
        await sessionManager.markItemPlayed(playerStore.currentSong.id)
        logger.debug('Song marked as played', { songId: playerStore.currentSong.id })
      }
    },
  )

  // Cleanup on unmount
  onUnmounted(async () => {
    stopProgressReporting()

    // Report final stop if currently playing
    if (playerStore.currentSong && playerStore.isPlaying) {
      await sessionManager.reportPlaybackStop(
        playerStore.currentSong.id,
        playerStore.currentTime,
      )
    }
  })

  // Watch for auth state changes and initialize session when authenticated
  watch(
    () => authStore.isAuthenticated(),
    async isAuthenticated => {
      if (isAuthenticated) {
        await initialize()
      }
    },
    { immediate: true },
  )

  // Initialize on mount (in case already authenticated)
  onMounted(async () => {
    if (authStore.isAuthenticated()) {
      await initialize()
    }
  })

  return {
    sessionManager,
    initialize,
  }
}
