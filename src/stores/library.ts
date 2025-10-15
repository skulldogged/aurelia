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
  const loadLibrary = async (): Promise<void> => {
    if (isLoaded.value) {
      appLogger.info('Library already loaded, skipping.');
      return;
    }

    isLoading.value = true;
    error.value = null;
    appLogger.info('loadLibrary: Loading library data...');
    console.time('loadLibrary');

    const result = await commands.getLibrary();

    if (result.status === 'ok') {
      const { songs, artists, albums } = result.data;
      allSongs.value = songs;

      // Post-process data to link songs to albums and artists
      const albumMap = new Map<string, Song[]>();
      for (const song of songs) {
        if (song.albumId) {
          if (!albumMap.has(song.albumId)) {
            albumMap.set(song.albumId, []);
          }
          albumMap.get(song.albumId)!.push(song);
        }
      }
      for (const album of albums) {
        if (album.id) {
          album.songs = albumMap.get(album.id) || [];
        }
      }

      const artistMap = new Map<string, Song[]>();
      for (const song of songs) {
        if (song.artistIds) {
          for (const artistId of song.artistIds) {
            if (!artistMap.has(artistId)) {
              artistMap.set(artistId, []);
            }
            artistMap.get(artistId)!.push(song);
          }
        }
      }
      for (const artist of artists) {
        if (artist.id) {
          artist.songs = artistMap.get(artist.id) || [];
        }
      }

      allArtistsWithSongs.value = artists;
      allAlbums.value = albums;
      isLoaded.value = true;
      appLogger.info(`loadLibrary: Loaded ${songs.length} songs, ${artists.length} artists, and ${albums.length} albums.`);
    } else {
      error.value = `Failed to load library: ${result.error}`;
      appLogger.error('Failed to load library:', result.error);
    }

    isLoading.value = false;
    console.timeEnd('loadLibrary');
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
          await loadLibrary()
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
          await loadLibrary()
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
