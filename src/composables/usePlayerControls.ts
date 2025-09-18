import { ref, readonly } from 'vue'
import { usePlayerStore } from '@/stores'

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

  const isQueueOpen = ref(false)
  const isEqualizerOpen = ref(false)
  const isFullScreenPlayerOpen = ref(false)
  const searchQuery = ref('')
  const isSearchVisible = ref(false)

  const musicPlayerRef = ref<MusicPlayerRef | null>(null)

  const handleGlobalSearch = (query: string) => {
    searchQuery.value = query
    isSearchVisible.value = true
  }

  const toggleQueue = () => {
    if (isEqualizerOpen.value) {
      isEqualizerOpen.value = false
    }
    isQueueOpen.value = !isQueueOpen.value
  }

  const toggleEqualizer = () => {
    if (isQueueOpen.value) {
      isQueueOpen.value = false
    }
    const newState = !isEqualizerOpen.value
    isEqualizerOpen.value = newState
    playerStore.setEQEnabled(newState)
  }

  const toggleFullScreenPlayer = () => {
    isFullScreenPlayerOpen.value = !isFullScreenPlayerOpen.value
  }

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

  const toggleSearchVisibility = (visible: boolean) => {
    isSearchVisible.value = visible
  }

  return {
    isQueueOpen:            readonly(isQueueOpen),
    isEqualizerOpen:        readonly(isEqualizerOpen),
    isFullScreenPlayerOpen: readonly(isFullScreenPlayerOpen),
    searchQuery:            readonly(searchQuery),
    isSearchVisible:        readonly(isSearchVisible),
    musicPlayerRef,

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

    playerStore,
  }
}
