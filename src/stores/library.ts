import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { Song, Album, Artist, Credentials } from '@/bindings'
import { commands } from '@/bindings'
import { appLogger } from '@/lib/logger'

export const useLibraryStore = defineStore('library', () => {
  // State
  const allSongs = ref<Song[]>([])
  const allArtistsWithSongs = ref<Artist[]>([])
  const albumArtistsWithSongs = ref<Artist[]>([])
  const allAlbums = ref<Album[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const isLoaded = ref(false)

  // Actions
  const loadLibrary = async (credentials: Credentials) => {
    if (isLoaded.value) {
      // Already loaded, no need to fetch again
      return
    }

    isLoading.value = true
    error.value = null

    try {
      appLogger.info('Loading library data...')

      const songsResult = await commands.getSongs(
        credentials.serverUrl,
        credentials.token,
        null, null, null, null,
      )

      if (songsResult.status === 'error') {
        throw new Error(`Failed to load songs: ${songsResult.error}`)
      }

      allSongs.value = songsResult.data

      const [artistsWithSongsResult, albumArtistsResult, albumsResult] = await Promise.all([
        commands.getArtists(credentials.serverUrl, credentials.token, true, false, null, null),
        commands.getArtists(credentials.serverUrl, credentials.token, true, true, null, null),
        commands.getAlbums(credentials.serverUrl, credentials.token, true, null, null), // includeSongs: true
      ])

      if (artistsWithSongsResult.status === 'error') {
        throw new Error(`Failed to load artists: ${artistsWithSongsResult.error}`)
      }
      if (albumArtistsResult.status === 'error') {
        throw new Error(`Failed to load album artists: ${albumArtistsResult.error}`)
      }
      if (albumsResult.status === 'error') {
        throw new Error(`Failed to load albums: ${albumsResult.error}`)
      }

      allArtistsWithSongs.value = artistsWithSongsResult.data
      albumArtistsWithSongs.value = albumArtistsResult.data
      allAlbums.value = albumsResult.data

      isLoaded.value = true
      appLogger.info('Library data loaded successfully')
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load music library'
      error.value = errorMessage
      appLogger.error('Failed to load library:', err)
      throw new Error(errorMessage)
    } finally {
      isLoading.value = false
    }
  }

  const syncLibrary = async (credentials: Credentials) => {
    try {
      appLogger.info('Starting library sync...')
      const syncResult = await commands.syncLibrary(credentials.serverUrl, credentials.token)
      if (syncResult.status === 'error')
        throw new Error(`Failed to sync library: ${syncResult.error}`)

      // Reset loaded state to force reload
      isLoaded.value = false
      await loadLibrary(credentials)
      appLogger.info('Library sync completed successfully.')
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to sync music library'
      error.value = errorMessage
      appLogger.error('Failed to sync library:', err)
      throw new Error(errorMessage)
    }
  }

  const clearCache = async (credentials: Credentials) => {
    try {
      appLogger.info('Starting cache clear...')
      const clearResult = await commands.clearCache(credentials.serverUrl, credentials.token)
      if (clearResult.status === 'error')
        throw new Error(`Failed to clear cache: ${clearResult.error}`)

      // Reset loaded state to force reload
      isLoaded.value = false
      await loadLibrary(credentials)
      appLogger.info('Cache clear completed successfully.')
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to clear music cache'
      error.value = errorMessage
      appLogger.error('Failed to clear cache:', err)
      throw new Error(errorMessage)
    }
  }

  const clearData = () => {
    allSongs.value = []
    allArtistsWithSongs.value = []
    albumArtistsWithSongs.value = []
    allAlbums.value = []
    isLoaded.value = false
    error.value = null
  }

  return {
    // State
    allSongs: readonly(allSongs),
    allArtistsWithSongs: readonly(allArtistsWithSongs),
    albumArtistsWithSongs: readonly(albumArtistsWithSongs),
    allAlbums: readonly(allAlbums),
    isLoading: readonly(isLoading),
    error: readonly(error),
    isLoaded: readonly(isLoaded),

    // Actions
    loadLibrary,
    syncLibrary,
    clearCache,
    clearData,
  }
})
