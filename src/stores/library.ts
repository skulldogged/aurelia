import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'

import type { Album, Artist, Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { appLogger } from '@/lib/logger'
import { withCustomState, withMultipleResults } from '@/lib/result'

export const useLibraryStore = defineStore('library', () => {
  // State
  const allSongs = ref<Song[]>([])
  const allSongsView = computed(() => allSongs.value)
  const allArtistsWithSongs = ref<Artist[]>([])
  const albumArtistsWithSongs = ref<Artist[]>([])
  const allAlbums = ref<Album[]>([])
  const isLoading = ref(false)
  const error = ref<null | string>(null)
  const isLoaded = ref(false)

  // Actions
  const loadLibrary = async (credentials: Credentials): Promise<void> => {
    if (isLoaded.value)
      return

    // Load songs first
    await withCustomState(
      () => commands.getSongs(credentials.serverUrl, credentials.token, null, null, null, null),
      {
        onError: errorString => {
          error.value = `Failed to load songs: ${errorString}`
          appLogger.error('Failed to load songs:', errorString)
          isLoading.value = false
        },
        onStart: () => {
          isLoading.value = true
          error.value = null
          appLogger.info('Loading library data...')
        },
        onSuccess: songs => {
          allSongs.value = songs
        },
      },
    )

    // Then load artists and albums in parallel
    await withMultipleResults(
      [
        () => commands.getArtists(credentials.serverUrl, credentials.token, true, false, null, null),
        () => commands.getArtists(credentials.serverUrl, credentials.token, true, true, null, null),
        () => commands.getAlbums(credentials.serverUrl, credentials.token, true, null, null),
      ] as const,
      {
        onError: errors => {
          error.value = `Failed to load library data: ${errors.join(', ')}`
          isLoading.value = false
          appLogger.error('Failed to load library data:', errors)
        },
        onSuccess: ([artistsWithSongs, albumArtists, albums]) => {
          allArtistsWithSongs.value = artistsWithSongs
          albumArtistsWithSongs.value = albumArtists
          allAlbums.value = albums
          isLoaded.value = true
          isLoading.value = false
          appLogger.info('Library data loaded successfully')
        },
      },
    )
  }

  const syncLibrary = async (credentials: Credentials): Promise<void> => {
    appLogger.info('Starting library sync...')

    await withCustomState(
      () => commands.syncLibrary(credentials.serverUrl, credentials.token),
      {
        onError: errorString => {
          const errorMessage = `Failed to sync library: ${errorString}`
          error.value = errorMessage
          appLogger.error('Failed to sync library:', errorString)
        },
        onSuccess: async () => {
          // Reset loaded state to force reload
          isLoaded.value = false
          await loadLibrary(credentials)
          appLogger.info('Library sync completed successfully.')
        },
      },
    )
  }

  const clearCache = async (credentials: Credentials): Promise<void> => {
    appLogger.info('Starting cache clear...')

    await withCustomState(
      () => commands.clearCache(credentials.serverUrl, credentials.token),
      {
        onError: errorString => {
          const errorMessage = `Failed to clear cache: ${errorString}`
          error.value = errorMessage
          appLogger.error('Failed to clear cache:', errorString)
        },
        onSuccess: async () => {
          // Reset loaded state to force reload
          isLoaded.value = false
          await loadLibrary(credentials)
          appLogger.info('Cache clear completed successfully.')
        },
      },
    )
  }

  const clearData = (): void => {
    allSongs.value = []
    allArtistsWithSongs.value = []
    albumArtistsWithSongs.value = []
    allAlbums.value = []
    isLoaded.value = false
    error.value = null
  }

  return {
    albumArtistsWithSongs: readonly(albumArtistsWithSongs),
    allAlbums:             readonly(allAlbums),
    allArtistsWithSongs:   readonly(allArtistsWithSongs),
    // State
    allSongs:              allSongsView,
    clearCache,
    clearData,
    error:                 readonly(error),

    isLoaded:  readonly(isLoaded),
    isLoading: readonly(isLoading),
    // Actions
    loadLibrary,
    syncLibrary,
  }
})
