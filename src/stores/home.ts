import { defineStore } from 'pinia'
import { readonly, ref } from 'vue'

import type { Album, Song } from '@/bindings'

import { commands } from '@/bindings'
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

    const result = await commands.getHomeViewData()

    console.log('Home data result:', result)

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
    } else {
      error.value = 'Failed to load home data'
      logger.error('Failed to load home data:', result.error)
    }

    isLoading.value = false
  }

  const refreshHomeData = async (): Promise<void> => {
    isLoaded.value = false
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