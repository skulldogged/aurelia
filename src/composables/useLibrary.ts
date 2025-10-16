import { readonly, ref, type Ref } from 'vue'

import type { Album, Artist, Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { logger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'

const albumArtistsWithSongs = ref<Artist[]>([])
const allAlbums = ref<Album[]>([])
const allArtistsWithSongs = ref<Artist[]>([])
const allSongs = ref<Song[]>([])
const libraryError = ref<null | string>(null)
const libraryLoading = ref(false)

const loadLibrary = async (_credentials: Credentials): Promise<void> => {
  await withCustomState(
    () => commands.getLibrary(),
    {
      onError: error => {
        libraryError.value = `Failed to load library: ${error}`
        libraryLoading.value = false
      },
      onStart: () => {
        libraryLoading.value = true
        libraryError.value = null
      },
      onSuccess: libraryData => {
        allSongs.value = libraryData.songs
        allArtistsWithSongs.value = libraryData.artists
        albumArtistsWithSongs.value = libraryData.artists
        allAlbums.value = libraryData.albums
        libraryError.value = null
        libraryLoading.value = false
      },
    },
  )
}

const syncLibrary = async (credentials: Credentials): Promise<void> => {
  logger.info('Starting library sync...')

  await withCustomState(
    () => commands.syncLibrary(credentials.serverUrl, credentials.token),
    {
      onError: error => {
        const errorMessage = `Failed to sync library: ${error}`
        libraryError.value = errorMessage
        logger.error('Failed to sync library:', error)
      },
      onSuccess: async () => {
        await loadLibrary(credentials)
        logger.info('Library sync completed successfully.')
      },
    },
  )
}

const clearCache = async (credentials: Credentials): Promise<void> => {
  logger.info('Starting cache clear...')

  await withCustomState(
    () => commands.clearCache(credentials.serverUrl, credentials.token),
    {
      onError: error => {
        const errorMessage = `Failed to clear cache: ${error}`
        libraryError.value = errorMessage
        logger.error('Failed to clear cache:', error)
      },
      onSuccess: async () => {
        await loadLibrary(credentials)
        logger.info('Cache cleared and library reloaded successfully.')
      },
    },
  )
}

export interface Library {
  albumArtistsWithSongs: Ref<Artist[]>
  allAlbums:             Ref<Album[]>
  allArtistsWithSongs:   Ref<Artist[]>
  allSongs:              Ref<Song[]>
  clearCache:            (credentials: Credentials) => Promise<void>
  libraryError:          Readonly<Ref<null | string>>
  libraryLoading:        Readonly<Ref<boolean>>
  loadLibrary:           (credentials: Credentials) => Promise<void>
  syncLibrary:           (credentials: Credentials) => Promise<void>
}

export const useLibrary = (): Library => ({
  albumArtistsWithSongs,
  allAlbums,
  allArtistsWithSongs,
  allSongs,
  clearCache,
  libraryError:   readonly(libraryError),
  libraryLoading: readonly(libraryLoading),
  loadLibrary,
  syncLibrary,
})
