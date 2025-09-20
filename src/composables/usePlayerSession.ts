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

  const initialize = async () => {
    if (isInitialized) return

    await sessionManager.initializeSession()
    await sessionManager.registerCapabilities()
    isInitialized = true

    logger.info('Player session integration initialized')
  }

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
        'TimeUpdate',
        false,
      )
    }, 30000) // Report every 30 seconds
  }

  const stopProgressReporting = () => {
    if (progressReportTimer) {
      clearInterval(progressReportTimer)
      progressReportTimer = null
    }
  }

  watch(
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
        startProgressReporting()
      }
    },
    { immediate: true },
  )

  watch(
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
        startProgressReporting()
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

  watch(
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

  onUnmounted(async () => {
    stopProgressReporting()

    if (playerStore.currentSong && playerStore.isPlaying)
      await sessionManager.reportPlaybackStop(
        playerStore.currentSong.id,
        playerStore.currentTime,
      )
  })

  watch(
    () => authStore.isAuthenticated(),
    async isAuthenticated => {
      if (isAuthenticated)
        await initialize()
    },
    { immediate: true },
  )

  onMounted(async () => {
    if (authStore.isAuthenticated())
      await initialize()
  })

  return {
    sessionManager,
    initialize,
  }
}
