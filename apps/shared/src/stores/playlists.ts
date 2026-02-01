import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'
import { toast } from 'vue-sonner'

import type { Playlist, PlaylistCreateData, PlaylistUpdateData, Song } from '../lib/api/types'

import { useImageLoader } from '../composables/useImageLoader'
import { getApiClient } from '../index'
import { logger } from '../lib/logger'
import { withCustomState } from '../lib/result'
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
      logger.error('Failed to initialize playlist store:', error)
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
    await withCustomState<Playlist[], string>(
      () => getApiClient().getPlaylists(),
      {
        onError: errorString => {
          error.value = `Failed to load playlists: ${errorString}`
          logger.error('Failed to load playlists:', errorString)
          isLoading.value = false
        },
        onStart: () => {
          isLoading.value = true
          error.value = null
          logger.info('Loading playlists...')
        },
        onSuccess: loadedPlaylists => {
          playlists.value = loadedPlaylists.map(p => ({
            ...p,
            createdAt: new Date(p.dateCreated || p.dateLastSaved || Date.now()),
            songs:     p.songs ?? [],
            updatedAt: new Date(p.dateLastSaved || p.dateCreated || Date.now()),
          }))
          isLoading.value = false
          logger.info(`Playlists loaded successfully: ${playlists.value.length} playlists`)
          playlists.value.forEach(p => {
            logger.debug(`Playlist: ${p.name} (ID: ${p.id}, Songs: ${p.childCount || 0})`)
          })
        },
      },
    )
  }

  const createPlaylist = async (data: PlaylistCreateData): Promise<null | PlaylistWithMeta> => {
    let newPlaylist: null | PlaylistWithMeta = null

    await withCustomState<Playlist, string>(
      () => getApiClient().createPlaylist(data),
      {
        onError: errorString => {
          error.value = `Failed to create playlist: ${errorString}`
          logger.error('Failed to create playlist:', errorString)
          toast.error('Failed to create playlist')
        },
        onStart: () => {
          error.value = null
          logger.info('Creating playlist:', data.name)
        },
        onSuccess: createdPlaylist => {
          newPlaylist = {
            ...createdPlaylist,
            createdAt: new Date(createdPlaylist.dateCreated || createdPlaylist.dateLastSaved || Date.now()),
            songs:     createdPlaylist.songs ?? [],
            updatedAt: new Date(createdPlaylist.dateLastSaved || createdPlaylist.dateCreated || Date.now()),
          }
          playlists.value.push(newPlaylist)
          logger.info('Playlist created successfully:', newPlaylist.name)
          toast.success(`Created playlist "${newPlaylist.name}"`)
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

    await withCustomState<Playlist, string>(
      () => getApiClient().updatePlaylist(id, buildUpdatePayload(updates)),
      {
        onError: errorString => {
          error.value = `Failed to update playlist: ${errorString}`
          logger.error('Failed to update playlist:', errorString)
        },
        onStart: () => {
          error.value = null
          logger.info('Updating playlist:', id)
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
          logger.info('Playlist updated successfully:', updatedPlaylist.name)
        },
      },
    )

    return success
  }

  const deletePlaylist = async (id: string): Promise<boolean> => {
    let success = false
    const playlistName = playlists.value.find(p => p.id === id)?.name

    await withCustomState<void, string>(
      () => getApiClient().deletePlaylist(id),
      {
        onError: errorString => {
          error.value = `Failed to delete playlist: ${errorString}`
          logger.error('Failed to delete playlist:', errorString)
          toast.error('Failed to delete playlist')
        },
        onStart: () => {
          error.value = null
          logger.info('Deleting playlist:', id)
        },
        onSuccess: async () => {
          const index = playlists.value.findIndex(p => p.id === id)
          if (index !== -1) {
            const deletedName = playlists.value[index].name
            playlists.value.splice(index, 1)
            // Clear current playlist if it was deleted
            if (currentPlaylistId.value === id) {
              setCurrentPlaylist(null)
            }
            logger.info('Playlist deleted successfully:', deletedName)
            logger.info('Clearing image cache for playlist ID:', id)
          }

          // Clear the image from frontend cache
          const imageLoader = useImageLoader()
          await imageLoader.clearImageFromCache(id, 'Primary')
          logger.info('Frontend image cache cleared for:', id)

          success = true
          toast.success(playlistName ? `Deleted "${playlistName}"` : 'Playlist deleted')
        },
      },
    )

    return success
  }

  const addSongsToPlaylist = async (playlistId: string, songs: Song[]): Promise<boolean> => {
    const playlist = playlists.value.find(p => p.id === playlistId)
    if (!playlist) {
      error.value = 'Playlist not found'
      toast.error('Playlist not found')
      return false
    }

    // Fetch current playlist items to get the actual song list
    const currentSongs = await getPlaylistItems(playlistId)

    // Avoid duplicates
    const existingIds = new Set(currentSongs.map(s => s.id))
    const newSongs = songs.filter(s => !existingIds.has(s.id))

    if (newSongs.length === 0) {
      toast.info('Songs already in playlist')
      return true // No new songs to add
    }

    // Combine existing song IDs with new song IDs
    const updatedIds = [...currentSongs.map(s => s.id), ...newSongs.map(s => s.id)]
    const success = await updatePlaylist(playlistId, { ids: updatedIds })

    if (success) {
      const songText = newSongs.length === 1 ? 'song' : 'songs'
      toast.success(`Added ${newSongs.length} ${songText} to "${playlist.name}"`)
    }

    return success
  }

  const removeSongsFromPlaylist = async (playlistId: string, songIds: string[]): Promise<boolean> => {
    const playlist = playlists.value.find(p => p.id === playlistId)
    if (!playlist) {
      error.value = 'Playlist not found'
      toast.error('Playlist not found')
      return false
    }

    const updatedSongs = playlist.songs.filter(s => !songIds.includes(s.id))
    const success = await updatePlaylist(playlistId, { songs: updatedSongs })

    if (success) {
      const songText = songIds.length === 1 ? 'song' : 'songs'
      toast.success(`Removed ${songIds.length} ${songText} from playlist`)
    }

    return success
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

    const newFavoriteState = !playlist.isFavorite
    const success = await updatePlaylist(playlistId, { isFavorite: newFavoriteState })

    if (success) {
      toast.success(newFavoriteState ? 'Added to favorites' : 'Removed from favorites')
    }

    return success
  }

  const clearError = (): void => {
    error.value = null
  }

  const getPlaylistItems = async (playlistId: string): Promise<Song[]> => {
    let items: Song[] = []

    await withCustomState<Song[], string>(
      () => getApiClient().getPlaylistItems(playlistId),
      {
        onError: errorString => {
          error.value = `Failed to get playlist items: ${errorString}`
          logger.error('Failed to get playlist items:', errorString)
        },
        onStart: () => {
          error.value = null
          logger.info('Getting playlist items for:', playlistId)
        },
        onSuccess: songs => {
          items = songs
          logger.info('Retrieved playlist items successfully')
        },
      },
    )

    return items
  }

  const playPlaylist = async (playlistId: string, shuffle = false): Promise<void> => {
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

      logger.info(`Started playing playlist: ${playlistId}`)
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
