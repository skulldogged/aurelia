import { defineStore } from 'pinia'
import { readonly, ref } from 'vue'

import type { Album, Song } from '@/bindings'

import { commands } from '@/bindings'
import { appLogger } from '@/lib/logger'

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
      appLogger.info('Home data already loaded, skipping.')
      return
    }

    isLoading.value = true
    error.value = null
    appLogger.info('Loading home data...')

    const result = await commands.getHomeViewData()

    if (result.status === 'ok') {
      recentlyPlayed.value = result.data.recently_played
      recentlyAdded.value = result.data.recently_added
      randomAlbums.value = result.data.random_albums
      featuredAlbums.value = result.data.featured_albums
      isLoaded.value = true
      appLogger.info('Home data loaded successfully')
    } else {
      error.value = 'Failed to load home data'
      appLogger.error('Failed to load home data:', result.error)
    }

    isLoading.value = false
  }

  const refreshHomeData = async (): Promise<void> => {
    isLoaded.value = false
    await loadHomeData()
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
  }
})