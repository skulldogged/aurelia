<script setup lang="ts">
import { useColorMode } from '@vueuse/core'
import { computed, onMounted, ref, watch } from 'vue'

import type { Credentials } from '@shared'

import {
  MainLayout,
  MusicPlayer,
  FullscreenPlayer,
  Queue,
  Equalizer,
  LyricsSidebar,
  GlobalSearch,
  Login,
  Toaster,
  getApiClient,
  useAuth,
  useAudioEngine,
  useNavigation,
  usePlayerControls,
  usePlayerSession,
  useSongInteractions,
  useTopBar,
  useHomeStore,
  useLibraryStore,
  usePlayerStore,
  useThemeStore,
  useAccentColorStore,
} from '@shared'

useColorMode()
useThemeStore() // Initialize theme and apply CSS variables
useAccentColorStore() // Initialize accent colors and apply CSS variables

const { authStatus, clearError: clearAuthError, credentials, error: authError, login, logout } = useAuth()
const libraryStore = useLibraryStore()
const homeStore = useHomeStore()
const playerStore = usePlayerStore()

const navigation = useNavigation()
const playerControls = usePlayerControls()
const songInteractions = useSongInteractions(credentials)
const topBar = useTopBar()

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

const isAuthenticated = computed(() => authStatus.value === 'loggedIn')

const showLogoutConfirm = ref(false)
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

const handleLogin = (creds: Credentials) => {
  login(creds)
  // Watcher will handle sync
}

const handleLogout = () => {
  logout()
  showLogoutConfirm.value = false
}

const handleQuit = () => {
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
  
  <div class="h-screen w-screen overflow-hidden bg-background text-foreground">
    <template v-if="isAuthenticated">
      <MainLayout
        :navigation-state="{
          canGoBack: navigation.canGoBack.value,
          canGoForward: navigation.canGoForward.value,
          currentView: navigation.currentView.value,
        }"
        :player-state="{
          hasPlayer: true,
          isEqualizerOpen: playerControls.isEqualizerOpen.value,
          isLyricsOpen: playerControls.isLyricsOpen.value,
          isQueueOpen: playerControls.isQueueOpen.value,
        }"
        @logout="showLogoutConfirm = true"
        @quit="handleQuit"
        @navigate-back="navigation.navigateBack"
        @navigate-forward="navigation.navigateForward"
        @navigate="navigation.handleNavigation"
      >
        <RouterView v-slot="{ Component }">
          <component
            :is="Component"
            :credentials="credentials"
            :current-song="playerStore.currentSong"
            :is-clearing="isClearing"
            :is-syncing="isSyncing"
            :last-sync-time="lastSyncTime"
            @play-song="songInteractions.playSong"
            @play-songs="songInteractions.playSongs"
            @play-instant-mix="songInteractions.playInstantMix"
            @toggle-favorite="songInteractions.toggleFavorite"
            @select-album="navigation.navigateToAlbum"
            @select-artist="navigation.navigateToArtist"
            @sync-library="handleSyncLibrary"
            @clear-cache="handleClearCache"
          />
        </RouterView>

        <template #queue>
          <div class="h-full w-full overflow-hidden">
            <Queue
              v-if="playerControls.isQueueOpen.value"
              @remove-song="songInteractions.removeSongFromPlaylist"
            />
            <Equalizer
              v-else-if="playerControls.isEqualizerOpen.value"
            />
            <LyricsSidebar
              v-else-if="playerControls.isLyricsOpen.value"
              :current-song="playerStore.currentSong"
              :current-time="playerStore.currentTime"
              :duration="playerStore.duration"
              @seek="playerControls.handleSeek"
            />
          </div>
        </template>

        <template #player>
          <MusicPlayer
            :server-url="credentials?.serverUrl ?? ''"
            :token="credentials?.token ?? ''"
            :is-queue-open="playerControls.isQueueOpen.value"
            :is-equalizer-open="playerControls.isEqualizerOpen.value"
            :is-lyrics-open="playerControls.isLyricsOpen.value"
            @toggle-favorite="songInteractions.toggleFavorite"
            @instant-mix="songInteractions.playInstantMix"
            @toggle-queue="playerControls.toggleQueue"
            @toggle-lyrics="playerControls.toggleLyrics"
            @toggle-equalizer="playerControls.toggleEqualizer"
            @toggle-fullscreen="playerControls.toggleFullScreenPlayer"
            @add-to-playlist="() => {}"
          />
        </template>
      </MainLayout>
      
      <FullscreenPlayer
        v-if="playerControls.isFullScreenPlayerOpen.value"
        :player-state="{
          currentSong: playerStore.currentSong,
          currentTime: playerStore.currentTime,
          duration: playerStore.duration,
          hasNext: true, // Derived from playlist length in component
          hasPrevious: true, // Derived from playlist length in component
          isBuffering: playerStore.isBuffering,
          isMuted: playerStore.isMuted,
          isPlaying: playerStore.isPlaying,
          isShuffled: playerStore.isShuffled,
          repeatMode: playerStore.repeatMode,
          volume: playerStore.volume,
        }"
        :server-url="credentials?.serverUrl ?? ''"
        :show="true"
        :token="credentials?.token ?? ''"
        :is-equalizer-open="playerControls.isEqualizerOpen.value"
        :is-lyrics-open="playerControls.isLyricsOpen.value"
        :is-queue-open="playerControls.isQueueOpen.value"
        @close="playerControls.toggleFullScreenPlayer"
        @toggle-favorite="songInteractions.toggleFavorite"
        @instant-mix="songInteractions.playInstantMix"
        @toggle-queue="playerControls.toggleQueue"
        @toggle-lyrics="playerControls.toggleLyrics"
        @toggle-equalizer="playerControls.toggleEqualizer"
        @toggle-fullscreen="playerControls.toggleFullScreenPlayer"
        @toggle-play-pause="handleTogglePlayPause"
        @previous-song="handlePreviousSong"
        @next-song="handleNextSong"
        @toggle-shuffle="handleToggleShuffle"
        @toggle-repeat="handleToggleRepeat"
        @seek="handleSeek"
        @volume-change="handleVolumeChange"
        @toggle-mute="handleToggleMute"
      />

      <GlobalSearch :open="false" />
    </template>
    
    <template v-else>
      <Login
        :auth-error="authError || undefined"
        :auth-status="authStatus"
        @clear-error="clearAuthError"
        @login="handleLogin"
      />
    </template>
  </div>
</template>
