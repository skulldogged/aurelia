import { readonly, ref, type Ref } from 'vue'

import { usePlayerStore } from '@/stores'

interface MusicPlayerRef {
  nextSong:        () => void
  onSeek:          (value: number[]) => void
  previousSong:    () => void
  togglePlayPause: () => void
  toggleRepeat:    () => void
  toggleShuffle:   () => void
}

const isQueueOpen = ref(false)
const isEqualizerOpen = ref(false)
const isFullScreenPlayerOpen = ref(false)
const searchQuery = ref('')
const isSearchVisible = ref(false)

const musicPlayerRef = ref<MusicPlayerRef | null>(null)

const handleGlobalSearch = (query: string): void => {
  searchQuery.value = query
  isSearchVisible.value = true
}

const toggleQueue = (): void => {
  if (isEqualizerOpen.value)
    isEqualizerOpen.value = false

  isQueueOpen.value = !isQueueOpen.value
}

const toggleFullScreenPlayer = (): void => {
  isFullScreenPlayerOpen.value = !isFullScreenPlayerOpen.value
}

const handleTogglePlayPause = (): void => {
  musicPlayerRef.value?.togglePlayPause()
}

const handleSeek = (value: number): void => {
  musicPlayerRef.value?.onSeek([value])
}

const toggleSearchVisibility = (visible: boolean): void => {
  isSearchVisible.value = visible
}

export interface PlayerControls {
  handleGlobalSearch:     (query: string) => void
  handleNextSong:         () => void
  handlePreviousSong:     () => void
  handleSeek:             (value: number) => void
  handleTogglePlayPause:  () => void
  handleToggleRepeat:     () => void
  handleToggleShuffle:    () => void
  isEqualizerOpen:        Readonly<Ref<boolean>>
  isFullScreenPlayerOpen: Readonly<Ref<boolean>>
  isQueueOpen:            Readonly<Ref<boolean>>
  isSearchVisible:        Readonly<Ref<boolean>>
  musicPlayerRef:         Ref<MusicPlayerRef | null>
  playerStore:            ReturnType<typeof usePlayerStore>
  searchQuery:            Readonly<Ref<string>>
  toggleEqualizer:        () => void
  toggleFullScreenPlayer: () => void
  toggleQueue:            () => void
  toggleSearchVisibility: (visible: boolean) => void
}

export const usePlayerControls = (): PlayerControls => {
  const playerStore = usePlayerStore()

  const toggleEqualizer = (): void => {
    if (isQueueOpen.value)
      isQueueOpen.value = false

    const newState = !isEqualizerOpen.value
    isEqualizerOpen.value = newState
    playerStore.setEQEnabled(newState)
  }

  const handlePreviousSong = (): void => {
    playerStore.previousSong()
    musicPlayerRef.value?.previousSong()
  }

  const handleNextSong = (): void => {
    playerStore.nextSong()
    musicPlayerRef.value?.nextSong()
  }

  return {
    handleGlobalSearch,
    handleNextSong,
    handlePreviousSong,
    handleSeek,
    handleTogglePlayPause,
    handleToggleRepeat: playerStore.cycleRepeatMode,

    handleToggleShuffle:    playerStore.toggleShuffle,
    isEqualizerOpen:        readonly(isEqualizerOpen),
    isFullScreenPlayerOpen: readonly(isFullScreenPlayerOpen),
    isQueueOpen:            readonly(isQueueOpen),
    isSearchVisible:        readonly(isSearchVisible),
    musicPlayerRef,
    playerStore,
    searchQuery:            readonly(searchQuery),
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleQueue,

    toggleSearchVisibility,
  }
}
