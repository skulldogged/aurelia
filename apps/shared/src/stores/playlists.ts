import { defineStore } from 'pinia'
import { computed, readonly, ref } from 'vue'
import { toast } from 'vue-sonner'

import type { Playlist, PlaylistCreateData, PlaylistUpdateData, Song } from '../lib/api/types'

import { ApiError } from '../effect/errors'
import {
  createPlaylistEffect,
  deletePlaylistEffect,
  getPlaylistItemsEffect,
  getPlaylistsEffect,
  updatePlaylistEffect,
} from '../effect/services/api'
import { runAureliaEffect } from '../effect/runtime'
import { useImageLoader } from '../composables/useImageLoader'
import { logger } from '../lib/logger'
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

const toPlaylistWithMeta = (playlist: Playlist): PlaylistWithMeta => ({
  ...playlist,
  createdAt: new Date(playlist.dateCreated || playlist.dateLastSaved || Date.now()),
  songs:     playlist.songs ?? [],
  updatedAt: new Date(playlist.dateLastSaved || playlist.dateCreated || Date.now()),
})

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
    isLoading.value = true
    error.value = null
    logger.info('Loading playlists...')

    try {
      const loadedPlaylists = await runAureliaEffect(getPlaylistsEffect())
      playlists.value = loadedPlaylists.map(toPlaylistWithMeta)
      logger.info(`Playlists loaded successfully: ${playlists.value.length} playlists`)
      playlists.value.forEach(p => {
        logger.debug(`Playlist: ${p.name} (ID: ${p.id}, Songs: ${p.childCount || 0})`)
      })
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to load playlists: ${errorMessage}`
      logger.error('Failed to load playlists:', errorMessage)
    } finally {
      isLoading.value = false
    }
  }

  const createPlaylist = async (data: PlaylistCreateData): Promise<null | PlaylistWithMeta> => {
    error.value = null
    logger.info('Creating playlist:', data.name)

    try {
      const createdPlaylist = await runAureliaEffect(createPlaylistEffect(data))
      const newPlaylist = toPlaylistWithMeta(createdPlaylist)
      playlists.value.push(newPlaylist)
      logger.info('Playlist created successfully:', newPlaylist.name)
      toast.success(`Created playlist "${newPlaylist.name}"`)
      return newPlaylist
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to create playlist: ${errorMessage}`
      logger.error('Failed to create playlist:', errorMessage)
      toast.error('Failed to create playlist')
      return null
    }
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
    error.value = null
    logger.info('Updating playlist:', id)

    try {
      const updatedPlaylist = await runAureliaEffect(updatePlaylistEffect(id, buildUpdatePayload(updates)))
      const index = playlists.value.findIndex(p => p.id === id)
      if (index !== -1) {
        // Convert date strings to Date objects and normalize field names
        playlists.value[index] = toPlaylistWithMeta(updatedPlaylist)
      }
      logger.info('Playlist updated successfully:', updatedPlaylist.name)
      return true
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to update playlist: ${errorMessage}`
      logger.error('Failed to update playlist:', errorMessage)
      return false
    }
  }

  const deletePlaylist = async (id: string): Promise<boolean> => {
    const playlistName = playlists.value.find(p => p.id === id)?.name

    error.value = null
    logger.info('Deleting playlist:', id)

    try {
      await runAureliaEffect(deletePlaylistEffect(id))

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

      // Clear the image from frontend cache (best-effort)
      const imageLoader = useImageLoader()
      await imageLoader.clearImageFromCache(id, 'Primary').catch(cacheError => {
        logger.warn('Failed to clear playlist image from frontend cache:', cacheError)
      })
      logger.info('Frontend image cache cleared for:', id)

      toast.success(playlistName ? `Deleted "${playlistName}"` : 'Playlist deleted')
      return true
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to delete playlist: ${errorMessage}`
      logger.error('Failed to delete playlist:', errorMessage)
      toast.error('Failed to delete playlist')
      return false
    }
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
    error.value = null
    logger.info('Getting playlist items for:', playlistId)

    try {
      const songs = await runAureliaEffect(getPlaylistItemsEffect(playlistId))
      logger.info('Retrieved playlist items successfully')
      return songs
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      error.value = `Failed to get playlist items: ${errorMessage}`
      logger.error('Failed to get playlist items:', errorMessage)
      return []
    }
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
