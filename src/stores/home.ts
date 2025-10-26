import { defineStore } from 'pinia'
import { readonly, ref } from 'vue'

import type { Album, Song } from '@/bindings'

import { commands } from '@/bindings'
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

  // Getters
  const recentlyPlayedSongs = recentlyPlayed
  const recentlyAddedAlbums = recentlyAdded
  const randomLibraryAlbums = randomAlbums
  const featuredLibraryAlbums = featuredAlbums

  // Actions
  const loadHomeData = async (): Promise<void> => {
    if (isLoaded.value) {
      logger.info('Home data already loaded, skipping.')
      return
    }

    isLoading.value = true
    error.value = null
    logger.info('Loading home data...')

    const maxRetries = 5
    let retryDelay = 200

    for (let attempt = 0; attempt < maxRetries; attempt++) {
      const result = await commands.getHomeViewData()

      if (result.status === 'ok') {
        const data = result.data
        recentlyPlayed.value = data.recently_played || []
        recentlyAdded.value = data.recently_added || []
        randomAlbums.value = data.random_albums || []
        featuredAlbums.value = data.featured_albums || []

        logger.info(
          `Home data loaded: ${recentlyPlayed.value.length} recently played, ` +
          `${recentlyAdded.value.length} recently added, ` +
          `${randomAlbums.value.length} random, ${featuredAlbums.value.length} featured`,
        )

        isLoaded.value = true
        logger.info('Home data loaded successfully')
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

  const refreshHomeData = async (): Promise<void> => {
    resetHomeData()
    await loadHomeData()
  }

  const resetHomeData = (): void => {
    recentlyPlayed.value = []
    recentlyAdded.value = []
    randomAlbums.value = []
    featuredAlbums.value = []
    isLoaded.value = false
    error.value = null
    logger.info('Home data reset')
  }

  return {
    // State
    error:    readonly(error),
    // Data
    featuredLibraryAlbums,
    isLoaded: readonly(isLoaded),

    isLoading: readonly(isLoading),
    // Actions
    loadHomeData,
    randomLibraryAlbums,
    recentlyAddedAlbums,

    recentlyPlayedSongs,
    refreshHomeData,
    resetHomeData,
  }
})