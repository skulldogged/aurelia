<script setup lang="ts">
  import { useColorMode } from '@vueuse/core'
  import { storeToRefs } from 'pinia'
  import { onMounted, watch } from 'vue'
  import { ref } from 'vue'

  import type { Credentials, Song } from '@/bindings'

  import { commands } from '@/bindings'
  import SearchResults from '@/components/shared/SearchResultsView.vue'
  import WindowControls from '@/components/shared/WindowControls.vue'
  import { Button } from '@/components/ui/button'
  import { useAuth } from '@/composables/useAuth'
  import { useImageLoader } from '@/composables/useImageLoader'
  import { useNavigation } from '@/composables/useNavigation'
  import { usePlayerControls } from '@/composables/usePlayerControls'
  import { usePlayerSession } from '@/composables/usePlayerSession'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { useSystemTray } from '@/composables/useSystemTray'
  import { appLogger } from '@/lib/logger'
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
    hasNext,
    hasPrevious,
    isPlaying,
    isShuffled,
    playlist,
    progress,
    repeatMode,
  } = storeToRefs(playerStore)

  const isSyncing = ref(false)
  const isClearing = ref(false)

  usePlayerSession()

  onMounted(async () => {
    playerStore.setVolume(playerStore.volume)

    // Apply saved blur mode when app loads
    try {
      // Small delay to ensure Tauri window is fully initialized
      await new Promise(resolve => setTimeout(resolve, 100))
      await commands.setBlurMode(blurStore.selectedBlurMode.name)
    } catch (error) {
      console.error('Failed to apply initial blur mode:', error)
    }
  })

  // Load library data when user becomes logged in
  watch(authStatus, async newStatus => {
    if (newStatus === 'loggedIn' && credentials.value) {
      try {
        await libraryStore.loadLibrary(credentials.value)

        // Preload recent images for better performance
        try {
          await preloadRecentImages(credentials.value.serverUrl, credentials.value.token, 20)
          appLogger.info('Preloaded recent images for better performance')
        } catch (err) {
          appLogger.warn('Failed to preload images:', err)
        }
      } catch (err) {
        appLogger.error('Failed to load library on auth:', err)
      }
    }
  })

  // Clear library data on logout
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

  const handleToggleFavorite = (song: Song): void => {
    toggleFavorite(song)
  }

  const handleVolumeChange = (newVolume: number): void => {
    playerStore.setVolume(newVolume)
  }

  const handleSyncLibrary = async (): Promise<void> => {
    if (!credentials.value) return
    isSyncing.value = true
    try {
      await libraryStore.syncLibrary(credentials.value)
    } catch (err) {
      appLogger.error('Failed to sync library:', err)
    } finally {
      isSyncing.value = false
    }
  }

  const handleClearCache = async (): Promise<void> => {
    if (!credentials.value) return
    isClearing.value = true
    try {
      await libraryStore.clearCache(credentials.value)
    } catch (err) {
      appLogger.error('Failed to clear cache:', err)
    } finally {
      isClearing.value = false
    }
  }
</script>

<template>
  <div id='app' class='h-screen text-foreground'>
    <div v-if="authStatus === 'pending'" class='h-full w-full flex items-center justify-center'>
      <div class='text-center'>
        <div class='animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4' />
        <p class='text-muted-foreground'>
          Loading...
        </p>
      </div>
    </div>
    <div v-else-if="authStatus === 'error'" class='h-full w-full flex items-center justify-center'>
      <div class='text-center max-w-md mx-auto p-8'>
        <div class='text-red-500 text-6xl mb-4'>
          ⚠️
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
            @play-song='playSong'
            @play-songs='playSongs'
            @reload-library='() => credentials && libraryStore.loadLibrary(credentials)'
            @select-album='navigateToAlbum'
            @select-artist='navigateToArtist'
            @sync-library='handleSyncLibrary'
            @toggle-favorite='toggleFavorite'
            :key='$route.path'
            :album-artists='libraryStore.albumArtistsWithSongs'
            :all-albums='libraryStore.allAlbums'
            :all-artists='libraryStore.allArtistsWithSongs'
            :all-songs='libraryStore.allSongs'
            :credentials='credentials'
            :current-song='currentSong'
            :is-clearing='isClearing'
            :is-playing='!!currentSong'
            :is-syncing='isSyncing'
            :library-loaded='libraryStore.isLoaded'
            :library-loading='libraryStore.isLoading'
            :server-url='credentials?.serverUrl'
            :token='credentials?.token'
            :user-id='credentials?.userId'
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
          @play-song='playSong'
          @remove-song='removeSongFromPlaylist'
          @update:playlist='updatePlaylist'
          v-if='isQueueOpen'
          :current-song='currentSong as any'
          :playlist='playlist as any'
        />
        <Equalizer v-if='isEqualizerOpen' />
        <LyricsSidebar
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
      @toggle-play-pause='handleTogglePlayPause'
      @toggle-repeat='handleToggleRepeat'
      @toggle-shuffle='handleToggleShuffle'
      @update:playlist='updatePlaylist'
      :current-time='currentTime'
      :duration='duration'
      :has-next='hasNext'
      :has-previous='hasPrevious'
      :is-playing='isPlaying'
      :is-shuffled='isShuffled'
      :playlist='playlist'
      :progress='progress'
      :repeat-mode='repeatMode'
      :server-url='credentials?.serverUrl'
      :show='isFullScreenPlayerOpen'
      :song='currentSong as any'
      :token='credentials?.token'
    />

    <WindowControls v-if='!isFullScreenPlayerOpen' class='fixed top-0 right-0 z-[100]' />
  </div>
</template>
