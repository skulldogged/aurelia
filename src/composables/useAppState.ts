import { readonly, reactive, computed } from 'vue'
import { useAuth } from './useAuth'
import { useLibrary } from './useLibrary'
import { useNavigation } from './useNavigation'
import { usePlayerControls } from './usePlayerControls'
import { useSongInteractions } from './useSongInteractions'

// Global app state composable that provides access to all app state
export const useAppState = () => {
  // Initialize all composables
  const { authStatus, credentials, error: authError, login, logout, clearError } = useAuth()
  const {
    allSongs,
    allArtists,
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
    isFullScreenPlayerOpen,
    searchQuery,
    isSearchVisible,
    musicPlayerRef,
    handleGlobalSearch,
    toggleQueue,
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
    // Auth state
    authStatus,
    credentials,
    authError,

    // Library state
    allSongs,
    allArtists,
    allArtistsWithSongs,
    albumArtistsWithSongs,
    allAlbums,
    libraryLoading,
    libraryError,

    // Navigation state
    currentView,
    canGoBack,
    canGoForward,

    // Player controls state
    isQueueOpen,
    isFullScreenPlayerOpen,
    searchQuery,
    isSearchVisible,

    // Player store state
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
  })

  // No watchers needed - reactive system handles updates automatically

  // Actions
  const actions = {
    // Auth actions
    login,
    logout,
    clearAuthError: clearError,

    // Library actions
    loadLibrary,
    syncLibrary,
    clearCache,

    // Navigation actions
    navigateBack,
    navigateForward,
    handleNavigation,
    navigateToArtist,
    navigateToAlbum,

    // Player control actions
    handleGlobalSearch,
    toggleQueue,
    toggleFullScreenPlayer,
    toggleSearchVisibility,
    handleTogglePlayPause,
    handlePreviousSong,
    handleNextSong,
    handleToggleShuffle,
    handleToggleRepeat,
    handleSeek,

    // Song interaction actions
    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,
  }

  return {
    // State
    appState: readonly(appState),

    // Actions
    ...actions,

    // Direct access to composables for advanced usage
    playerStore,
    musicPlayerRef,
  }
}
