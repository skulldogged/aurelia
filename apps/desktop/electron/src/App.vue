<script setup lang='ts'>
  import type { Song } from '@shared/lib/api/types'
  import type { Credentials } from '@shared/lib/api/types'
  import type { PlayerState } from '@shared/stores'

  import { getSyncStateEffect, runAureliaEffect } from '@shared'
  import { quitDesktopApp } from '@shared/lib/desktop-shell'
  // Import from shared package
  import MainLayout from '@shared/components/layout/MainLayout.vue'
  import Equalizer from '@shared/components/player/Equalizer.vue'
  import FullscreenPlayer from '@shared/components/player/FullscreenPlayer.vue'
  import LyricsSidebar from '@shared/components/player/LyricsSidebar.vue'
  import MusicPlayer from '@shared/components/player/MusicPlayer.vue'
  import Queue from '@shared/components/player/Queue.vue'
  import GlobalSearch from '@shared/components/shared/GlobalSearch.vue'
  import Button from '@shared/components/ui/Button.vue'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@shared/components/ui/dialog'
  import { Toaster } from '@shared/components/ui/sonner'
  import { useAuth } from '@shared/composables/useAuth'
  import { useDiscordPresence } from '@shared/composables/useDiscordPresence'
  import { useLastFm } from '@shared/composables/useLastFm'
  import { useListenBrainz } from '@shared/composables/useListenBrainz'
  import { useNavigation } from '@shared/composables/useNavigation'
  import { usePlayerControls } from '@shared/composables/usePlayerControls'
  import { usePlayerSession } from '@shared/composables/usePlayerSession'
  import { useSongInteractions } from '@shared/composables/useSongInteractions'
  import { useSystemTray } from '@shared/composables/useSystemTray'
  import { useTopBar } from '@shared/composables/useTopBar'
  import { useVisualizerData } from '@shared/composables/useVisualizerData'
  import { setAuthLogout } from '@shared/lib/auth-interceptor'
  import Login from '@shared/pages/login.vue'
  import { useHomeStore, useLibraryStore } from '@shared/stores'
  import { useColorMode, useMagicKeys } from '@vueuse/core'
  import { computed, onMounted, ref, watch } from 'vue'

  useColorMode()

  const {
    authStatus: authStatusRef,
    clearError: clearAuthError,
    credentials: credentialsRef,
    error: authErrorRef,
    login,
    logout,
  } = useAuth()
  setAuthLogout(logout)
  const libraryStore = useLibraryStore()
  const homeStore = useHomeStore()
  useSystemTray()
  useDiscordPresence()
  useLastFm()
  useListenBrainz()

  // Unwrap readonly refs for template use
  const authStatus = computed(() => authStatusRef.value)
  const credentials = computed(() => credentialsRef.value)
  const authError = computed(() => authErrorRef.value)

  const { topBarContent: topBarContentRef } = useTopBar()
  const topBarContent = computed(() => topBarContentRef.value)

  // Visualizer data from Rust backend FFT analysis
  const {
    frequencyData: frequencyDataRef,
    setEnabled: setAnalyzerEnabled,
    timeDomainData: timeDomainDataRef,
  } = useVisualizerData()
  const frequencyData = computed(() => frequencyDataRef.value)
  const timeDomainData = computed(() => timeDomainDataRef.value)

  const isSearchOpen = ref(false)
  const showExitDialog = ref(false)
  const keys = useMagicKeys()
  const ctrlK = keys['Ctrl+K']
  watch(ctrlK, v => {
    if (v)
      isSearchOpen.value = !isSearchOpen.value
  })

  const {
    canGoBack: canGoBackRef,
    canGoForward: canGoForwardRef,
    currentView: currentViewRef,
    handleNavigation,
    navigateBack,
    navigateForward,
    navigateToAlbum,
    navigateToArtist,
  } = useNavigation()

  // Unwrap navigation refs for template
  const canGoBack = computed(() => canGoBackRef.value)
  const canGoForward = computed(() => canGoForwardRef.value)
  const currentView = computed(() => currentViewRef.value)

  const {
    handleNextSong,
    handlePreviousSong,
    handleSeek,
    handleTogglePlayPause,
    handleToggleRepeat,
    handleToggleShuffle,
    isEqualizerOpen: isEqualizerOpenRef,
    isFullScreenPlayerOpen: isFullScreenPlayerOpenRef,
    isLyricsOpen: isLyricsOpenRef,
    isQueueOpen: isQueueOpenRef,
    musicPlayerRef,
    playerStore,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleLyrics,
    toggleQueue,
  } = usePlayerControls()

  // Unwrap player control refs for template
  const isEqualizerOpen = computed(() => isEqualizerOpenRef.value)
  const isFullScreenPlayerOpen = computed(() => isFullScreenPlayerOpenRef.value)
  const isLyricsOpen = computed(() => isLyricsOpenRef.value)
  const isQueueOpen = computed(() => isQueueOpenRef.value)

  // State for fullscreen player panels
  const isFsQueueOpen = ref(false)
  const isFsEqualizerOpen = ref(false)
  const isFsLyricsOpen = ref(false)

  const toggleFsQueue = (): void => {
    if (!isFsQueueOpen.value) {
      isFsEqualizerOpen.value = false
      isFsLyricsOpen.value = false
    }
    isFsQueueOpen.value = !isFsQueueOpen.value
  }
  const toggleFsEqualizer = (): void => {
    if (!isFsEqualizerOpen.value) {
      isFsQueueOpen.value = false
      isFsLyricsOpen.value = false
    }
    isFsEqualizerOpen.value = !isFsEqualizerOpen.value
  }
  const toggleFsLyrics = (): void => {
    if (!isFsLyricsOpen.value) {
      isFsQueueOpen.value = false
      isFsEqualizerOpen.value = false
    }
    isFsLyricsOpen.value = !isFsLyricsOpen.value
  }

  // Enable analyzer when visualizer is enabled and playing
  watch(
    [() => playerStore.visualizerEnabled, () => playerStore.isPlaying],
    ([vizEnabled, isPlaying]) => {
      const shouldEnable = vizEnabled && isPlaying
      setAnalyzerEnabled(shouldEnable)
    },
    { immediate: true },
  )

  const {
    playInstantMix,
    playSong,
    playSongs,
    removeSongFromPlaylist,
    toggleFavorite,
    updatePlaylist,
  } = useSongInteractions(credentialsRef)

  // Access player state directly via computed refs (avoids Pinia 3 cross-package storeToRefs type issues)
  const currentSong = computed(() => playerStore.currentSong)
  const currentTime = computed(() => playerStore.currentTime)
  const duration = computed(() => playerStore.duration)
  const isPlaying = computed(() => playerStore.isPlaying)
  const isShuffled = computed(() => playerStore.isShuffled)
  const playlist = computed(() => playerStore.playlist)
  const progress = computed(() => playerStore.progress)
  const repeatMode = computed(() => playerStore.repeatMode)

  const hasNext = computed(() => playerStore.canGoNext())
  const hasPrevious = computed(() => playerStore.canGoPrevious())

  const playerState = computed(() => ({
    currentSong:     currentSong.value,
    currentTime:     currentTime.value,
    duration:        duration.value,
    hasNext:         hasNext.value,
    hasPlayer:       !!currentSong.value,
    hasPrevious:     hasPrevious.value,
    isEqualizerOpen: isEqualizerOpen.value,
    isLyricsOpen:    isLyricsOpen.value,
    isMuted:         playerStore.isMuted,
    isPlaying:       isPlaying.value,
    isQueueOpen:     isQueueOpen.value,
    isShuffled:      isShuffled.value,
    playlist:        playlist.value,
    progress:        progress.value,
    repeatMode:      repeatMode.value,
    volume:          playerStore.volume * 100,
  }))

  const isSyncing = ref(false)
  const isClearing = ref(false)
  const lastSyncTime = ref<null | string>(null)
  const swipeProgress = ref<null | {
    deltaY:    number
    direction: 'down' | 'left' | 'right' | 'up' | null
    startY:    number
  }>(null)

  const emptyVisualizerData = new Uint8Array(0)
  const inactiveFullscreenPlayerState: PlayerState = {
    currentSong: null,
    currentTime: 0,
    duration:    0,
    hasNext:     false,
    hasPrevious: false,
    isMuted:     false,
    isPlaying:   false,
    isShuffled:  false,
    playlist:    [],
    progress:    0,
    repeatMode:  'none',
    volume:      0,
  }
  const isFullscreenPlayerActive = computed(() =>
    isFullScreenPlayerOpen.value || swipeProgress.value?.direction === 'up',
  )
  const fullscreenPlayerState = computed<PlayerState>(() =>
    isFullscreenPlayerActive.value ? playerState.value : inactiveFullscreenPlayerState,
  )
  const fullscreenFrequencyData = computed(() =>
    isFullscreenPlayerActive.value ? frequencyData.value : emptyVisualizerData,
  )
  const fullscreenTimeDomainData = computed(() =>
    isFullscreenPlayerActive.value ? timeDomainData.value : emptyVisualizerData,
  )

  usePlayerSession()

  onMounted(async () => {
    playerStore.setVolume(playerStore.volume)
  })

  const loadLibraryAndHomeData = async (): Promise<void> => {
    await libraryStore.loadLibrary()
    if (!libraryStore.isLoaded)
      return
    await homeStore.refreshHomeData()
  }

  const fetchSyncState = async (): Promise<void> => {
    try {
      const syncState = await runAureliaEffect(getSyncStateEffect())
      if (syncState.lastSyncTime)
        lastSyncTime.value = syncState.lastSyncTime
    } catch {
      // best-effort sync state fetch
    }
  }

  watch(authStatus, async newStatus => {
    if (newStatus === 'loggedIn' && credentials.value) {
      await loadLibraryAndHomeData()
      await fetchSyncState()
    }
  })

  watch(authStatus, newStatus => {
    if (newStatus === 'loggedOut') {
      libraryStore.clearData()
      homeStore.resetHomeData()
    }
  })

  const handleLogin = async (loginCredentials: Credentials): Promise<void> => {
    login(loginCredentials)
  }

  const handleLogout = (): void => {
    logout()
    playerStore.setCurrentSong(null)
    playerStore.setPlaylist([])
    playerStore.setCurrentIndex(-1)
  }

  const handleToggleFavorite = (song: null | Song): void => {
    if (song) {
      toggleFavorite(song)
    }
  }

  const handleVolumeChange = (newVolume: number): void => {
    playerStore.setVolume(newVolume / 100)
  }

  const handleSwipeProgress = (
    progress: null | {
      deltaY:    number
      direction: 'down' | 'left' | 'right' | 'up' | null
      startY:    number
    },
  ): void => {
    swipeProgress.value = progress
  }

  const handleSyncLibrary = async (): Promise<void> => {
    if (!credentials.value) return
    isSyncing.value = true
    await libraryStore.syncLibrary()
    await homeStore.refreshHomeData()
    await fetchSyncState()
    isSyncing.value = false
  }

  const handleClearCache = async (): Promise<void> => {
    if (!credentials.value) return
    isClearing.value = true
    await libraryStore.clearCache()
    await homeStore.refreshHomeData()
    isClearing.value = false
  }

  const handleLyricsLoaded = (hasLyrics: boolean): void => {
    // Pass lyrics availability to player controls
    playerStore.setHasLyrics(hasLyrics)
  }

  const confirmExit = async (): Promise<void> => {
    showExitDialog.value = false
    // Exit the app
    await quitDesktopApp().catch(() => {})
  }

  const cancelExit = (): void => {
    showExitDialog.value = false
  }
</script>

<template>
  <div id='app' class='h-screen text-foreground'>
    <div
      v-if="authStatus === 'pending' || authStatus === 'initializing'"
      class='size-full flex items-center justify-center'
    >
      <div class='text-center'>
        <div class='animate-spin size-8 border-4 border-primary border-t-transparent rounded-full mx-auto' />
        <p class='mt-4 text-muted-foreground'>
          Connecting to server...
        </p>
      </div>
    </div>
    <div
      v-else-if="authStatus === 'error'"
      class='size-full flex items-center justify-center'
    >
      <div class='text-center max-w-md mx-auto p-8'>
        <div class='text-red-500 text-6xl mb-4'>
          !
        </div>
        <h2 class='text-xl font-semibold mb-2'>
          Connection Error
        </h2>
        <p class='text-muted-foreground mb-4'>
          {{ authError?.message || 'Failed to connect to server' }}
        </p>
        <Button @click='clearAuthError' variant='outline'>
          Try Again
        </Button>
      </div>
    </div>
    <Login @login='handleLogin' v-else-if="authStatus === 'loggedOut'" />
    <MainLayout
      @global-search='isSearchOpen = !isSearchOpen'
      @logout='handleLogout'
      @navigate='handleNavigation'
      @navigate-back='navigateBack'
      @navigate-forward='navigateForward'
      v-else
      :navigation-state='{
        canGoBack,
        canGoForward,
        currentView,
      }'
      :player-state='{
        hasPlayer: !!currentSong,
        isEqualizerOpen,
        isLyricsOpen,
        isQueueOpen,
      }'
    >
      <RouterView v-slot='{ Component }'>
        <component
          :is='Component'
          @clear-cache='handleClearCache'
          @logout='handleLogout'
          @play-instant-mix='playInstantMix'
          @play-song='playSong'
          @play-songs='playSongs'
          @reload-library='loadLibraryAndHomeData'
          @select-album='navigateToAlbum'
          @select-artist='navigateToArtist'
          @sync-library='handleSyncLibrary'
          @toggle-favorite='handleToggleFavorite'
          :credentials='credentials'
          :current-song='currentSong'
          :is-clearing='isClearing'
          :is-syncing='isSyncing'
          :last-sync-time='lastSyncTime'
        />
      </RouterView>

      <template #queue>
        <div class='sidebar-panels h-full relative'>
          <Queue
            @remove-song='removeSongFromPlaylist'
            v-if='isQueueOpen'
            class='absolute inset-0'
          />
          <Equalizer
            v-if='isEqualizerOpen'
            class='absolute inset-0'
          />
          <LyricsSidebar
            @lyrics-loaded='handleLyricsLoaded'
            @seek='handleSeek'
            v-if='isLyricsOpen'
            :current-song='currentSong as any'
            :current-time='currentTime'
            :duration='duration'
            class='absolute inset-0'
          />
        </div>
      </template>

      <template #player>
        <MusicPlayer
          @swipe-progress='handleSwipeProgress'
          @toggle-equalizer='toggleEqualizer'
          @toggle-favorite='handleToggleFavorite'
          @toggle-fullscreen='toggleFullScreenPlayer'
          @toggle-lyrics='toggleLyrics'
          @toggle-queue='toggleQueue'
          v-if='currentSong'
          ref='musicPlayerRef'
          :frequency-data='frequencyData'
          :is-equalizer-open='isEqualizerOpen'
          :is-lyrics-open='isLyricsOpen'
          :is-queue-open='isQueueOpen'
          :server-url='credentials!.serverUrl'
          :time-domain-data='timeDomainData'
          :token='credentials!.token'
        />
      </template>

      <template #top-bar>
        <component
          :is='topBarContent?.component'
          v-if='topBarContent'
          v-bind='topBarContent.props'
        />
      </template>
    </MainLayout>

    <FullscreenPlayer
      @close='toggleFullScreenPlayer'
      @next-song='handleNextSong'
      @play-song='playSong'
      @previous-song='handlePreviousSong'
      @remove-song='removeSongFromPlaylist'
      @seek='handleSeek'
      @toggle-equalizer='toggleFsEqualizer'
      @toggle-favorite='handleToggleFavorite'
      @toggle-lyrics='toggleFsLyrics'
      @toggle-mute='playerStore.toggleMute'
      @toggle-play-pause='handleTogglePlayPause'
      @toggle-queue='toggleFsQueue'
      @toggle-repeat='handleToggleRepeat'
      @toggle-shuffle='handleToggleShuffle'
      @update:playlist='updatePlaylist'
      @volume-change='handleVolumeChange'
      :frequency-data='fullscreenFrequencyData'
      :is-equalizer-open='isFsEqualizerOpen'
      :is-lyrics-open='isFsLyricsOpen'
      :is-queue-open='isFsQueueOpen'
      :player-state='fullscreenPlayerState'
      :preview-progress='swipeProgress'
      :server-url='credentials?.serverUrl'
      :show='isFullScreenPlayerOpen'
      :time-domain-data='fullscreenTimeDomainData'
      :token='credentials?.token'
    />

    <GlobalSearch v-model:open='isSearchOpen' />

    <!-- Exit Confirmation Dialog -->
    <Dialog v-model:open='showExitDialog'>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Exit Application</DialogTitle>
          <DialogDescription>
            Are you sure you want to exit Aurelia?
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button @click='cancelExit' variant='outline'>
            Cancel
          </Button>
          <Button @click='confirmExit' variant='destructive'>
            Exit
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Toast Notifications -->
    <Toaster position='top-center' />
  </div>
</template>
