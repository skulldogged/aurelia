<script setup lang='ts'>
  import { onBackButtonPress } from '@tauri-apps/api/app'
  import { useColorMode, useMagicKeys } from '@vueuse/core'
  import { storeToRefs } from 'pinia'
  import { computed, onMounted, ref, watch } from 'vue'

  import type { Credentials, Song } from '@/bindings'

  import { commands } from '@/bindings'
  import MainLayout from '@/components/layout/MainLayout.vue'
  import Equalizer from '@/components/player/Equalizer.vue'
  import FullscreenPlayer from '@/components/player/FullscreenPlayer.vue'
  import LyricsSidebar from '@/components/player/LyricsSidebar.vue'
  import MusicPlayer from '@/components/player/MusicPlayer.vue'
  import Queue from '@/components/player/Queue.vue'
  import GlobalSearch from '@/components/shared/GlobalSearch.vue'
  import WindowControls from '@/components/shared/WindowControls.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'
  import { Toaster } from '@/components/ui/sonner'
  import { useAndroidNowPlayingService } from '@/composables/useAndroidNowPlayingService'
  import { useAuth } from '@/composables/useAuth'
  import { useDiscordPresence } from '@/composables/useDiscordPresence'
  import { useLastFm } from '@/composables/useLastFm'
  import { useListenBrainz } from '@/composables/useListenBrainz'
  import { useNavigation } from '@/composables/useNavigation'
  import { usePlayerControls } from '@/composables/usePlayerControls'
  import { usePlayerSession } from '@/composables/usePlayerSession'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { useSystemTray } from '@/composables/useSystemTray'
  import { useTopBar } from '@/composables/useTopBar'
  import { useVisualizerData } from '@/composables/useVisualizerData'
  import { setAuthLogout } from '@/lib/auth-interceptor'
  import { getPlatform, isMobile, Platform } from '@/lib/platform'
  import Login from '@/pages/login.vue'
  import { useHomeStore } from '@/stores'
  import { useLibraryStore } from '@/stores/library'

  useColorMode()

  const { authStatus, clearError: clearAuthError, credentials, error: authError, login, logout } = useAuth()
  setAuthLogout(logout)
  const libraryStore = useLibraryStore()
  const homeStore = useHomeStore()
  useSystemTray()
  useDiscordPresence()
  useLastFm()
  useListenBrainz()
  useAndroidNowPlayingService()

  const { topBarContent } = useTopBar()

  // Visualizer data from Rust backend FFT analysis
  const { frequencyData, setEnabled: setAnalyzerEnabled, timeDomainData } = useVisualizerData()

  const isSearchOpen = ref(false)
  const showExitDialog = ref(false)
  const keys = useMagicKeys()
  const ctrlK = keys['Ctrl+K']
  watch(ctrlK, v => {
    if (v)
      isSearchOpen.value = !isSearchOpen.value
  })

  const {
    canGoBack,
    canGoForward,
    currentView,
    handleNavigation,
    navigateBack,
    navigateForward,
    navigateToAlbum,
    navigateToArtist,
  } = useNavigation()

  const {
    handleNextSong,
    handlePreviousSong,
    handleSeek,
    handleTogglePlayPause,
    handleToggleRepeat,
    handleToggleShuffle,
    isEqualizerOpen,
    isFullScreenPlayerOpen,
    isLyricsOpen,
    isQueueOpen,
    musicPlayerRef,
    playerStore,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleLyrics,
    toggleQueue,
  } = usePlayerControls()

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
  } = useSongInteractions(credentials)

  const {
    currentSong,
    currentTime,
    duration,
    isPlaying,
    isShuffled,
    playlist,
    progress,
    repeatMode,
  } = storeToRefs(playerStore)

  const hasNext = computed(() =>
    playlist.value.length > 1
    && playerStore.currentIndex > -1
    && playerStore.currentIndex < playlist.value.length - 1,
  )
  const hasPrevious = computed(() => playlist.value.length > 1 && playerStore.currentIndex > 0)

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
  const swipeProgress = ref<null | {
    deltaY:    number
    direction: 'down' | 'left' | 'right' | 'up' | null
    startY:    number
  }>(null)

  usePlayerSession()

  onMounted(async () => {
    playerStore.setVolume(playerStore.volume)

    if (isMobile()) {
      onBackButtonPress(async () => {
        console.log('Back button pressed', {
          canGoBack:              canGoBack.value,
          isFullScreenPlayerOpen: isFullScreenPlayerOpen.value,
        })

        if (isFullScreenPlayerOpen.value) {
          console.log('Closing fullscreen player')
          toggleFullScreenPlayer()
          return true
        }
        if (canGoBack.value) {
          console.log('Navigating back')
          navigateBack()
          return true
        }
        // Show exit confirmation dialog
        console.log('Showing exit dialog')
        showExitDialog.value = true
        return true // Prevent default back behavior while showing dialog
      })
    }
  })

  const loadLibraryAndHomeData = async (): Promise<void> => {
    await libraryStore.loadLibrary()
    if (!libraryStore.isLoaded)
      return
    await homeStore.refreshHomeData()
  }

  watch(authStatus, async newStatus => {
    if (newStatus === 'loggedIn' && credentials.value)
      await loadLibraryAndHomeData()
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

  const handleToggleFavorite = async (song: Song): Promise<void> => {
    await toggleFavorite(song)
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
    await libraryStore.syncLibrary(credentials.value)
    await homeStore.refreshHomeData()
    isSyncing.value = false
  }

  const handleClearCache = async (): Promise<void> => {
    if (!credentials.value) return
    isClearing.value = true
    await libraryStore.clearCache(credentials.value)
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
    await commands.quitApplication()
  }

  const cancelExit = (): void => {
    showExitDialog.value = false
  }
</script>

<template>
  <div id='app' class='h-screen text-foreground'>
    <div v-if="authStatus === 'pending'" class='size-full flex items-center justify-center'>
      <div class='text-center'>
        <div class='animate-spin size-8 border-4 border-primary border-t-transparent rounded-full mx-auto' />
        <p class='mt-4 text-muted-foreground'>
          Connecting to server...
        </p>
      </div>
    </div>
    <div v-else-if="authStatus === 'error'" class='size-full flex items-center justify-center'>
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
          @toggle-favorite='toggleFavorite'
          :credentials='credentials'
          :current-song='currentSong'
          :is-clearing='isClearing'
          :is-syncing='isSyncing'
        />
      </RouterView>

      <template #queue>
        <div class='sidebar-panels h-full relative'>
          <Queue
            @remove-song='removeSongFromPlaylist'
            v-show='isQueueOpen'
            class='absolute inset-0'
          />
          <Equalizer
            v-show='isEqualizerOpen'
            class='absolute inset-0'
          />
          <LyricsSidebar
            @lyrics-loaded='handleLyricsLoaded'
            @seek='handleSeek'
            v-show='isLyricsOpen'
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
      :frequency-data='frequencyData'
      :is-equalizer-open='isFsEqualizerOpen'
      :is-lyrics-open='isFsLyricsOpen'
      :is-queue-open='isFsQueueOpen'
      :player-state='playerState'
      :preview-progress='swipeProgress'
      :server-url='credentials?.serverUrl'
      :show='isFullScreenPlayerOpen'
      :time-domain-data='timeDomainData'
      :token='credentials?.token'
    />

    <GlobalSearch v-model:open='isSearchOpen' />

    <WindowControls
      v-if='!isFullScreenPlayerOpen && !isMobile() && getPlatform() !== Platform.MacOS'
      class='fixed top-0 right-0 z-100'
    />

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