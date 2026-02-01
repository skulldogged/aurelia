<script setup lang="ts">
  import type { Credentials } from '@shared'

  import {
    Equalizer,
    FullscreenPlayer,
    getApiClient,
    GlobalSearch,
    Login,
    LyricsSidebar,
    MainLayout,
    MusicPlayer,
    Queue,
    Toaster,
    useAccentColorStore,

    useAuth,
    useHomeStore,
    useLibraryStore,
    useNavigation,
    usePlayerControls,
    usePlayerSession,
    usePlayerStore,
    useSongInteractions,
    useThemeStore,
    useTopBar,
    useVisualizerData,
  } from '@shared'
  import { useColorMode } from '@vueuse/core'
  import { computed, onMounted, ref, watch } from 'vue'

  useColorMode()
  useThemeStore() // Initialize theme and apply CSS variables
  useAccentColorStore() // Initialize accent colors and apply CSS variables

  const { authStatus, credentials, login, logout } = useAuth()

  // Visualizer data from Web Audio API
  const {
    frequencyData: frequencyDataRef,
    setEnabled: setAnalyzerEnabled,
    timeDomainData: timeDomainDataRef,
  } = useVisualizerData()
  const frequencyData = computed(() => frequencyDataRef.value)
  const timeDomainData = computed(() => timeDomainDataRef.value)
  const libraryStore = useLibraryStore()
  const homeStore = useHomeStore()
  const playerStore = usePlayerStore()

  const navigation = useNavigation()
  const playerControls = usePlayerControls()
  const songInteractions = useSongInteractions(credentials)
  const _topBar = useTopBar()

  // Destructure player control handlers that are needed for FullscreenPlayer
  const {
    handleNextSong,
    handlePreviousSong,
    handleSeek,
    handleTogglePlayPause,
    handleToggleRepeat,
    handleToggleShuffle,
  } = playerControls

  // Initialize player session for playback reporting
  usePlayerSession()

  // Enable analyzer when visualizer is enabled and playing
  watch(
    [() => playerStore.visualizerEnabled, () => playerStore.isPlaying],
    ([vizEnabled, isPlaying]) => {
      const shouldEnable = vizEnabled && isPlaying
      setAnalyzerEnabled(shouldEnable)
    },
    { immediate: true },
  )

  const isAuthenticated = computed(() => authStatus.value === 'loggedIn' || authStatus.value === 'verifying')

  const isSyncing = ref(false)
  const isClearing = ref(false)
  const lastSyncTime = ref<null | string>(null)

  const loadLibraryAndHomeData = async (): Promise<void> => {
    await libraryStore.loadLibrary()
    if (!libraryStore.isLoaded)
      return
    await homeStore.refreshHomeData()
  }

  const fetchSyncState = async (): Promise<void> => {
    const result = await getApiClient().getSyncState()
    if (result.status === 'ok' && result.data.lastSyncTime)
      lastSyncTime.value = result.data.lastSyncTime
  }

  const handleSyncLibrary = async (): Promise<void> => {
    if (!credentials.value) return
    isSyncing.value = true
    await libraryStore.syncLibrary(credentials.value)
    await homeStore.refreshHomeData()
    await fetchSyncState()
    isSyncing.value = false
  }

  const handleClearCache = async (): Promise<void> => {
    if (!credentials.value) return
    isClearing.value = true
    await libraryStore.clearCache(credentials.value)
    await homeStore.refreshHomeData()
    isClearing.value = false
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

  const handleLogin = (creds: Credentials): void => {
    login(creds)
  // Watcher will handle sync
  }

  const handleLogout = (): void => {
    logout()
  }

  const handleQuit = (): void => {
    window.location.reload()
  }

  const handleVolumeChange = (value: number): void => {
    playerStore.setVolume(value / 100)
  }

  const handleToggleMute = (): void => {
    playerStore.toggleMute()
  }

  onMounted(async () => {
    if (isAuthenticated.value && credentials.value) {
      await loadLibraryAndHomeData()
      await fetchSyncState()
    }
  })
</script>

<template>
  <Toaster />

  <div class='h-screen w-screen overflow-hidden bg-background text-foreground'>
    <template v-if="authStatus === 'pending' || authStatus === 'initializing'">
      <div class='size-full flex items-center justify-center'>
        <div class='text-center'>
          <div class='animate-spin size-8 border-4 border-primary border-t-transparent rounded-full mx-auto' />
          <p class='mt-4 text-muted-foreground'>
            Connecting to server...
          </p>
        </div>
      </div>
    </template>

    <template v-else-if='isAuthenticated'>
      <MainLayout
        @logout='handleLogout'
        @navigate='navigation.handleNavigation'
        @navigate-back='navigation.navigateBack'
        @navigate-forward='navigation.navigateForward'
        @quit='handleQuit'
        :navigation-state='{
          canGoBack: navigation.canGoBack.value,
          canGoForward: navigation.canGoForward.value,
          currentView: navigation.currentView.value,
        }'
        :player-state='{
          hasPlayer: true,
          isEqualizerOpen: playerControls.isEqualizerOpen.value,
          isLyricsOpen: playerControls.isLyricsOpen.value,
          isQueueOpen: playerControls.isQueueOpen.value,
        }'
      >
        <RouterView v-slot='{ Component }'>
          <component
            :is='Component'
            @clear-cache='handleClearCache'
            @logout='handleLogout'
            @play-instant-mix='songInteractions.playInstantMix'
            @play-song='songInteractions.playSong'
            @play-songs='songInteractions.playSongs'
            @select-album='navigation.navigateToAlbum'
            @select-artist='navigation.navigateToArtist'
            @sync-library='handleSyncLibrary'
            @toggle-favorite='songInteractions.toggleFavorite'
            :credentials='credentials'
            :current-song='playerStore.currentSong'
            :is-clearing='isClearing'
            :is-syncing='isSyncing'
            :last-sync-time='lastSyncTime'
          />
        </RouterView>

        <template #queue>
          <div class='h-full w-full overflow-hidden'>
            <Queue
              @remove-song='songInteractions.removeSongFromPlaylist'
              v-if='playerControls.isQueueOpen.value'
            />
            <Equalizer
              v-else-if='playerControls.isEqualizerOpen.value'
            />
            <LyricsSidebar
              @seek='playerControls.handleSeek'
              v-else-if='playerControls.isLyricsOpen.value'
              :current-song='playerStore.currentSong'
              :current-time='playerStore.currentTime'
              :duration='playerStore.duration'
            />
          </div>
        </template>

        <template #player>
          <MusicPlayer
            @add-to-playlist='() => {}'
            @instant-mix='songInteractions.playInstantMix'
            @toggle-equalizer='playerControls.toggleEqualizer'
            @toggle-favorite='songInteractions.toggleFavorite'
            @toggle-fullscreen='playerControls.toggleFullScreenPlayer'
            @toggle-lyrics='playerControls.toggleLyrics'
            @toggle-queue='playerControls.toggleQueue'
            :frequency-data='frequencyData'
            :is-equalizer-open='playerControls.isEqualizerOpen.value'
            :is-lyrics-open='playerControls.isLyricsOpen.value'
            :is-queue-open='playerControls.isQueueOpen.value'
            :server-url="credentials?.serverUrl ?? ''"
            :time-domain-data='timeDomainData'
            :token="credentials?.token ?? ''"
          />
        </template>
      </MainLayout>

      <FullscreenPlayer
        @close='playerControls.toggleFullScreenPlayer'
        @instant-mix='songInteractions.playInstantMix'
        @next-song='handleNextSong'
        @previous-song='handlePreviousSong'
        @seek='handleSeek'
        @toggle-equalizer='playerControls.toggleEqualizer'
        @toggle-favorite='songInteractions.toggleFavorite'
        @toggle-fullscreen='playerControls.toggleFullScreenPlayer'
        @toggle-lyrics='playerControls.toggleLyrics'
        @toggle-mute='handleToggleMute'
        @toggle-play-pause='handleTogglePlayPause'
        @toggle-queue='playerControls.toggleQueue'
        @toggle-repeat='handleToggleRepeat'
        @toggle-shuffle='handleToggleShuffle'
        @volume-change='handleVolumeChange'
        v-if='playerControls.isFullScreenPlayerOpen.value'
        :frequency-data='frequencyData'
        :is-equalizer-open='playerControls.isEqualizerOpen.value'
        :is-lyrics-open='playerControls.isLyricsOpen.value'
        :is-queue-open='playerControls.isQueueOpen.value'
        :player-state='{
          currentSong: playerStore.currentSong,
          currentTime: playerStore.currentTime,
          duration: playerStore.duration,
          hasNext: true, // Derived from playlist length in component
          hasPrevious: true, // Derived from playlist length in component
          isMuted: playerStore.isMuted,
          isPlaying: playerStore.isPlaying,
          isShuffled: playerStore.isShuffled,
          repeatMode: playerStore.repeatMode,
          volume: playerStore.volume,
          playlist: playerStore.playlist,
          progress: playerStore.progress,
        }'
        :server-url="credentials?.serverUrl ?? ''"
        :show='true'
        :time-domain-data='timeDomainData'
        :token="credentials?.token ?? ''"
      />

      <GlobalSearch :open='false' />
    </template>

    <template v-else-if="authStatus === 'loggedOut' || authStatus === 'error'">
      <Login @login='handleLogin' />
    </template>
  </div>
</template>
