import { readonly, ref, type Ref } from 'vue'

import type { Album, Artist, Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { appLogger } from '@/lib/logger'
import { withCustomState, withMultipleResults } from '@/lib/result'

const albumArtistsWithSongs = ref<Artist[]>([])
const allAlbums = ref<Album[]>([])
const allArtistsWithSongs = ref<Artist[]>([])
const allSongs = ref<Song[]>([])
const libraryError = ref<null | string>(null)
const libraryLoading = ref(false)

const loadLibrary = async (credentials: Credentials): Promise<void> => {
  // Load songs first
  await withCustomState(
    () => commands.getSongs(credentials.serverUrl, credentials.token, null, null, null, null),
    {
      onError: error => {
        libraryError.value = `Failed to load songs: ${error}`
        libraryLoading.value = false
      },
      onStart: () => {
        libraryLoading.value = true
        libraryError.value = null
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
        libraryError.value = `Failed to load library data: ${errors.join(', ')}`
        libraryLoading.value = false
      },
      onSuccess: ([artistsWithSongs, albumArtists, albums]) => {
        allArtistsWithSongs.value = artistsWithSongs
        albumArtistsWithSongs.value = albumArtists
        allAlbums.value = albums
        libraryError.value = null
        libraryLoading.value = false
      },
    },
  )
}

const syncLibrary = async (credentials: Credentials): Promise<void> => {
  appLogger.info('Starting library sync...')

  await withCustomState(
    () => commands.syncLibrary(credentials.serverUrl, credentials.token),
    {
      onError: error => {
        const errorMessage = `Failed to sync library: ${error}`
        libraryError.value = errorMessage
        appLogger.error('Failed to sync library:', error)
      },
      onSuccess: async () => {
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
      onError: error => {
        const errorMessage = `Failed to clear cache: ${error}`
        libraryError.value = errorMessage
        appLogger.error('Failed to clear cache:', error)
      },
      onSuccess: async () => {
        await loadLibrary(credentials)
        appLogger.info('Cache cleared and library reloaded successfully.')
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
