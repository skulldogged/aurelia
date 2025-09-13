import { type Ref } from 'vue'
import type { Song, Credentials } from '@/bindings'
import { commands } from '@/bindings'
import { usePlayerStore } from '@/stores'
import { playerLogger } from '@/lib/logger'

export const useSongInteractions = (credentials: Ref<Credentials | null>) => {
  const playerStore = usePlayerStore()

  // Play a single song
  const playSong = (song: Song) => {
    if (!song || !song.id) {
      playerLogger.error('Invalid song passed to playSong:', song)
      return
    }

    playerStore.setCurrentSong(song)

    // Add to playlist if not already there
    if (!playerStore.playlist.find((s: Song) => s.id === song.id)) {
      playerStore.setPlaylist([...playerStore.playlist, song])
    }

    // Update current index
    const index = playerStore.playlist.findIndex(s => s.id === song.id)
    if (index !== -1) {
      playerStore.setCurrentIndex(index)
    }
  }

  // Play multiple songs (full album/playlist)
  const playSongs = (songs: Song[]) => {
    if (songs.length === 0) {
      playerLogger.warn('No songs to play')
      return
    }

    // Check for invalid songs
    const invalidSongs = songs.filter(song => !song || !song.id)
    if (invalidSongs.length > 0) {
      playerLogger.error('Found songs with invalid IDs:', invalidSongs)
    }

    // Set playlist and play first song
    playerStore.setPlaylist(songs)
    if (songs.length > 0) {
      playSong(songs[0])
    }
  }

  // Update playlist (from queue component)
  const updatePlaylist = (newPlaylist: Song[]) => {
    playerStore.setPlaylist(newPlaylist)

    // Update current index if current song is still in playlist
    if (playerStore.currentSong) {
      const index = newPlaylist.findIndex(s => s.id === playerStore.currentSong!.id)
      playerStore.setCurrentIndex(index)
    }
  }

  // Remove song from playlist
  const removeSongFromPlaylist = (song: Song) => {
    const newPlaylist = playerStore.playlist.filter(s => s.id !== song.id)
    updatePlaylist(newPlaylist)
  }

  // Handle song change from player
  const handleSongChanged = (song: Song) => {
    playerStore.setCurrentSong(song)
  }

  // Handle current song updates
  const handleUpdateCurrentSong = (song: Song | null) => {
    playerStore.setCurrentSong(song)
  }

  // Toggle favorite status
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

      if (result.status === 'error') {
        throw new Error(result.error)
      }

      // Update song objects in various places
      const newStatus = result.data

      // Update in current song
      if (playerStore.currentSong && playerStore.currentSong.id === song.id) {
        playerStore.currentSong.isFavorite = newStatus
      }

      // Update in playlist
      const playlistSong = playerStore.playlist.find(s => s.id === song.id)
      if (playlistSong) {
        playlistSong.isFavorite = newStatus
      }

    } catch (err) {
      playerLogger.error('Failed to toggle favorite status:', err)
    }
  }

  return {
    // Actions
    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,

    // Player store access
    playerStore,
  }
}
