import { defineStore } from 'pinia'
import { Effect, Schedule } from 'effect'
import { computed, readonly, ref } from 'vue'

import type { Album, Song } from '../lib/api/types'

import { ApiError } from '../effect/errors'
import { getHomeViewDataEffect } from '../effect/services/api'
import { runAureliaEffect } from '../effect/runtime'
import { getAuthLogout } from '../lib/auth-interceptor'
import { logger } from '../lib/logger'

const HOME_DATA_MAX_RETRIES = 3
const HOME_DATA_RETRY_DELAY_MS = 200

const isWaitingForLibraryError = (errorMessage: string): boolean =>
  errorMessage.includes('Library not loaded')

const normalizeHomeData = (raw: Record<string, unknown>): {
  featuredAlbums: Album[]
  randomAlbums:   Album[]
  recentlyAdded:  Album[]
  recentlyPlayed: Song[]
} => ({
  featuredAlbums: (raw.featuredAlbums ?? raw.featured_albums ?? []) as Album[],
  randomAlbums:   (raw.randomAlbums ?? raw.random_albums ?? []) as Album[],
  recentlyAdded:  (raw.recentlyAdded ?? raw.recently_added ?? []) as Album[],
  recentlyPlayed: (raw.recentlyPlayed ?? raw.recently_played ?? []) as Song[],
})

export const useHomeStore = defineStore('home', () => {
  // State
  const recentlyPlayed = ref<Song[]>([])
  const recentlyAdded = ref<Album[]>([])
  const randomAlbums = ref<Album[]>([])
  const featuredAlbums = ref<Album[]>([])
  const isLoading = ref(false)
  const isLoaded = ref(false)
  const error = ref<null | string>(null)

  // Progressive loading state
  const loadingStage = ref<'extended' | 'full' | 'initial'>('initial')
  const hasMoreData = ref({
    featuredAlbums: false,
    randomAlbums:   false,
    recentlyAdded:  false,
    recentlyPlayed: false,
  })

  // Getters with progressive limits
  const recentlyPlayedSongs = computed(() => {
    if (loadingStage.value === 'initial') return recentlyPlayed.value.slice(0, 10)
    if (loadingStage.value === 'extended') return recentlyPlayed.value.slice(0, 25)
    return recentlyPlayed.value
  })

  const recentlyAddedAlbums = computed(() => {
    if (loadingStage.value === 'initial') return recentlyAdded.value.slice(0, 12)
    if (loadingStage.value === 'extended') return recentlyAdded.value.slice(0, 30)
    return recentlyAdded.value
  })

  const randomLibraryAlbums = computed(() => {
    if (loadingStage.value === 'initial') return randomAlbums.value.slice(0, 12)
    if (loadingStage.value === 'extended') return randomAlbums.value.slice(0, 30)
    return randomAlbums.value
  })

  const featuredLibraryAlbums = featuredAlbums

  // Actions
  const loadHomeData = async (stage: 'extended' | 'full' | 'initial' = 'initial'): Promise<void> => {
    // Skip if we're already at or beyond this stage
    if (
      (stage === 'initial' && loadingStage.value !== 'initial') ||
      (stage === 'extended' && loadingStage.value === 'full') ||
      (stage === 'full' && loadingStage.value === 'full')
    ) {
      logger.info(`Home data stage '${stage}' already loaded, skipping.`)
      return
    }

    isLoading.value = true
    error.value = null
    logger.info(`Loading home data stage: ${stage}...`)

    let attemptCount = 0

    try {
      const data = await runAureliaEffect(
        getHomeViewDataEffect().pipe(
          Effect.tapError(apiError =>
            Effect.sync(() => {
              attemptCount += 1
              if (
                isWaitingForLibraryError(apiError.message)
                && attemptCount < HOME_DATA_MAX_RETRIES
              ) {
                const retryDelay = HOME_DATA_RETRY_DELAY_MS * 2 ** (attemptCount - 1)
                logger.warn(
                  `Home data not ready yet (attempt ${attemptCount}/${HOME_DATA_MAX_RETRIES}). ` +
                  `Retrying in ${retryDelay}ms`,
                )
              }
            }),
          ),
          Effect.retry({
            schedule: Schedule.exponential(HOME_DATA_RETRY_DELAY_MS),
            times:    HOME_DATA_MAX_RETRIES - 1,
            while:    apiError => isWaitingForLibraryError(apiError.message),
          }),
          Effect.map(result => normalizeHomeData(result as Record<string, unknown>)),
        ),
      )

      // Store full data but only expose progressive amounts
      recentlyPlayed.value = data.recentlyPlayed
      recentlyAdded.value = data.recentlyAdded
      randomAlbums.value = data.randomAlbums
      featuredAlbums.value = data.featuredAlbums

      // Track if we have more data for progressive loading
      hasMoreData.value = {
        featuredAlbums: false, // Featured albums are typically limited already
        randomAlbums:   (data.randomAlbums?.length || 0) > getStageLimit('randomAlbums', stage),
        recentlyAdded:  (data.recentlyAdded?.length || 0) > getStageLimit('recentlyAdded', stage),
        recentlyPlayed: (data.recentlyPlayed?.length || 0) > getStageLimit('recentlyPlayed', stage),
      }

      loadingStage.value = stage
      isLoaded.value = true

      logger.info(
        `Home data stage '${stage}' loaded: ${recentlyPlayed.value.length} recently played, ` +
        `${recentlyAdded.value.length} recently added, ` +
        `${randomAlbums.value.length} random, ${featuredAlbums.value.length} featured`,
      )
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : 'Failed to load home data'

      if (isWaitingForLibraryError(errorMessage)) {
        error.value = 'Library not loaded yet. Please try again shortly.'
        logger.error('Max retries reached while waiting for library to load for home data')
        return
      }

      error.value = errorMessage
      logger.error('Failed to load home data:', errorMessage)
      if (errorMessage.toLowerCase().includes('unauthorized')) {
        const logout = getAuthLogout()
        if (logout)
          logout()
      }
    } finally {
      isLoading.value = false
    }
  }

  // Helper function to get stage limits
  const getStageLimit = (dataType: string, stage: string): number => {
    const limits = {
      extended: { randomAlbums: 30, recentlyAdded: 30, recentlyPlayed: 25 },
      full:     { randomAlbums: Infinity, recentlyAdded: Infinity, recentlyPlayed: Infinity },
      initial:  { randomAlbums: 12, recentlyAdded: 12, recentlyPlayed: 10 },
    }
    return limits[stage as keyof typeof limits]?.[dataType as keyof typeof limits.initial] || 0
  }

  // Progressive loading functions
  const loadMoreData = async (): Promise<void> => {
    if (loadingStage.value === 'initial') {
      await loadHomeData('extended')
    } else if (loadingStage.value === 'extended') {
      await loadHomeData('full')
    }
  }

  const loadInitialData = async (): Promise<void> => {
    await loadHomeData('initial')
  }

  const refreshHomeData = async (): Promise<void> => {
    resetHomeData()
    await loadInitialData()
  }

  const resetHomeData = (): void => {
    recentlyPlayed.value = []
    recentlyAdded.value = []
    randomAlbums.value = []
    featuredAlbums.value = []
    isLoaded.value = false
    loadingStage.value = 'initial'
    hasMoreData.value = {
      featuredAlbums: false,
      randomAlbums:   false,
      recentlyAdded:  false,
      recentlyPlayed: false,
    }
    error.value = null
    logger.info('Home data reset')
  }

  return {
    // State
    error:       readonly(error),
    // Data
    featuredLibraryAlbums,
    hasMoreData: readonly(hasMoreData),
    isLoaded:    readonly(isLoaded),
    isLoading:   readonly(isLoading),
    // Actions
    loadHomeData,

    loadingStage: readonly(loadingStage),
    loadInitialData,
    loadMoreData,
    randomLibraryAlbums,
    recentlyAddedAlbums,
    recentlyPlayedSongs,
    refreshHomeData,
    resetHomeData,
  }
})
