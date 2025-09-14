import { ref, readonly } from 'vue'
import { usePlayerStore } from '@/stores'

// Interface for MusicPlayer component methods
interface MusicPlayerRef {
  togglePlayPause: () => void
  previousSong:    () => void
  nextSong:        () => void
  toggleShuffle:   () => void
  toggleRepeat:    () => void
  onSeek:          (value: number[]) => void
}

export const usePlayerControls = () => {
  const playerStore = usePlayerStore()

  // UI state for player controls
  const isQueueOpen = ref(false)
  const isEqualizerOpen = ref(false)
  const isFullScreenPlayerOpen = ref(false)
  const searchQuery = ref('')
  const isSearchVisible = ref(false)

  // Music player component ref (will be set by component)
  const musicPlayerRef = ref<MusicPlayerRef | null>(null)

  // Search handling
  const handleGlobalSearch = (query: string) => {
    searchQuery.value = query
    isSearchVisible.value = true
  }

  // Queue management
  const toggleQueue = () => {
    if (isEqualizerOpen.value) {
      isEqualizerOpen.value = false
    }
    isQueueOpen.value = !isQueueOpen.value
  }

  // Equalizer management
  const toggleEqualizer = () => {
    if (isQueueOpen.value) {
      isQueueOpen.value = false
    }
    const newState = !isEqualizerOpen.value
    isEqualizerOpen.value = newState
    // Also toggle the EQ enabled state in the player store
    playerStore.setEQEnabled(newState)
  }

  // Fullscreen player
  const toggleFullScreenPlayer = () => {
    isFullScreenPlayerOpen.value = !isFullScreenPlayerOpen.value
  }

  // Player controls
  const handleTogglePlayPause = () => {
    musicPlayerRef.value?.togglePlayPause()
  }

  const handlePreviousSong = () => {
    playerStore.previousSong()
    musicPlayerRef.value?.previousSong()
  }

  const handleNextSong = () => {
    playerStore.nextSong()
    musicPlayerRef.value?.nextSong()
  }

  const handleSeek = (value: number) => {
    musicPlayerRef.value?.onSeek([value])
  }

  // Toggle search visibility
  const toggleSearchVisibility = (visible: boolean) => {
    isSearchVisible.value = visible
  }

  return {
    // State
    isQueueOpen:            readonly(isQueueOpen),
    isEqualizerOpen:        readonly(isEqualizerOpen),
    isFullScreenPlayerOpen: readonly(isFullScreenPlayerOpen),
    searchQuery:            readonly(searchQuery),
    isSearchVisible:        readonly(isSearchVisible),
    musicPlayerRef,

    // Actions
    handleGlobalSearch,
    toggleQueue,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleSearchVisibility,
    handleTogglePlayPause,
    handlePreviousSong,
    handleNextSong,
    handleToggleShuffle: playerStore.toggleShuffle,
    handleToggleRepeat:  playerStore.cycleRepeatMode,
    handleSeek,

    // Direct access to player store for components
    playerStore,
  }
}
