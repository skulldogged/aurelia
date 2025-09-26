import { type Ref } from 'vue'

import type { Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { playerLogger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'
import { usePlayerStore } from '@/stores'

const playSong = (playerStore: ReturnType<typeof usePlayerStore>, song: Song): void => {
  if (!song || !song.id) {
    playerLogger.error('Invalid song passed to playSong:', song)
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
    playerLogger.warn('No songs to play')
    return
  }

  const invalidSongs = songs.filter(song => !song || !song.id)
  if (invalidSongs.length > 0)
    playerLogger.error('Found songs with invalid IDs:', invalidSongs)

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
  const newPlaylist = playerStore.playlist.filter(s => s.id !== song.id)
  updatePlaylist(playerStore, newPlaylist)
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
  credentials: Ref<Credentials | null>,
  song: Song,
): Promise<void> => {
  if (!credentials.value) {
    playerLogger.error('Cannot toggle favorite: no credentials')
    return
  }

  await withCustomState(
    () => commands.toggleFavoriteStatus(
      credentials.value!.serverUrl,
      credentials.value!.token,
      credentials.value!.userId,
      song.id,
      !song.isFavorite,
    ),
    {
      onError: error => {
        playerLogger.error('Failed to toggle favorite status:', error)
      },
      onSuccess: newStatus => {
        if (playerStore.currentSong && playerStore.currentSong.id === song.id) {
          playerStore.currentSong.isFavorite = newStatus
        }

        const playlistSong = playerStore.playlist.find((s: Song) => s.id === song.id)
        if (playlistSong) {
          playlistSong.isFavorite = newStatus
        }
      },
    },
  )
}

export interface SongInteractions {
  handleSongChanged:       (song: Song) => void
  handleUpdateCurrentSong: (song: null | Song) => void
  playerStore:             ReturnType<typeof usePlayerStore>
  playSong:                (song: Song) => void
  playSongs:               (songs: Song[]) => void
  removeSongFromPlaylist:  (song: Song) => void
  toggleFavorite:          (song: Song) => Promise<void>
  updatePlaylist:          (newPlaylist: Song[]) => void
}

export const useSongInteractions = (credentials: Ref<Credentials | null>): SongInteractions => {
  const playerStore = usePlayerStore()

  return {
    handleSongChanged:       song => handleSongChanged(playerStore, song),
    handleUpdateCurrentSong: song => handleUpdateCurrentSong(playerStore, song),
    playerStore,
    playSong:                song => playSong(playerStore, song),
    playSongs:               songs => playSongs(playerStore, songs),
    removeSongFromPlaylist:  song => removeSongFromPlaylist(playerStore, song),
    toggleFavorite:          song => toggleFavorite(playerStore, credentials, song),
    updatePlaylist:          newPlaylist => updatePlaylist(playerStore, newPlaylist),
  }
}
