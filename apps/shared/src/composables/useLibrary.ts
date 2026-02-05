import { readonly, ref, type Ref } from 'vue'

import type { Album, Artist, Credentials, LibraryData, Song } from '../lib/api/types'

import { ApiError, runAureliaEffect } from '../effect'
import { clearCacheEffect, getLibraryEffect, syncLibraryEffect } from '../effect/services/api'
import { logger } from '../lib/logger'

const albumArtistsWithSongs = ref<Artist[]>([])
const allAlbums = ref<Album[]>([])
const allArtistsWithSongs = ref<Artist[]>([])
const allSongs = ref<Song[]>([])
const libraryError = ref<null | string>(null)
const libraryLoading = ref(false)

const loadLibrary = async (_credentials: Credentials): Promise<void> => {
  libraryLoading.value = true
  libraryError.value = null

  try {
    const libraryData: LibraryData = await runAureliaEffect(getLibraryEffect())
    allSongs.value = libraryData.songs
    allArtistsWithSongs.value = libraryData.artists
    albumArtistsWithSongs.value = libraryData.artists
    allAlbums.value = libraryData.albums
    libraryError.value = null
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    libraryError.value = `Failed to load library: ${errorMessage}`
  } finally {
    libraryLoading.value = false
  }
}

const syncLibrary = async (credentials: Credentials): Promise<void> => {
  logger.info('Starting library sync...')

  try {
    await runAureliaEffect(syncLibraryEffect())
    await loadLibrary(credentials)
    logger.info('Library sync completed successfully.')
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    libraryError.value = `Failed to sync library: ${errorMessage}`
    logger.error('Failed to sync library:', cause)
  }
}

const clearCache = async (credentials: Credentials): Promise<void> => {
  logger.info('Starting cache clear...')

  try {
    await runAureliaEffect(clearCacheEffect())
    await loadLibrary(credentials)
    logger.info('Cache cleared and library reloaded successfully.')
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    libraryError.value = `Failed to clear cache: ${errorMessage}`
    logger.error('Failed to clear cache:', cause)
  }
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
