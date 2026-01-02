import { type Ref } from 'vue'
import { toast } from 'vue-sonner'

import type { Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { logger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'
import { useLibraryStore, usePlayerStore } from '@/stores'

const playSong = (playerStore: ReturnType<typeof usePlayerStore>, song: Song): void => {
  if (!song || !song.id) {
    logger.error('Invalid song passed to playSong:', song)
    return
  }

  playerStore.setCurrentSong(song)

  if (!playerStore.playlist.find((s: Song) => s.id === song.id))
    playerStore.setPlaylist([...playerStore.playlist, song])

  const index = playerStore.playlist.findIndex(s => s.id === song.id)
  if (index !== -1)
    playerStore.setCurrentIndex(index)
}

const playSongs = (playerStore: ReturnType<typeof usePlayerStore>, songs: Song[]): void => {
  if (songs.length === 0) {
    logger.warn('No songs to play')
    return
  }

  const invalidSongs = songs.filter(song => !song || !song.id)
  if (invalidSongs.length > 0)
    logger.error('Found songs with invalid IDs:', invalidSongs)

  playerStore.setPlaylist(songs)
  if (songs.length > 0) {
    playSong(playerStore, songs[0])
  }
}

const updatePlaylist = (playerStore: ReturnType<typeof usePlayerStore>, newPlaylist: Song[]): void => {
  playerStore.setPlaylist(newPlaylist)

  if (playerStore.currentSong) {
    const index = newPlaylist.findIndex(s => s.id === playerStore.currentSong!.id)
    playerStore.setCurrentIndex(index)
  }
}

const removeSongFromPlaylist = (playerStore: ReturnType<typeof usePlayerStore>, song: Song): void => {
  updatePlaylist(playerStore, playerStore.playlist.filter(s => s.id !== song.id))
}

const handleSongChanged = (
  playerStore: ReturnType<typeof usePlayerStore>,
  song: Song,
): void => playerStore.setCurrentSong(song)

const handleUpdateCurrentSong = (
  playerStore: ReturnType<typeof usePlayerStore>,
  song: null | Song,
): void => playerStore.setCurrentSong(song)

const toggleFavorite = async (
  playerStore: ReturnType<typeof usePlayerStore>,
  libraryStore: ReturnType<typeof useLibraryStore>,
  credentials: Ref<Credentials | null>,
  song: Song,
): Promise<void> => {
  logger.debug('toggleFavorite called', { hasCredentials: !!credentials.value, songId: song.id })

  if (!credentials.value) {
    logger.error('Cannot toggle favorite: no credentials')
    return
  }

  const oldFavoriteStatus = !!song.isFavorite
  const newFavoriteStatus = !oldFavoriteStatus

  // Optimistic update using store methods
  playerStore.updateSongFavorite(song.id, newFavoriteStatus)
  libraryStore.updateSongFavorite(song.id, newFavoriteStatus)

  await withCustomState(
    () => commands.toggleFavoriteStatus(
      credentials.value!.serverUrl,
      credentials.value!.token,
      credentials.value!.userId,
      song.id,
      newFavoriteStatus,
    ),
    {
      onError: error => {
        // Revert optimistic update on error
        playerStore.updateSongFavorite(song.id, oldFavoriteStatus)
        libraryStore.updateSongFavorite(song.id, oldFavoriteStatus)
        logger.error('Failed to toggle favorite status:', error)
        toast.error('Failed to update favorite')
      },
      onSuccess: newStatus => {
        logger.debug('Successfully toggled favorite status', { newStatus, songId: song.id })
        // Ensure final state matches server response
        playerStore.updateSongFavorite(song.id, newStatus)
        libraryStore.updateSongFavorite(song.id, newStatus)
        toast.success(newStatus ? 'Added to favorites' : 'Removed from favorites')
      },
    },
  )
}

const playInstantMix = async (playerStore: ReturnType<typeof usePlayerStore>, song: Song): Promise<void> => {
  if (!song || !song.id) {
    logger.error('Invalid song passed to playInstantMix:', song)
    return
  }

  try {
    const result = await commands.getInstantMix(song.id)
    if (result.status === 'error') {
      logger.error('Failed to get instant mix:', result.error)
      toast.error('Failed to create instant mix')
      return
    }

    const instantMixSongs = result.data
    if (instantMixSongs.length === 0) {
      logger.warn('No songs found in instant mix')
      toast.warning('No similar songs found for instant mix')
      return
    }

    // Add the original song at the beginning if it's not already there
    const songsToPlay = instantMixSongs.find(s => s.id === song.id)
      ? instantMixSongs
      : [song, ...instantMixSongs]

    playSongs(playerStore, songsToPlay)
    logger.info(`Started instant mix with ${songsToPlay.length} songs`)
    toast.success(`Started instant mix with ${songsToPlay.length} songs`)
  } catch (error) {
    logger.error('Error playing instant mix:', error)
    toast.error('Failed to create instant mix')
  }
}

export interface SongInteractions {
  handleSongChanged:       (song: Song) => void
  handleUpdateCurrentSong: (song: null | Song) => void
  playerStore:             ReturnType<typeof usePlayerStore>
  playInstantMix:          (song: Song) => Promise<void>
  playSong:                (song: Song) => void
  playSongs:               (songs: Song[]) => void
  removeSongFromPlaylist:  (song: Song) => void
  toggleFavorite:          (song: Song) => Promise<void>
  updatePlaylist:          (newPlaylist: Song[]) => void
}

export const useSongInteractions = (credentials: Ref<Credentials | null>): SongInteractions => {
  const playerStore = usePlayerStore()
  const libraryStore = useLibraryStore()

  return {
    handleSongChanged:       song => handleSongChanged(playerStore, song),
    handleUpdateCurrentSong: song => handleUpdateCurrentSong(playerStore, song),
    playerStore,
    playInstantMix:          song => playInstantMix(playerStore, song),
    playSong:                song => playSong(playerStore, song),
    playSongs:               songs => playSongs(playerStore, songs),
    removeSongFromPlaylist:  song => removeSongFromPlaylist(playerStore, song),
    toggleFavorite:          song => toggleFavorite(playerStore, libraryStore, credentials, song),
    updatePlaylist:          newPlaylist => updatePlaylist(playerStore, newPlaylist),
  }
}
