<script setup lang="ts">
  import { onMounted, computed } from 'vue'
  import { useColorMode } from '@vueuse/core'
  import { storeToRefs } from 'pinia'
  import type { Credentials } from '@/bindings'
  import { Button } from '@/components/ui/button'
  import Login from './views/LoginView.vue'
  import MusicPlayer from './components/player/MusicPlayer.vue'
  import Queue from './components/player/Queue.vue'
  import Equalizer from './components/player/Equalizer.vue'
  import MainLayout from './components/layout/MainLayout.vue'
  import SearchResults from '@/components/shared/SearchResultsView.vue'
  import FullscreenPlayer from './components/player/FullscreenPlayer.vue'
  import WindowControls from '@/components/shared/WindowControls.vue'

  import { useAuth } from '@/composables/useAuth'
  import { useLibrary } from '@/composables/useLibrary'
  import { useNavigation } from '@/composables/useNavigation'
  import { usePlayerControls } from '@/composables/usePlayerControls'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { usePlayerSession } from '@/composables/usePlayerSession'
  import { appLogger } from '@/lib/logger'

  useColorMode()

  const { authStatus, credentials, error: authError, login, logout, clearError: clearAuthError } = useAuth()
  const {
    allSongs,
    allArtistsWithSongs,
    allAlbums,
    loadLibrary,
    syncLibrary,
    clearCache,
  } = useLibrary()
  const {
    currentView,
    canGoBack,
    canGoForward,
    navigateBack,
    navigateForward,
    handleNavigation,
    navigateToArtist,
    navigateToAlbum,
  } = useNavigation()
  const {
    isQueueOpen,
    isEqualizerOpen,
    isFullScreenPlayerOpen,
    searchQuery,
    isSearchVisible,
    musicPlayerRef,
    handleGlobalSearch,
    toggleQueue,
    toggleEqualizer,
    toggleFullScreenPlayer,
    toggleSearchVisibility,
    handleTogglePlayPause,
    handlePreviousSong,
    handleNextSong,
    handleToggleShuffle,
    handleToggleRepeat,
    handleSeek,
    playerStore,
  } = usePlayerControls()
  const {
    playSong,
    playSongs,
    updatePlaylist,
    removeSongFromPlaylist,
    handleSongChanged,
    handleUpdateCurrentSong,
    toggleFavorite,
  } = useSongInteractions(credentials)

  const {
    currentSong,
    playlist,
    isPlaying,
    currentTime,
    duration,
    isShuffled,
    repeatMode,
    progress,
    hasNext,
    hasPrevious,
  } = storeToRefs(playerStore)

  usePlayerSession()

  onMounted(() => {
    playerStore.setVolume(playerStore.volume)
  })

  const handleLogin = async (loginCredentials: Credentials) => {
    login(loginCredentials)
  }

  const handleLogout = () => {
    logout()
    playerStore.setCurrentSong(null)
    playerStore.setPlaylist([])
    playerStore.setCurrentIndex(-1)
  }

  const handleVolumeChange = (newVolume: number) => {
    playerStore.setVolume(newVolume)
  }

  const handleSyncLibrary = async () => {
    if (!credentials.value) return
    try {
      await syncLibrary(credentials.value)
    } catch (err) {
      appLogger.error('Failed to sync library:', err)
    }
  }

  const handleClearCache = async () => {
    if (!credentials.value) return
    try {
      await clearCache(credentials.value)
    } catch (err) {
      appLogger.error('Failed to clear cache:', err)
    }
  }
</script>

<template>
  <div id='app' class='h-screen bg-background text-foreground'>
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
            @reload-library='() => credentials && loadLibrary(credentials)'
            @select-album='navigateToAlbum'
            @select-artist='navigateToArtist'
            @sync-library='handleSyncLibrary'
            @toggle-favorite='toggleFavorite'
            :key='$route.path'
            :credentials='credentials'
            :current-song='currentSong'
            :is-playing='!!currentSong'
            :server-url='credentials?.serverUrl'
            :token='credentials?.token'
            :user-id='credentials?.userId'
          />
        </transition>
      </router-view>

      <template #search-results='{ onResultClick }'>
        <SearchResults
          @close='toggleSearchVisibility(false)'
          @play-song='playSong'
          @result-clicked='onResultClick'
          :albums='allAlbums as any'
          :artists='allArtistsWithSongs as any'
          :is-visible='isSearchVisible'
          :query='searchQuery'
          :server-url='credentials?.serverUrl'
          :songs='allSongs as any'
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
      </template>

      <template #player>
        <MusicPlayer
          @song-changed='handleSongChanged'
          @toggle-equalizer='toggleEqualizer'
          @toggle-favorite='toggleFavorite'
          @toggle-fullscreen='toggleFullScreenPlayer'
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
