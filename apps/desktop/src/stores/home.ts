import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'

import type { Album, Song } from '@/lib/api/bindings'

import { commands } from '@/lib/api/bindings'
import { getAuthLogout } from '@/lib/auth-interceptor'
import { logger } from '@/lib/logger'

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

    const maxRetries = 3 // Reduce retries for progressive loading
    let retryDelay = 200

    for (let attempt = 0; attempt < maxRetries; attempt++) {
      const result = await commands.getHomeViewData()

      if (result.status === 'ok') {
        const data = result.data

        // Store full data but only expose progressive amounts
        recentlyPlayed.value = data.recently_played || []
        recentlyAdded.value = data.recently_added || []
        randomAlbums.value = data.random_albums || []
        featuredAlbums.value = data.featured_albums || []

        // Track if we have more data for progressive loading
        hasMoreData.value = {
          featuredAlbums: false, // Featured albums are typically limited already
          randomAlbums:   (data.random_albums?.length || 0) > getStageLimit('randomAlbums', stage),
          recentlyAdded:  (data.recently_added?.length || 0) > getStageLimit('recentlyAdded', stage),
          recentlyPlayed: (data.recently_played?.length || 0) > getStageLimit('recentlyPlayed', stage),
        }

        loadingStage.value = stage
        isLoaded.value = true

        logger.info(
          `Home data stage '${stage}' loaded: ${recentlyPlayed.value.length} recently played, ` +
          `${recentlyAdded.value.length} recently added, ` +
          `${randomAlbums.value.length} random, ${featuredAlbums.value.length} featured`,
        )

        isLoading.value = false
        return
      }

      const errorMessage = result.error ?? 'Failed to load home data'
      const isWaitingForLibrary = errorMessage.includes('Library not loaded')

      if (!isWaitingForLibrary) {
        error.value = errorMessage
        logger.error('Failed to load home data:', errorMessage)
        if (errorMessage.toLowerCase().includes('unauthorized')) {
          const logout = getAuthLogout()
          if (logout)
            logout()
        }
        isLoading.value = false
        return
      }

      const attemptNumber = attempt + 1

      if (attemptNumber >= maxRetries) {
        error.value = 'Library not loaded yet. Please try again shortly.'
        logger.error('Max retries reached while waiting for library to load for home data')
        isLoading.value = false
        return
      }

      logger.warn(
        `Home data not ready yet (attempt ${attemptNumber}/${maxRetries}). Retrying in ${retryDelay}ms`,
      )
      await new Promise(resolve => setTimeout(resolve, retryDelay))
      retryDelay *= 2
    }

    isLoading.value = false
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