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
const isLyricsOpen = ref(false)
const searchQuery = ref('')
const isSearchVisible = ref(false)

const musicPlayerRef = ref<MusicPlayerRef | null>(null)

const handleGlobalSearch = (query: string): void => {
  searchQuery.value = query
  isSearchVisible.value = true
}

const closePanels = (except?: 'equalizer' | 'lyrics' | 'queue'): void => {
  if (except !== 'queue') isQueueOpen.value = false
  if (except !== 'equalizer') isEqualizerOpen.value = false
  if (except !== 'lyrics') isLyricsOpen.value = false
}

const toggleQueue = (): void => {
  if (!isQueueOpen.value)
    closePanels('queue')
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
  isLyricsOpen:           Readonly<Ref<boolean>>
  isQueueOpen:            Readonly<Ref<boolean>>
  isSearchVisible:        Readonly<Ref<boolean>>
  musicPlayerRef:         Ref<MusicPlayerRef | null>
  playerStore:            ReturnType<typeof usePlayerStore>
  searchQuery:            Readonly<Ref<string>>
  toggleEqualizer:        () => void
  toggleFullScreenPlayer: () => void
  toggleLyrics:           () => void
  toggleQueue:            () => void
  toggleSearchVisibility: (visible: boolean) => void
}

export const usePlayerControls = (): PlayerControls => {
  const playerStore = usePlayerStore()

  const toggleEqualizer = (): void => {
    if (!isEqualizerOpen.value)
      closePanels('equalizer')
    const newState = !isEqualizerOpen.value
    isEqualizerOpen.value = newState
    playerStore.setEQEnabled(newState)
  }

  const toggleLyrics = (): void => {
    if (!isLyricsOpen.value)
      closePanels('lyrics')
    isLyricsOpen.value = !isLyricsOpen.value
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
    isLyricsOpen:           readonly(isLyricsOpen),
    isQueueOpen:            readonly(isQueueOpen),
    isSearchVisible:        readonly(isSearchVisible),
    musicPlayerRef,
    playerStore,
    searchQuery:            readonly(searchQuery),
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleLyrics,
    toggleQueue,

    toggleSearchVisibility,
  }
}
