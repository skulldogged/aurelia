import { readonly, reactive, computed } from 'vue'
import { useAuth } from './useAuth'
import { useLibrary } from './useLibrary'
import { useNavigation } from './useNavigation'
import { usePlayerControls } from './usePlayerControls'
import { useSongInteractions } from './useSongInteractions'
import { useWebAudioPlayer } from './useWebAudioPlayer'

export const useAppState = () => {
  const { authStatus, credentials, error: authError, login, logout, clearError } = useAuth()
  const {
    allSongs,
    allArtistsWithSongs,
    albumArtistsWithSongs,
    allAlbums,
    libraryLoading,
    libraryError,
    loadLibrary,
    syncLibrary,
    clearCache,
  } = useLibrary()
  const {
    currentView,
    canGoBack,
    canGoForward,
    navigateBack,
    navigateForward,
    handleNavigation,
    navigateToArtist,
    navigateToAlbum,
  } = useNavigation()
  const {
    isQueueOpen,
    isEqualizerOpen,
    isFullScreenPlayerOpen,
    searchQuery,
    isSearchVisible,
    musicPlayerRef,
    handleGlobalSearch,
    toggleQueue,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleSearchVisibility,
    handleTogglePlayPause,
    handlePreviousSong,
    handleNextSong,
    handleToggleShuffle,
    handleToggleRepeat,
    handleSeek,
    playerStore,
  } = usePlayerControls()
  // Song interactions - reactive to credentials
  const {
    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,
  } = useSongInteractions(credentials)

  // Simplified state - just expose the reactive refs directly
  const appState = reactive({
    authStatus,
    credentials,
    authError,

    allSongs,
    allArtistsWithSongs,
    albumArtistsWithSongs,
    allAlbums,
    libraryLoading,
    libraryError,

    currentView,
    canGoBack,
    canGoForward,

    isQueueOpen,
    isEqualizerOpen,
    isFullScreenPlayerOpen,
    searchQuery,
    isSearchVisible,

    currentSong: computed(() => playerStore.currentSong),
    playlist:    computed(() => playerStore.playlist),
    isPlaying:   computed(() => playerStore.isPlaying),
    currentTime: computed(() => playerStore.currentTime),
    duration:    computed(() => playerStore.duration),
    volume:      computed(() => playerStore.volume),
    isShuffled:  computed(() => playerStore.isShuffled),
    repeatMode:  computed(() => playerStore.repeatMode),
    progress:    computed(() => playerStore.progress),
    hasNext:     computed(() => playerStore.hasNext),
    hasPrevious: computed(() => playerStore.hasPrevious),
    eqEnabled:   computed(() => playerStore.eqEnabled),
    eqBands:     computed(() => playerStore.eqBands),
  })

  // No watchers needed - reactive system handles updates automatically

  const actions = {
    login,
    logout,
    clearAuthError: clearError,

    loadLibrary,
    syncLibrary,
    clearCache,

    navigateBack,
    navigateForward,
    handleNavigation,
    navigateToArtist,
    navigateToAlbum,

    handleGlobalSearch,
    toggleQueue,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleSearchVisibility,
    handleTogglePlayPause,
    handlePreviousSong,
    handleNextSong,
    handleToggleShuffle,
    handleToggleRepeat,
    handleSeek,

    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,

    // EQ actions - these need to call both the WebAudio player and the store
    setEQEnabled: (enabled: boolean) => {
      const webAudioPlayer = useWebAudioPlayer()
      webAudioPlayer.setEQEnabled(enabled)
      playerStore.setEQEnabled(enabled)
    },
    setEQBandGain: (bandIndex: number, gain: number) => {
      const webAudioPlayer = useWebAudioPlayer()
      webAudioPlayer.setEQBandGain(bandIndex, gain)
      playerStore.setEQBandGain(bandIndex, gain)
    },
    resetEQ: () => {
      const webAudioPlayer = useWebAudioPlayer()
      webAudioPlayer.resetEQ()
      playerStore.resetEQ()
    },
  }

  return {
    appState: readonly(appState),

    ...actions,

    // Direct access to composables for advanced usage
    playerStore,
    musicPlayerRef,
  }
}
