<script setup lang="ts">
  import { onMounted } from 'vue'
  import { useColorMode } from '@vueuse/core'
  import type { Credentials } from '@/bindings'
  import { Button } from '@/components/ui/button'
  import Login from './views/LoginView.vue'
  import MusicPlayer from './components/player/MusicPlayer.vue'
  import Queue from './components/player/Queue.vue'
  import MainLayout from './components/layout/MainLayout.vue'
  import SearchResults from '@/components/shared/SearchResultsView.vue'
  import FullscreenPlayer from './components/player/FullscreenPlayer.vue'
  import WindowControls from '@/components/shared/WindowControls.vue'

  // Centralized app state
  import { useAppState } from '@/composables/useAppState'
  import { usePlayerSession } from '@/composables/usePlayerSession'
  import { appLogger } from '@/lib/logger'

  useColorMode()

  // Initialize player session management (for Jellyfin reporting)
  usePlayerSession()

  // Initialize centralized app state
  const {
    appState,
    login,
    logout,
    clearAuthError,
    loadLibrary,
    syncLibrary,
    clearCache,
    navigateBack,
    navigateForward,
    handleNavigation,
    navigateToArtist,
    navigateToAlbum,
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
    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,
    playerStore,
    musicPlayerRef,
  } = useAppState()

  // Initialize player session management (for Jellyfin reporting)
  // Must be called after useAppState so auth is available
  usePlayerSession()

  // Initialize volume from player store
  onMounted(() => {
    playerStore.setVolume(playerStore.volume)
  })

  // Handle login success
  const handleLogin = async (loginCredentials: Credentials) => {
    login(loginCredentials)
  }

  // Handle logout
  const handleLogout = () => {
    logout()
    // Reset player store state
    playerStore.setCurrentSong(null)
    playerStore.setPlaylist([])
    playerStore.setCurrentIndex(-1)
  }

  // Handle volume changes from player
  const handleVolumeChange = (newVolume: number) => {
    playerStore.setVolume(newVolume)
  }

  // Library management handlers
  const handleSyncLibrary = async () => {
    if (!appState.credentials) return
    try {
      await syncLibrary(appState.credentials)
    } catch (err) {
      appLogger.error('Failed to sync library:', err)
    }
  }

  const handleClearCache = async () => {
    if (!appState.credentials) return
    try {
      await clearCache(appState.credentials)
    } catch (err) {
      appLogger.error('Failed to clear cache:', err)
    }
  }
</script>

<template>
  <div id='app' class='h-screen bg-background text-foreground'>
    <div v-if="appState.authStatus === 'pending'" class='h-full w-full flex items-center justify-center'>
      <div class='text-center'>
        <div class='animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4' />
        <p class='text-muted-foreground'>
          Loading...
        </p>
      </div>
    </div>
    <div v-else-if="appState.authStatus === 'error'" class='h-full w-full flex items-center justify-center'>
      <div class='text-center max-w-md mx-auto p-8'>
        <div class='text-red-500 text-6xl mb-4'>
          ⚠️
        </div>
        <h2 class='text-xl font-semibold mb-2'>
          Connection Error
        </h2>
        <p class='text-muted-foreground mb-4'>
          {{ appState.authError?.message || 'Failed to connect to server' }}
        </p>
        <Button @click='clearAuthError' variant='outline'>
          Try Again
        </Button>
      </div>
    </div>
    <Login @login='handleLogin' v-else-if="appState.authStatus === 'loggedOut'" />
    <MainLayout
      @global-search='handleGlobalSearch'
      @logout='handleLogout'
      @navigate='handleNavigation'
      @navigate-back='navigateBack'
      @navigate-forward='navigateForward'
      v-else
      :can-go-back='appState.canGoBack'
      :can-go-forward='appState.canGoForward'
      :current-view='appState.currentView'
      :has-player='!!appState.currentSong'
      :is-queue-open='appState.isQueueOpen'
    >
      <router-view v-slot='{ Component }'>
        <transition mode='out-in' name='page-fade'>
          <component
            :is='Component'
            @clear-cache='handleClearCache'
            @logout='handleLogout'
            @play-song='playSong'
            @play-songs='playSongs'
            @reload-library='() => appState.credentials && loadLibrary(appState.credentials)'
            @select-album='navigateToAlbum'
            @select-artist='navigateToArtist'
            @sync-library='handleSyncLibrary'
            @toggle-favorite='toggleFavorite'
            :key='$route.path'
            :credentials='appState.credentials'
            :current-song='appState.currentSong'
            :is-playing='!!appState.currentSong'
            :server-url='appState.credentials?.serverUrl'
            :token='appState.credentials?.token'
            :user-id='appState.credentials?.userId'
          />
        </transition>
      </router-view>

      <template #search-results='{ onResultClick }'>
        <SearchResults
          @close='toggleSearchVisibility(false)'
          @play-song='playSong'
          @result-clicked='onResultClick'
          :albums='appState.allAlbums as any'
          :artists='appState.allArtistsWithSongs as any'
          :is-visible='appState.isSearchVisible'
          :query='appState.searchQuery'
          :server-url='appState.credentials?.serverUrl'
          :songs='appState.allSongs as any'
          :token='appState.credentials?.token'
        />
      </template>

      <template #queue>
        <Queue
          @play-song='playSong'
          @remove-song='removeSongFromPlaylist'
          @update:playlist='updatePlaylist'
          v-if='appState.isQueueOpen'
          :current-song='appState.currentSong as any'
          :playlist='appState.playlist as any'
        />
      </template>

      <template #player>
        <MusicPlayer
          @song-changed='handleSongChanged'
          @toggle-favorite='toggleFavorite'
          @toggle-fullscreen='toggleFullScreenPlayer'
          @toggle-queue='toggleQueue'
          @update-current-song='handleUpdateCurrentSong'
          @volume-changed='handleVolumeChange'
          v-if='appState.currentSong'
          ref='musicPlayerRef'
          :server-url='appState.credentials!.serverUrl'
          :token='appState.credentials!.token'
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
      :current-time='appState.currentTime'
      :duration='appState.duration'
      :has-next='appState.hasNext'
      :has-previous='appState.hasPrevious'
      :is-playing='appState.isPlaying'
      :is-shuffled='appState.isShuffled'
      :playlist='appState.playlist'
      :progress='appState.progress'
      :repeat-mode='appState.repeatMode'
      :server-url='appState.credentials?.serverUrl'
      :show='appState.isFullScreenPlayerOpen'
      :song='appState.currentSong as any'
      :token='appState.credentials?.token'
    />

    <WindowControls v-if='!appState.isFullScreenPlayerOpen' class='fixed top-0 right-0 z-[100]' />
  </div>
</template>
