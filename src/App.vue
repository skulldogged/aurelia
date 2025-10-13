<script setup lang="ts">
  import { useColorMode } from '@vueuse/core'
  import { storeToRefs } from 'pinia'
  import { computed, onMounted, watch } from 'vue'
  import { ref } from 'vue'

  import type { Credentials, Song } from '@/bindings'

  import { commands } from '@/bindings'
  import SearchResults from '@/components/shared/SearchResultsView.vue'
  import WindowControls from '@/components/shared/WindowControls.vue'
  import Button from '@/components/ui/Button.vue'
  import { useAuth } from '@/composables/useAuth'
  import { useDiscordPresence } from '@/composables/useDiscordPresence'
  import { useImageLoader } from '@/composables/useImageLoader'
  import { useLastFm } from '@/composables/useLastFm'
  import { useListenBrainz } from '@/composables/useListenBrainz'
  import { useNavigation } from '@/composables/useNavigation'
  import { usePlayerControls } from '@/composables/usePlayerControls'
  import { usePlayerSession } from '@/composables/usePlayerSession'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { useSystemTray } from '@/composables/useSystemTray'
  import { useWebAudioPlayer } from '@/composables/useWebAudioPlayer'
  import { useBlurStore, useLibraryStore } from '@/stores'

  import MainLayout from './components/layout/MainLayout.vue'
  import Equalizer from './components/player/Equalizer.vue'
  import FullscreenPlayer from './components/player/FullscreenPlayer.vue'
  import LyricsSidebar from './components/player/LyricsSidebar.vue'
  import MusicPlayer from './components/player/MusicPlayer.vue'
  import Queue from './components/player/Queue.vue'
  import Login from './views/LoginView.vue'

  useColorMode()

  const { authStatus, clearError: clearAuthError, credentials, error: authError, login, logout } = useAuth()
  const libraryStore = useLibraryStore()
  const blurStore = useBlurStore()
  const { preloadRecentImages } = useImageLoader()
  useSystemTray() // Initialize system tray functionality
  useDiscordPresence()
  useLastFm() // Initialize Last.fm scrobbling
  useListenBrainz() // Initialize ListenBrainz scrobbling

  const webAudioPlayer = useWebAudioPlayer()

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
    handleGlobalSearch,
    handleNextSong,
    handlePreviousSong,
    handleSeek: handleSeek,
    handleTogglePlayPause,
    handleToggleRepeat,
    handleToggleShuffle,
    isEqualizerOpen,
    isFullScreenPlayerOpen,
    isLyricsOpen,
    isQueueOpen,
    isSearchVisible,
    musicPlayerRef,
    playerStore,
    searchQuery,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleLyrics,
    toggleQueue,
    toggleSearchVisibility,
  } = usePlayerControls()

  const {
    handleSongChanged,
    handleUpdateCurrentSong,
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
    visualizerEnabled,
    visualizerStyle,
  } = storeToRefs(playerStore)

  // Compute navigation state locally like MusicPlayer does
  const hasNext = computed(() =>
    playlist.value.length > 1
    && playerStore.currentIndex > -1
    && playerStore.currentIndex < playlist.value.length - 1,
  )
  const hasPrevious = computed(() => playlist.value.length > 1 && playerStore.currentIndex > 0)

  const isSyncing = ref(false)
  const isClearing = ref(false)

  usePlayerSession()

  onMounted(async () => {
    playerStore.setVolume(playerStore.volume)

    await new Promise(resolve => setTimeout(resolve, 100))
    await commands.setBlurMode(blurStore.selectedBlurMode.name)
  })

  watch(authStatus, async newStatus => {
    if (newStatus === 'loggedIn' && credentials.value) {
      await libraryStore.loadLibrary(credentials.value)

      await preloadRecentImages(credentials.value.serverUrl, credentials.value.token, 20)
    }
  })

  watch(authStatus, newStatus => {
    if (newStatus === 'loggedOut')
      libraryStore.clearData()
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

  const handleSyncLibrary = async (): Promise<void> => {
    if (!credentials.value) return
    isSyncing.value = true
    await libraryStore.syncLibrary(credentials.value)
    isSyncing.value = false
  }

  const handleClearCache = async (): Promise<void> => {
    if (!credentials.value) return
    isClearing.value = true
    await libraryStore.clearCache(credentials.value)
    isClearing.value = false
  }

  const handleLyricsLoaded = (hasLyrics: boolean): void => {
    // Pass lyrics availability to player controls
    playerStore.setHasLyrics(hasLyrics)
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
      @global-search='handleGlobalSearch'
      @logout='handleLogout'
      @navigate='handleNavigation'
      @navigate-back='navigateBack'
      @navigate-forward='navigateForward'
      v-else
      :can-go-back='canGoBack'
      :can-go-forward='canGoForward'
      :current-view='currentView'
      :has-player='!!currentSong'
      :is-equalizer-open='isEqualizerOpen'
      :is-lyrics-open='isLyricsOpen'
      :is-queue-open='isQueueOpen'
    >
      <router-view v-slot='{ Component }'>
        <transition mode='out-in' name='page-fade'>
          <component
            :is='Component'
            @clear-cache='handleClearCache'
            @logout='handleLogout'
            @play-instant-mix='playInstantMix'
            @play-song='playSong'
            @play-songs='playSongs'
            @reload-library='() => credentials && libraryStore.loadLibrary(credentials)'
            @select-album='navigateToAlbum'
            @select-artist='navigateToArtist'
            @sync-library='handleSyncLibrary'
            @toggle-favorite='toggleFavorite'
            :key='$route.path'
            :credentials='credentials'
            :current-song='currentSong'
            :is-clearing='isClearing'
            :is-syncing='isSyncing'
          />
        </transition>
      </router-view>

      <template #search-results='{ isSidebarCollapsed, onResultClick }'>
        <SearchResults
          @close='toggleSearchVisibility(false)'
          @play-song='playSong'
          @result-clicked='onResultClick'
          :albums='libraryStore.allAlbums as any'
          :artists='libraryStore.allArtistsWithSongs as any'
          :is-sidebar-collapsed='isSidebarCollapsed'
          :is-visible='isSearchVisible'
          :query='searchQuery'
          :server-url='credentials?.serverUrl'
          :songs='libraryStore.allSongs as any'
          :token='credentials?.token'
        />
      </template>

      <template #queue>
        <Queue
          @remove-song='removeSongFromPlaylist'
          v-if='isQueueOpen'
        />
        <Equalizer v-if='isEqualizerOpen' />
        <LyricsSidebar
          @lyrics-loaded='handleLyricsLoaded'
          @seek='handleSeek'
          v-if='isLyricsOpen'
          :current-song='currentSong as any'
          :current-time='currentTime'
          :duration='duration'
        />
      </template>

      <template #player>
        <MusicPlayer
          @song-changed='handleSongChanged'
          @toggle-equalizer='toggleEqualizer'
          @toggle-favorite='handleToggleFavorite'
          @toggle-fullscreen='toggleFullScreenPlayer'
          @toggle-lyrics='toggleLyrics'
          @toggle-queue='toggleQueue'
          @update-current-song='handleUpdateCurrentSong'
          @volume-changed='handleVolumeChange'
          v-if='currentSong'
          ref='musicPlayerRef'
          :is-equalizer-open='isEqualizerOpen'
          :is-lyrics-open='isLyricsOpen'
          :is-queue-open='isQueueOpen'
          :server-url='credentials!.serverUrl'
          :token='credentials!.token'
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
      @toggle-equalizer='toggleEqualizer'
      @toggle-favorite='handleToggleFavorite'
      @toggle-lyrics='toggleLyrics'
      @toggle-mute='playerStore.toggleMute'
      @toggle-play-pause='handleTogglePlayPause'
      @toggle-queue='toggleQueue'
      @toggle-repeat='handleToggleRepeat'
      @toggle-shuffle='handleToggleShuffle'
      @update:playlist='updatePlaylist'
      @volume-change='handleVolumeChange'
      :analyser-node='webAudioPlayer.getAnalyserNode()'
      :current-time='currentTime'
      :duration='duration'
      :has-next='hasNext'
      :has-previous='hasPrevious'
      :is-equalizer-open='isEqualizerOpen'
      :is-lyrics-open='isLyricsOpen'
      :is-muted='playerStore.isMuted'
      :is-playing='isPlaying'
      :is-queue-open='isQueueOpen'
      :is-shuffled='isShuffled'
      :playlist='playlist'
      :progress='progress'
      :repeat-mode='repeatMode'
      :server-url='credentials?.serverUrl'
      :show='isFullScreenPlayerOpen'
      :song='currentSong as any'
      :token='credentials?.token'
      :visualizer-enabled='visualizerEnabled'
      :visualizer-style='visualizerStyle'
      :volume='playerStore.volume * 100'
    />

    <WindowControls v-if='!isFullScreenPlayerOpen' class='fixed top-0 right-0 z-[100]' />
  </div>
</template>
