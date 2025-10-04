import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'

import type { Playlist, PlaylistCreateData, PlaylistUpdateData, Song } from '@/bindings'

import { commands } from '@/bindings'
import { useImageLoader } from '@/composables/useImageLoader'
import { appLogger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'

import { usePlayerStore } from './player'

const STORAGE_KEYS = {
  CURRENT_PLAYLIST_ID: 'playlists-current-id',
}

export type PlaylistWithMeta = Omit<Playlist, 'songs'> & {
  createdAt: Date;
  songs:     Song[];
  updatedAt: Date;
}

type PlaylistUpdateInput = Partial<PlaylistUpdateData>

export const usePlaylistStore = defineStore('playlists', () => {
  // State

  const playlists = ref<PlaylistWithMeta[]>([])
  const playlistsView = computed(() => playlists.value)
  const currentPlaylistId = ref<null | string>(null)
  const isLoading = ref(false)
  const error = ref<null | string>(null)
  const isInitialized = ref(false)

  // Initialize store data
  const initialize = async (): Promise<void> => {
    if (isInitialized.value) return

    try {
      await loadPlaylists()
      initializeCurrentPlaylist()
      isInitialized.value = true
    } catch (error) {
      appLogger.error('Failed to initialize playlist store:', error)
    }
  }

  // Computed
  const currentPlaylist = computed(() =>
    playlists.value.find(p => p.id === currentPlaylistId.value) || null,
  )

  const favoritePlaylists = computed(() =>
    playlists.value.filter(p => p.isFavorite),
  )

  // Actions
  const loadPlaylists = async (): Promise<void> => {
    await withCustomState(
      () => commands.getPlaylists(),
      {
        onError: errorString => {
          error.value = `Failed to load playlists: ${errorString}`
          appLogger.error('Failed to load playlists:', errorString)
          isLoading.value = false
        },
        onStart: () => {
          isLoading.value = true
          error.value = null
          appLogger.info('Loading playlists...')
        },
        onSuccess: loadedPlaylists => {
          playlists.value = loadedPlaylists.map(p => ({
            ...p,
            createdAt: new Date(p.dateCreated || p.dateLastSaved || Date.now()),
            songs:     p.songs ?? [],
            updatedAt: new Date(p.dateLastSaved || p.dateCreated || Date.now()),
          }))
          isLoading.value = false
          appLogger.info(`Playlists loaded successfully: ${playlists.value.length} playlists`)
          playlists.value.forEach(p => {
            appLogger.debug(`Playlist: ${p.name} (ID: ${p.id}, Songs: ${p.childCount || 0})`)
          })
        },
      },
    )
  }

  const createPlaylist = async (data: PlaylistCreateData): Promise<null | PlaylistWithMeta> => {
    let newPlaylist: null | PlaylistWithMeta = null

    await withCustomState(
      () => commands.createPlaylist(data),
      {
        onError: errorString => {
          error.value = `Failed to create playlist: ${errorString}`
          appLogger.error('Failed to create playlist:', errorString)
        },
        onStart: () => {
          error.value = null
          appLogger.info('Creating playlist:', data.name)
        },
        onSuccess: createdPlaylist => {
          newPlaylist = {
            ...createdPlaylist,
            createdAt: new Date(createdPlaylist.dateCreated || createdPlaylist.dateLastSaved || Date.now()),
            songs:     createdPlaylist.songs ?? [],
            updatedAt: new Date(createdPlaylist.dateLastSaved || createdPlaylist.dateCreated || Date.now()),
          }
          playlists.value.push(newPlaylist)
          appLogger.info('Playlist created successfully:', newPlaylist.name)
        },
      },
    )

    return newPlaylist
  }

  const buildUpdatePayload = (updates: PlaylistUpdateInput): PlaylistUpdateData => ({
    ids:        updates.ids ?? null,
    isFavorite: updates.isFavorite ?? null,
    isPublic:   updates.isPublic ?? null,
    name:       updates.name ?? null,
    songs:      updates.songs ?? null,
    userId:     updates.userId ?? null,
  })

  const updatePlaylist = async (id: string, updates: PlaylistUpdateInput): Promise<boolean> => {
    let success = false

    await withCustomState(
      () => commands.updatePlaylist(id, buildUpdatePayload(updates)),
      {
        onError: errorString => {
          error.value = `Failed to update playlist: ${errorString}`
          appLogger.error('Failed to update playlist:', errorString)
        },
        onStart: () => {
          error.value = null
          appLogger.info('Updating playlist:', id)
        },
        onSuccess: updatedPlaylist => {
          const index = playlists.value.findIndex(p => p.id === id)
          if (index !== -1) {
            // Convert date strings to Date objects and normalize field names
            playlists.value[index] = {
              ...updatedPlaylist,
              createdAt: new Date(updatedPlaylist.dateCreated || updatedPlaylist.dateLastSaved || Date.now()),
              songs:     updatedPlaylist.songs ?? [],
              updatedAt: new Date(updatedPlaylist.dateLastSaved || updatedPlaylist.dateCreated || Date.now()),
            }
          }
          success = true
          appLogger.info('Playlist updated successfully:', updatedPlaylist.name)
        },
      },
    )

    return success
  }

  const deletePlaylist = async (id: string): Promise<boolean> => {
    let success = false

    await withCustomState(
      () => commands.deletePlaylist(id),
      {
        onError: errorString => {
          error.value = `Failed to delete playlist: ${errorString}`
          appLogger.error('Failed to delete playlist:', errorString)
        },
        onStart: () => {
          error.value = null
          appLogger.info('Deleting playlist:', id)
        },
        onSuccess: () => {
          const index = playlists.value.findIndex(p => p.id === id)
          if (index !== -1) {
            const deletedName = playlists.value[index].name
            playlists.value.splice(index, 1)
            // Clear current playlist if it was deleted
            if (currentPlaylistId.value === id) {
              setCurrentPlaylist(null)
            }
            appLogger.info('Playlist deleted successfully:', deletedName)
            appLogger.info('Clearing image cache for playlist ID:', id)
          }

          // Clear the image from frontend cache
          const imageLoader = useImageLoader()
          imageLoader.clearImageFromCache(id, 'Primary')
          appLogger.info('Frontend image cache cleared for:', id)

          success = true
        },
      },
    )

    return success
  }

  const addSongsToPlaylist = async (playlistId: string, songs: Song[]): Promise<boolean> => {
    const playlist = playlists.value.find(p => p.id === playlistId)
    if (!playlist) {
      error.value = 'Playlist not found'
      return false
    }

    // Avoid duplicates
    const existingIds = new Set(playlist.songs.map(s => s.id))
    const newSongs = songs.filter(s => !existingIds.has(s.id))

    if (newSongs.length === 0) {
      return true // No new songs to add
    }

    // Combine existing song IDs with new song IDs
    const updatedIds = [...playlist.songs.map(s => s.id), ...newSongs.map(s => s.id)]
    return await updatePlaylist(playlistId, { ids: updatedIds })
  }

  const removeSongsFromPlaylist = async (playlistId: string, songIds: string[]): Promise<boolean> => {
    const playlist = playlists.value.find(p => p.id === playlistId)
    if (!playlist) {
      error.value = 'Playlist not found'
      return false
    }

    const updatedSongs = playlist.songs.filter(s => !songIds.includes(s.id))
    return await updatePlaylist(playlistId, { songs: updatedSongs })
  }

  const reorderPlaylistSongs = async (
    playlistId: string,
    newOrder: Song[],
  ): Promise<boolean> =>
    await updatePlaylist(playlistId, { songs: newOrder })

  const setCurrentPlaylist = (playlistId: null | string): void => {
    currentPlaylistId.value = playlistId
    if (playlistId) {
      localStorage.setItem(STORAGE_KEYS.CURRENT_PLAYLIST_ID, playlistId)
    } else {
      localStorage.removeItem(STORAGE_KEYS.CURRENT_PLAYLIST_ID)
    }
  }

  const togglePlaylistFavorite = async (playlistId: string): Promise<boolean> => {
    const playlist = playlists.value.find(p => p.id === playlistId)
    if (!playlist) {
      error.value = 'Playlist not found'
      return false
    }

    return await updatePlaylist(playlistId, { isFavorite: !playlist.isFavorite })
  }

  const clearError = (): void => {
    error.value = null
  }

  const getPlaylistItems = async (playlistId: string): Promise<Song[]> => {
    let items: Song[] = []

    await withCustomState(
      () => commands.getPlaylistItems(playlistId),
      {
        onError: errorString => {
          error.value = `Failed to get playlist items: ${errorString}`
          appLogger.error('Failed to get playlist items:', errorString)
        },
        onStart: () => {
          error.value = null
          appLogger.info('Getting playlist items for:', playlistId)
        },
        onSuccess: songs => {
          items = songs
          appLogger.info('Retrieved playlist items successfully')
        },
      },
    )

    return items
  }

  const playPlaylist = async (playlistId: string, shuffle = false): Promise<void> => {
    try {
      const songs = await getPlaylistItems(playlistId)
      if (songs.length > 0) {
        const playerStore = usePlayerStore()

        // Apply shuffle if requested
        const songsToPlay = [...songs]
        if (shuffle) {
          for (let i = songsToPlay.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1))
            ;[songsToPlay[i], songsToPlay[j]] = [songsToPlay[j], songsToPlay[i]]
          }
        }

        playerStore.setPlaylist(songsToPlay)
        playerStore.setCurrentIndex(0)
        playerStore.play()
        setCurrentPlaylist(playlistId)

        appLogger.info(`Started playing playlist: ${playlistId}`)
      }
    } catch (error) {
      appLogger.error('Failed to play playlist:', error)
    }
  }

  const initializeCurrentPlaylist = (): void => {
    const storedId = localStorage.getItem(STORAGE_KEYS.CURRENT_PLAYLIST_ID)
    if (storedId && playlists.value.some(p => p.id === storedId)) {
      currentPlaylistId.value = storedId
    }
  }

  return {
    // Actions
    addSongsToPlaylist,
    clearError,
    createPlaylist,
    // Computed
    currentPlaylist,

    currentPlaylistId: readonly(currentPlaylistId),
    deletePlaylist,

    error:     readonly(error),
    favoritePlaylists,
    getPlaylistItems,
    initialize,
    initializeCurrentPlaylist,
    isLoading: readonly(isLoading),
    loadPlaylists,
    // State
    playlists: playlistsView,
    playPlaylist,
    removeSongsFromPlaylist,
    reorderPlaylistSongs,
    setCurrentPlaylist,
    togglePlaylistFavorite,
    updatePlaylist,
  }
})
