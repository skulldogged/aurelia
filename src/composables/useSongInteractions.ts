import { type Ref } from 'vue'
import type { Song, Credentials } from '@/bindings'
import { commands } from '@/bindings'
import { usePlayerStore } from '@/stores'
import { playerLogger } from '@/lib/logger'

export const useSongInteractions = (credentials: Ref<Credentials | null>) => {
  const playerStore = usePlayerStore()

  const playSong = (song: Song) => {
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

  const playSongs = (songs: Song[]) => {
    if (songs.length === 0) {
      playerLogger.warn('No songs to play')
      return
    }

    const invalidSongs = songs.filter(song => !song || !song.id)
    if (invalidSongs.length > 0)
      playerLogger.error('Found songs with invalid IDs:', invalidSongs)

    playerStore.setPlaylist(songs)
    if (songs.length > 0) {
      playSong(songs[0])
    }
  }

  const updatePlaylist = (newPlaylist: Song[]) => {
    playerStore.setPlaylist(newPlaylist)

    if (playerStore.currentSong) {
      const index = newPlaylist.findIndex(s => s.id === playerStore.currentSong!.id)
      playerStore.setCurrentIndex(index)
    }
  }

  const removeSongFromPlaylist = (song: Song) => {
    const newPlaylist = playerStore.playlist.filter(s => s.id !== song.id)
    updatePlaylist(newPlaylist)
  }

  const handleSongChanged = (song: Song) => playerStore.setCurrentSong(song)

  const handleUpdateCurrentSong = (song: Song | null) => playerStore.setCurrentSong(song)

  const toggleFavorite = async (song: Song) => {
    if (!credentials.value) {
      playerLogger.error('Cannot toggle favorite: no credentials')
      return
    }

    try {
      const result = await commands.toggleFavoriteStatus(
        credentials.value.serverUrl,
        credentials.value.token,
        credentials.value.userId,
        song.id,
        !song.isFavorite,
      )

      if (result.status === 'error')
        throw new Error(result.error)

      const newStatus = result.data

      if (playerStore.currentSong && playerStore.currentSong.id === song.id)
        playerStore.currentSong.isFavorite = newStatus

      const playlistSong = playerStore.playlist.find(s => s.id === song.id)

      if (playlistSong)
        playlistSong.isFavorite = newStatus
    } catch (err) {
      playerLogger.error('Failed to toggle favorite status:', err)
    }
  }

  return {
    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,

    playerStore,
  }
}
