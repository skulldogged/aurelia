import { type Ref } from 'vue'

import type { Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { playerLogger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'
import { useLibraryStore, usePlayerStore } from '@/stores'

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
  playerLogger.debug('toggleFavorite called', { hasCredentials: !!credentials.value, songId: song.id })

  if (!credentials.value) {
    playerLogger.error('Cannot toggle favorite: no credentials')
    return
  }

  // Optimistic update
  const oldFavoriteStatus = song.isFavorite
  song.isFavorite = !oldFavoriteStatus

  await withCustomState(
    () => commands.toggleFavoriteStatus(
      credentials.value!.serverUrl,
      credentials.value!.token,
      credentials.value!.userId,
      song.id,
      !!song.isFavorite,
    ),
    {
      onError: error => {
        // Revert optimistic update on error
        song.isFavorite = oldFavoriteStatus
        playerLogger.error('Failed to toggle favorite status:', error)
      },
      onSuccess: newStatus => {
        playerLogger.debug('Successfully toggled favorite status', { newStatus, songId: song.id })

        // Update all references to ensure consistency
        song.isFavorite = newStatus

        if (playerStore.currentSong && playerStore.currentSong.id === song.id) {
          playerStore.currentSong.isFavorite = newStatus
          playerLogger.debug('Updated current song favorite status')
        }

        const playlistSong = playerStore.playlist.find((s: Song) => s.id === song.id)
        if (playlistSong) {
          playlistSong.isFavorite = newStatus
          playerLogger.debug('Updated playlist song favorite status')
        }

        const librarySong = libraryStore.allSongs.find((s: Song) => s.id === song.id)
        if (librarySong) {
          librarySong.isFavorite = newStatus
          playerLogger.debug('Updated library song favorite status')
        } else {
          const count = libraryStore.allSongs.length
          playerLogger.debug('Library song not found', { count, songId: song.id })
        }
      },
    },
  )
}

const playInstantMix = async (playerStore: ReturnType<typeof usePlayerStore>, song: Song): Promise<void> => {
  if (!song || !song.id) {
    playerLogger.error('Invalid song passed to playInstantMix:', song)
    return
  }

  try {
    const result = await commands.getInstantMix(song.id)
    if (result.status === 'error') {
      playerLogger.error('Failed to get instant mix:', result.error)
      return
    }

    const instantMixSongs = result.data
    if (instantMixSongs.length === 0) {
      playerLogger.warn('No songs found in instant mix')
      return
    }

    // Add the original song at the beginning if it's not already there
    const songsToPlay = instantMixSongs.find(s => s.id === song.id)
      ? instantMixSongs
      : [song, ...instantMixSongs]

    playSongs(playerStore, songsToPlay)
    playerLogger.info(`Started instant mix with ${songsToPlay.length} songs`)
  } catch (error) {
    playerLogger.error('Error playing instant mix:', error)
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
