import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'
import { toast } from 'vue-sonner'

import type { Album, Artist, Credentials, Song } from '../generated'

import { ApiError } from '../effect/errors'
import { runAureliaEffect } from '../effect/runtime'
import { clearCacheEffect, getLibraryEffect, syncLibraryEffect } from '../effect/services/api'
import { logger } from '../lib/logger'
import { useHomeStore } from './home'

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
  const loadLibrary = async (): Promise<void> => {
    if (isLoaded.value) {
      logger.info('Library already loaded, skipping.')
      return
    }

    isLoading.value = true
    error.value = null
    logger.info('loadLibrary: Loading library data...')
    const startTime = Date.now()

    const maxRetries = 10
    let retryDelay = 100

    try {
      for (let attempt = 0; attempt < maxRetries; attempt++) {
        const libraryData = await runAureliaEffect(getLibraryEffect())
        const { albums, artists, songs } = libraryData

        // Hybrid lazy-load: songs are synced, but albums/artists may be empty
        // (they're fetched on-demand when user visits detail pages)
        if (songs.length === 0 && attempt < maxRetries - 1) {
          // If we have no songs at all, database might still be initializing
          logger.warn(
            `No songs loaded (attempt ${attempt + 1}/${maxRetries}). Retrying in ${retryDelay}ms...`,
          )
          await new Promise(resolve => setTimeout(resolve, retryDelay))
          retryDelay = Math.min(retryDelay * 2, 1000)
          continue
        }

        allSongs.value = songs

        // Post-process data to link songs to albums and artists
        const albumMap = new Map<string, Song[]>()
        for (const song of songs)
          if (song.albumId) {
            if (!albumMap.has(song.albumId))
              albumMap.set(song.albumId, [])

            albumMap.get(song.albumId)!.push(song)
          }

        for (const album of albums)
          if (album.id)
            album.songs = albumMap.get(album.id) || []

        // Map songs to artists using both albumArtists and artistIds
        // Some artists only appear as contributing artists (compilations, features)
        const artistMap = new Map<string, Song[]>()
        for (const song of songs) {
          // Add from albumArtists (primary album artists)
          if (song.albumArtists)
            for (const pair of song.albumArtists)
              if (pair.id) {
                if (!artistMap.has(pair.id))
                  artistMap.set(pair.id, [])

                artistMap.get(pair.id)!.push(song)
              }

          // Add from artistIds (track-level artists, includes features)
          if (song.artistIds)
            for (const artistId of song.artistIds) {
              if (!artistMap.has(artistId))
                artistMap.set(artistId, [])

              // Avoid duplicates - check if song already added
              const existing = artistMap.get(artistId)!
              if (!existing.find(s => s.id === song.id))
                existing.push(song)
            }
        }

        // Assign mapped songs to artists
        for (const artist of artists) {
          if (artist.id)
            artist.songs = artistMap.get(artist.id) || []
        }

        allArtistsWithSongs.value = artists
        allAlbums.value = albums
        isLoaded.value = true
        logger.info(
          `loadLibrary: Loaded ${songs.length} songs, ${artists.length} artists, and ${albums.length} albums.`,
        )
        return
      }
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to load library: ${errorMessage}`
      logger.error('Failed to load library:', errorMessage)
      return
    } finally {
      isLoading.value = false
      logger.info(`loadLibrary: Completed in ${Date.now() - startTime}ms`)
    }
  }

  const syncLibrary = async (_credentials: Credentials): Promise<void> => {
    logger.info('Starting library sync...')
    const toastId = toast.loading('Syncing library...')

    try {
      await runAureliaEffect(syncLibraryEffect())

      // Reset loaded state to force reload
      isLoaded.value = false
      await loadLibrary()
      logger.info('Library sync completed successfully.')
      toast.success('Library synced successfully', { id: toastId })
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to sync library: ${errorMessage}`
      logger.error('Failed to sync library:', errorMessage)
      toast.error('Failed to sync library', { id: toastId })
    }
  }

  const clearCache = async (_credentials: Credentials): Promise<void> => {
    logger.info('Starting cache clear...')
    const toastId = toast.loading('Clearing cache...')

    // Reset home data when clearing cache
    const homeStore = useHomeStore()
    homeStore.resetHomeData()

    try {
      await runAureliaEffect(clearCacheEffect())

      // Reset loaded state to force reload
      isLoaded.value = false
      await loadLibrary()
      logger.info('Cache clear completed successfully.')
      toast.success('Cache cleared successfully', { id: toastId })
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to clear cache: ${errorMessage}`
      logger.error('Failed to clear cache:', errorMessage)
      toast.error('Failed to clear cache', { id: toastId })
    }
  }

  const clearData = (): void => {
    allSongs.value = []
    allArtistsWithSongs.value = []
    albumArtistsWithSongs.value = []
    allAlbums.value = []
    isLoaded.value = false
    error.value = null
  }

  const updateSongFavorite = (songId: string, isFavorite: boolean): void => {
    const index = allSongs.value.findIndex(s => s.id === songId)
    if (index !== -1) {
      allSongs.value = [
        ...allSongs.value.slice(0, index),
        { ...allSongs.value[index], isFavorite },
        ...allSongs.value.slice(index + 1),
      ]
    }
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
    updateSongFavorite,
  }
})
