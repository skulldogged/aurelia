import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'
import { toast } from 'vue-sonner'

import type { Album, Artist, Credentials, Song } from '@/lib/api/bindings'

import { commands } from '@/lib/api/bindings'
import { logger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'
import { useHomeStore } from '@/stores/home'

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

    for (let attempt = 0; attempt < maxRetries; attempt++) {
      const result = await commands.getLibrary()

      if (result.status === 'ok') {
        const { albums, artists, songs } = result.data

        // Check for suspicious data: if we have songs but no albums, the backend might still be loading
        const hasSongs = songs.length > 0
        const hasAlbums = albums.length > 0
        const dataIncomplete = hasSongs && !hasAlbums

        if (dataIncomplete) {
          const attemptNumber = attempt + 1
          if (attemptNumber >= maxRetries) {
            logger.error(
              `Library data appears incomplete after ${maxRetries} retries (${songs.length} songs but 0 albums). ` +
              'This may indicate a database issue. Please try syncing your library.',
            )
            error.value = 'Library data incomplete. Please sync your library from settings.'
            isLoading.value = false
            return
          }

          logger.warn(
            `Library data incomplete (attempt ${attemptNumber}/${maxRetries}): ` +
            `${songs.length} songs, ${albums.length} albums. Retrying in ${retryDelay}ms...`,
          )
          await new Promise(resolve => setTimeout(resolve, retryDelay))
          retryDelay = Math.min(retryDelay * 2, 1000) // Cap at 1 second
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
        isLoading.value = false
        logger.info(`loadLibrary: Completed in ${Date.now() - startTime}ms`)
        return
      }

      error.value = `Failed to load library: ${result.error}`
      logger.error('Failed to load library:', result.error)
      isLoading.value = false
      return
    }

    isLoading.value = false
    logger.info(`loadLibrary: Completed in ${Date.now() - startTime}ms`)
  }

  const syncLibrary = async (credentials: Credentials): Promise<void> => {
    logger.info('Starting library sync...')
    const toastId = toast.loading('Syncing library...')

    await withCustomState(
      () => commands.syncLibrary(credentials.serverUrl, credentials.token),
      {
        onError: errorString => {
          const errorMessage = `Failed to sync library: ${errorString}`
          error.value = errorMessage
          logger.error('Failed to sync library:', errorString)
          toast.error('Failed to sync library', { id: toastId })
        },
        onSuccess: async () => {
          // Reset loaded state to force reload
          isLoaded.value = false
          await loadLibrary()
          logger.info('Library sync completed successfully.')
          toast.success('Library synced successfully', { id: toastId })
        },
      },
    )
  }

  const clearCache = async (credentials: Credentials): Promise<void> => {
    logger.info('Starting cache clear...')
    const toastId = toast.loading('Clearing cache...')

    // Reset home data when clearing cache
    const homeStore = useHomeStore()
    homeStore.resetHomeData()

    await withCustomState(
      () => commands.clearCache(credentials.serverUrl, credentials.token),
      {
        onError: errorString => {
          const errorMessage = `Failed to clear cache: ${errorString}`
          error.value = errorMessage
          logger.error('Failed to clear cache:', errorString)
          toast.error('Failed to clear cache', { id: toastId })
        },
        onSuccess: async () => {
          // Reset loaded state to force reload
          isLoaded.value = false
          await loadLibrary()
          logger.info('Cache clear completed successfully.')
          toast.success('Cache cleared successfully', { id: toastId })
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
