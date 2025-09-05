<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useColorMode } from '@vueuse/core'
import { useRouter, useRoute } from 'vue-router'
import Login from './components/Login.vue'
import MusicPlayer from './components/MusicPlayer.vue'
import Queue from './components/Queue.vue'
import MainLayout from './components/MainLayout.vue'
import { invoke } from '@tauri-apps/api/core'
import { MusicItem } from './types'

// Define your interfaces here, or move them to a types.ts file
// For brevity, I'll assume they are defined.
interface Credentials {
  serverUrl: string
  username: string
  token: string
  userId: string
}

useColorMode()

// Router setup
const router = useRouter()
const route = useRoute()

// App state
const isLoggedIn = ref(false)
const credentials = ref<Credentials | null>(null)
const currentSong = ref<MusicItem | null>(null)
const playlist = ref<MusicItem[]>([])
const volume = ref(0.5)
const libraryComponent = ref<any>(null)
const isQueueOpen = ref(false)
const allSongs = ref<MusicItem[]>([])
const allAlbums = ref<any[]>([]) // Define a proper album type later
const allArtists = ref<any[]>([])
const libraryLoading = ref(false)
const libraryError = ref('')

// Current view from route
const currentView = ref('home')

// Watch route changes to update currentView
watch(() => route.name, (newName) => {
  if (newName) {
    currentView.value = newName as string
  }
}, { immediate: true })

// History management for MusicLibrary views
type LibraryHistoryState =
  | { view: 'main' }
  | { view: 'artist'; artist: { id: string, name: string; imageUrl?: string } }
  | { view: 'album'; album: { name: string; artist: string; songCount: number; albumArtUrl?: string } };

const libraryHistory = ref<LibraryHistoryState[]>([{ view: 'main' }])
const libraryHistoryIndex = ref(0)

const canGoBack = computed(() => libraryHistoryIndex.value > 0)
const canGoForward = computed(() => libraryHistoryIndex.value < libraryHistory.value.length - 1)

function navigateBack() {
  if (canGoBack.value) {
    libraryHistoryIndex.value--
    updateLibraryViewFromHistory()
  }
}

function navigateForward() {
  if (canGoForward.value) {
    libraryHistoryIndex.value++
    updateLibraryViewFromHistory()
  }
}

function updateLibraryViewFromHistory() {
  if (libraryComponent.value) {
    const state = libraryHistory.value[libraryHistoryIndex.value]
    libraryComponent.value.updateViewFromHistory(state)
  }
}

watch(libraryHistoryIndex, updateLibraryViewFromHistory)


const handleNavigation = (view: string) => {
  // Map view names to routes
  const routeMap: Record<string, string> = {
    'home': '/',
    'library': '/songs',
    'artists': '/artists',
    'albums': '/albums'
  }

  const routePath = routeMap[view]
  if (routePath) {
    router.push(routePath)
  }
}

const handleGlobalSearch = (query: string) => {
  console.log('Global search query:', query)
}

const handleSelectArtist = (artist: any) => {
  router.push(`/songs/artist/${artist.id}`)
}

const handleSelectAlbum = (album: any) => {
  router.push(`/songs/album/${encodeURIComponent(album.name)}`)
}

const loadLibrary = async () => {
  if (!credentials.value) return
  libraryLoading.value = true
  libraryError.value = ''
  try {
    const [songs, artists] = await Promise.all([
      invoke<MusicItem[]>('get_music_library', {
        serverUrl: credentials.value.serverUrl,
        token: credentials.value.token
      }),
      invoke<any[]>('get_all_artists', {
        serverUrl: credentials.value.serverUrl,
        token: credentials.value.token
      })
    ])
    allSongs.value = songs
    allArtists.value = artists

    // Compute albums from songs
    const albumMap = new Map<string, any>()
    songs.forEach((song) => {
      const albumName = song.album || 'Unknown Album'
      const artist = song.artists?.join(', ') || 'Unknown Artist'
      if (!albumMap.has(albumName)) {
        albumMap.set(albumName, {
          name: albumName,
          artist,
          songCount: 0,
          albumArtUrl: song.albumArtUrl,
        })
      }
      albumMap.get(albumName)!.songCount++
    })
    allAlbums.value = Array.from(albumMap.values()).sort((a, b) => a.name.localeCompare(b.name))

  } catch (err) {
    libraryError.value = err as string
  } finally {
    libraryLoading.value = false
  }
}


// Check for saved credentials on app start
onMounted(async () => {
  // Load saved volume
  try {
    const savedVolume = await invoke<number>('get_saved_volume')
    if (savedVolume !== null) {
      volume.value = savedVolume
    }
  } catch (err) {
    // No saved volume, use default
  }

  try {
    const saved = await invoke<Credentials>('get_saved_credentials')
    if (saved) {
      credentials.value = saved
      isLoggedIn.value = true
      await loadLibrary()
    }
  } catch (err) {
    // No saved credentials, continue to login screen
  }
})

// Save volume whenever it changes
watch(volume, (newVolume: number) => {
  invoke('save_volume', { volume: newVolume })
})

// Handle login success
const handleLogin = async (loginCredentials: Credentials) => {
  credentials.value = loginCredentials
  isLoggedIn.value = true
  await loadLibrary()
}

// Handle logout
const handleLogout = () => {
  credentials.value = null
  isLoggedIn.value = false
  currentSong.value = null
  playlist.value = []
  allSongs.value = []
  allAlbums.value = []
  allArtists.value = []
  currentView.value = 'home'
}

// Handle volume changes from player
const handleVolumeChange = (newVolume: number) => {
  volume.value = newVolume
}

// Handle song play request
const handlePlaySong = (song: MusicItem) => {
  currentSong.value = song
  if (!playlist.value.find((s: MusicItem) => s.id === song.id)) {
    playlist.value.push(song)
  }
  // Only call updateCurrentSong if the component is mounted and has the method
  if (libraryComponent.value && typeof libraryComponent.value.updateCurrentSong === 'function') {
    libraryComponent.value.updateCurrentSong(song, true)
  }
}

// Handle playing a full album or any list of songs
const handlePlaySongs = (songs: MusicItem[]) => {
  playlist.value = songs
  if (songs.length > 0) {
    handlePlaySong(songs[0])
  }
}

// Handle playlist updates from queue
const handleUpdatePlaylist = (newPlaylist: MusicItem[]) => {
  playlist.value = newPlaylist
}

const handleRemoveSong = (song: MusicItem) => {
  playlist.value = playlist.value.filter(s => s.id !== song.id)
}

const handleToggleQueue = () => {
  isQueueOpen.value = !isQueueOpen.value
}

// Handle song change from player
const handleSongChanged = (song: MusicItem) => {
  currentSong.value = song

  // Update library component
  if (libraryComponent.value) {
    libraryComponent.value.updateCurrentSong(song, true)
  }
}

// Handle player state updates
const handleUpdateCurrentSong = (song: MusicItem | null, isPlaying: boolean) => {
  currentSong.value = song

  // Update library component only if it's mounted and has the method
  if (libraryComponent.value && typeof libraryComponent.value.updateCurrentSong === 'function') {
    libraryComponent.value.updateCurrentSong(song, isPlaying)
  }
}

const handleToggleFavorite = async (song: MusicItem) => {
  if (!credentials.value) return
  // Optimistically update the UI
  const originalStatus = song.isFavorite
  const songIndex = allSongs.value.findIndex(s => s.id === song.id)
  if (songIndex !== -1) {
    allSongs.value[songIndex].isFavorite = !originalStatus
  }

  try {
    const newStatus = await invoke<boolean>('toggle_favorite_status', {
      serverUrl: credentials.value.serverUrl,
      token: credentials.value.token,
      userId: credentials.value.userId,
      itemId: song.id,
      isFavorite: originalStatus,
    })
    // Confirm the state from the backend's response
    if (songIndex !== -1) {
      allSongs.value[songIndex].isFavorite = newStatus
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    // Revert the change if the API call fails
    if (songIndex !== -1) {
      allSongs.value[songIndex].isFavorite = originalStatus
    }
  }
}

</script>

<template>
  <div id="app" class="h-screen bg-background text-foreground">
    <Login v-if="!isLoggedIn" @login="handleLogin" />
    <MainLayout v-else :current-view="currentView" :can-go-back="canGoBack" :can-go-forward="canGoForward"
      @navigate="handleNavigation" @navigate-back="navigateBack" @navigate-forward="navigateForward"
      @logout="handleLogout" @global-search="handleGlobalSearch">
      <router-view v-slot="{ Component }">
        <component :is="Component" :songs="allSongs" :albums="allAlbums" :artists="allArtists"
          :server-url="credentials?.serverUrl" :token="credentials?.token" :user-id="credentials?.userId"
          :loading="libraryLoading" :error="libraryError" :current-song="currentSong" :is-playing="!!currentSong"
          @play-songs="handlePlaySongs" @select-album="handleSelectAlbum" @play-song="handlePlaySong"
          @select-artist="handleSelectArtist" @logout="handleLogout" @reload-library="loadLibrary"
          @toggle-favorite="handleToggleFavorite" ref="libraryComponent" />
      </router-view>

      <template #player>
        <MusicPlayer v-if="currentSong" :key="currentSong.id" :current-song="currentSong"
          :server-url="credentials!.serverUrl" :token="credentials!.token" :playlist="playlist" :volume="volume"
          @song-changed="handleSongChanged" @update-current-song="handleUpdateCurrentSong"
          @volume-changed="handleVolumeChange" @toggle-queue="handleToggleQueue" />
        <Queue v-model="isQueueOpen" :playlist="playlist" :current-song="currentSong" @play-song="handlePlaySong"
          @update:playlist="handleUpdatePlaylist" @remove-song="handleRemoveSong" />
      </template>
    </MainLayout>
  </div>
</template>
