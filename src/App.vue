<script setup lang="ts">
  import { ref, onMounted, watch, onUnmounted } from 'vue'
  import { useColorMode } from '@vueuse/core'
  import { useRouter, useRoute } from 'vue-router'
  import Login from './views/LoginView.vue'
  import MusicPlayer from './components/player/MusicPlayer.vue'
  import Queue from './components/player/Queue.vue'
  import MainLayout from './components/layout/MainLayout.vue'
  import { MusicItem, ArtistInfo, AlbumWithSongs, ArtistWithSongs } from './types'
  import { useTauri } from '@/composables/useTauri'
  import SearchResults from '@/views/SearchResultsView.vue'
  import FullscreenPlayer from './components/player/FullscreenPlayer.vue'
  import { usePlayerStore } from '@/stores'
  import WindowControls from '@/components/shared/WindowControls.vue'

  // Define your interfaces here, or move them to a types.ts file
  // For brevity, I'll assume they are defined.
  interface Credentials {
    serverUrl: string
    username:  string
    token:     string
    userId:    string
  }

  interface ArtistSummary {
    id:        string
    name:      string
    songCount: number
    imageUrl?: string
  }

  useColorMode()

  // Router setup
  const router = useRouter()
  const route = useRoute()

  // Tauri commands
  const {
    getMusicLibrary,
    getAlbumsWithSongs,
    getArtistsWithSongs,
    getSavedCredentials,
    toggleFavoriteStatus,
    clearMusicCache,
  } = useTauri()

  // App state
  const authStatus = ref<'pending' | 'loggedIn' | 'loggedOut'>('pending')
  const credentials = ref<Credentials | null>(null)
  const currentSong = ref<MusicItem | null>(null)
  const playlist = ref<MusicItem[]>([])
  const volume = ref(0.5)
  const isQueueOpen = ref(false)
  const allSongs = ref<MusicItem[]>([])
  const allAlbums = ref<AlbumWithSongs[]>([])
  const allArtists = ref<ArtistInfo[]>([])
  const allArtistsWithSongs = ref<ArtistWithSongs[]>([])
  const albumArtistsWithSongs = ref<ArtistWithSongs[]>([])
  const libraryLoading = ref(false)
  const libraryError = ref('')
  const searchQuery = ref('')
  const isSearchVisible = ref(false)
  const isFullScreenPlayerOpen = ref(false)
  const musicPlayerRef = ref<InstanceType<typeof MusicPlayer> | null>(null)

  const playerStore = usePlayerStore()

  // Current view from route
  const currentView = ref('home')

  // Watch route changes to update currentView
  watch(() => route.name, newName => {
    if (newName)
      currentView.value = newName as string
  }, { immediate: true })

  const canGoBack = ref(false)
  const canGoForward = ref(false)

  const updateNavState = () => {
    // vue-router uses the history state's position to track navigation
    canGoBack.value = window.history.state.position > 0
    canGoForward.value = window.history.state.position < window.history.length - 1
  }

  router.afterEach(() => {
    updateNavState()
  })

  window.addEventListener('popstate', updateNavState)

  onMounted(() => {
    updateNavState()
  })

  onUnmounted(() => {
    window.removeEventListener('popstate', updateNavState)
  })

  const navigateBack = () => {
    router.back()
  }

  const navigateForward = () => {
    router.forward()
  }

  const handleGlobalSearch = (query: string) => {
    searchQuery.value = query
    isSearchVisible.value = true
  }

  const handleNavigation = (view: string) => {
    // Map view names to routes
    const routeMap: Record<string, string> = {
      'home':    '/',
      'library': '/songs',
      'artists': '/artists',
      'albums':  '/albums',
    }

    const routePath = routeMap[view]
    if (routePath) {
      router.push(routePath)
    }
  }

  const handleSelectArtist = (artist: ArtistSummary) => {
    router.push(`/songs/artist/${artist.id}`)
  }

  const handleSelectAlbum = (album: AlbumWithSongs) => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
  }

  const loadLibrary = async () => {
    console.log('DEBUG: loadLibrary called')
    if (!credentials.value) {
      console.log('DEBUG: No credentials, skipping loadLibrary')
      return
    }
    libraryLoading.value = true
    libraryError.value = ''
    try {
      console.log('DEBUG: Fetching songs...')
      const songs = await getMusicLibrary(
        credentials.value.serverUrl,
        credentials.value.token,
      )
      console.log(`DEBUG: Loaded ${songs.length} songs`)
      allSongs.value = songs

      // Since the other data is derived from songs, let's get them with the new commands
      console.log('DEBUG: Fetching albums and artists...')
      const [albums, artistsWithSongs, albumArtists] = await Promise.all([
        getAlbumsWithSongs(credentials.value.serverUrl, credentials.value.token),
        getArtistsWithSongs(credentials.value.serverUrl, credentials.value.token, false),
        getArtistsWithSongs(credentials.value.serverUrl, credentials.value.token, true),
      ])
      console.log(`DEBUG: Loaded ${albums.length} albums and ${artistsWithSongs.length} artists with songs`)
      allAlbums.value = albums
      allArtistsWithSongs.value = artistsWithSongs
      albumArtistsWithSongs.value = albumArtists
      console.log('DEBUG: Library load completed successfully')
    } catch (err) {
      console.error('DEBUG: Library load failed:', err)
      libraryError.value = err as string
    } finally {
      libraryLoading.value = false
    }
  }

  // Check for saved credentials on app start
  onMounted(() => {
    // Initialize volume from player store (which loads from localStorage)
    volume.value = playerStore.volume

    getSavedCredentials()
      .then(saved => {
        if (saved && saved.token) {
          credentials.value = saved
          authStatus.value = 'loggedIn'
        } else {
          authStatus.value = 'loggedOut'
        }
      })
      .catch(err => {
        console.error('Failed to get saved credentials:', err)
        authStatus.value = 'loggedOut'
      })
  })

  watch(authStatus, (status: string) => {
    if (status === 'loggedIn')
      loadLibrary()
  })

  // Handle login success
  const handleLogin = async (loginCredentials: Credentials) => {
    credentials.value = loginCredentials
    authStatus.value = 'loggedIn'
    await loadLibrary()
  }

  // Handle logout
  const handleLogout = () => {
    credentials.value = null
    authStatus.value = 'loggedOut'
    // Reset player store state
    playerStore.setCurrentSong(null)
    playerStore.setPlaylist([])
    playerStore.setCurrentIndex(-1)
    // Reset local state
    currentSong.value = null
    playlist.value = []
    allSongs.value = []
    allAlbums.value = []
    allArtists.value = []
    allArtistsWithSongs.value = []
    albumArtistsWithSongs.value = []
    currentView.value = 'home'
  }

  // Handle volume changes from player
  const handleVolumeChange = (newVolume: number) => {
    volume.value = newVolume
    // Player store setVolume already handles persistence
    playerStore.setVolume(newVolume)
  }

  const handlePlaySong = (song: MusicItem) => {
    currentSong.value = song
    playerStore.setCurrentSong(song)
    if (!playlist.value.find((s: MusicItem) => s.id === song.id)) {
      playlist.value.push(song)
      playerStore.setPlaylist([...playlist.value])
    }
    // Update current index in store
    const index = playlist.value.findIndex(s => s.id === song.id)
    if (index !== -1) {
      playerStore.setCurrentIndex(index)
    }
  }

  // Handle playing a full album or any list of songs
  const handlePlaySongs = (songs: MusicItem[]) => {
    playlist.value = songs
    playerStore.setPlaylist(songs)
    if (songs.length > 0) {
      handlePlaySong(songs[0])
    }
  }

  // Handle playlist updates from queue
  const handleUpdatePlaylist = (newPlaylist: MusicItem[]) => {
    playlist.value = newPlaylist
    playerStore.setPlaylist(newPlaylist)
    // Update current index if current song is still in playlist
    if (currentSong.value) {
      const index = newPlaylist.findIndex(s => s.id === currentSong.value!.id)
      playerStore.setCurrentIndex(index)
    }
  }

  const handleRemoveSong = (song: MusicItem) => {
    playlist.value = playlist.value.filter(s => s.id !== song.id)
  }

  const handleToggleQueue = () => {
    isQueueOpen.value = !isQueueOpen.value
  }

  const handleToggleFullScreenPlayer = () => {
    isFullScreenPlayerOpen.value = !isFullScreenPlayerOpen.value
  }

  const handleTogglePlayPause = () => {
    // Only use the MusicPlayer's method - it handles both audio control and store state updates
    musicPlayerRef.value?.togglePlayPause()
  }
  const handlePreviousSong = () => {
    playerStore.previousSong()
    musicPlayerRef.value?.previousSong()
  }
  const handleNextSong = () => {
    playerStore.nextSong()
    musicPlayerRef.value?.nextSong()
  }
  const handleToggleShuffle = () => {
    playerStore.toggleShuffle()
  }
  const handleToggleRepeat = () => {
    playerStore.cycleRepeatMode()
  }
  const handleSeek = (value: number) => musicPlayerRef.value?.onSeek([value])

  // Handle song change from player
  const handleSongChanged = (song: MusicItem) => {
    currentSong.value = song
    playerStore.setCurrentSong(song)
  }

  // Handle player state updates
  const handleUpdateCurrentSong = (song: MusicItem | null) => {
    currentSong.value = song
    playerStore.setCurrentSong(song)
  }

  // Favorite a song
  const handleToggleFavorite = (song: MusicItem) => {
    console.log('Toggling favorite for song:', song.name, 'Current status:', song.isFavorite)
    if (!credentials.value) {
      console.error('Cannot toggle favorite: no credentials')
      return
    }

    toggleFavoriteStatus(
      credentials.value.serverUrl,
      credentials.value.token,
      credentials.value.userId,
      song.id,
      !song.isFavorite,
    )
      .then(newStatus => {
        console.log('Favorite status updated to:', newStatus)
        const songInLibrary = allSongs.value.find(s => s.id === song.id)
        if (songInLibrary)
          songInLibrary.isFavorite = newStatus
        const songInPlaylist = playlist.value.find(s => s.id === song.id)
        if (songInPlaylist)
          songInPlaylist.isFavorite = newStatus
        if (currentSong.value && currentSong.value.id === song.id)
          currentSong.value.isFavorite = newStatus
      })
      .catch(err => {
        console.error('Failed to toggle favorite status:', err)
      })
  }

  const handleClearCache = async () => {
    console.log('DEBUG: Starting cache clear from UI...')
    try {
      await clearMusicCache()
      console.log('DEBUG: Cache clear command completed, now loading library...')
      await loadLibrary()
      console.log('DEBUG: Library reload completed')
    } catch (err) {
      console.error('Failed to clear cache:', err)
    }
  }

</script>

<template>
  <div id='app' class='h-screen bg-background text-foreground'>
    <div v-if="authStatus === 'pending'" class='h-full w-full' />
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
      :is-queue-open='isQueueOpen'
    >
      <router-view v-slot='{ Component }'>
        <transition mode='out-in' name='page-fade'>
          <component
            :is='Component'
            @clear-cache='handleClearCache'
            @logout='handleLogout'
            @play-song='handlePlaySong'
            @play-songs='handlePlaySongs'
            @reload-library='loadLibrary'
            @select-album='handleSelectAlbum'
            @select-artist='handleSelectArtist'
            @toggle-favorite='handleToggleFavorite'
            :key='$route.path'
            :album-artists='albumArtistsWithSongs'
            :albums='allAlbums'
            :artists='allArtistsWithSongs'
            :credentials='credentials'
            :current-song='currentSong'
            :error='libraryError'
            :is-playing='!!currentSong'
            :loading='libraryLoading'
            :server-url='credentials?.serverUrl'
            :songs='allSongs'
            :token='credentials?.token'
            :user-id='credentials?.userId'
          />
        </transition>
      </router-view>

      <template #search-results='{ onResultClick }'>
        <SearchResults
          @close='isSearchVisible = false'
          @play-song='handlePlaySong'
          @result-clicked='onResultClick'
          :albums='allAlbums'
          :artists='allArtists'
          :is-visible='isSearchVisible'
          :query='searchQuery'
          :songs='allSongs'
        />
      </template>

      <template #queue>
        <Queue
          @play-song='handlePlaySong'
          @remove-song='handleRemoveSong'
          @update:playlist='handleUpdatePlaylist'
          v-if='isQueueOpen'
          :current-song='currentSong'
          :playlist='playlist'
        />
      </template>

      <template #player>
        <MusicPlayer
          @song-changed='handleSongChanged'
          @toggle-favorite='handleToggleFavorite'
          @toggle-fullscreen='handleToggleFullScreenPlayer'
          @toggle-queue='handleToggleQueue'
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
      @close='handleToggleFullScreenPlayer'
      @next-song='handleNextSong'
      @play-song='handlePlaySong'
      @previous-song='handlePreviousSong'
      @remove-song='handleRemoveSong'
      @seek='handleSeek'
      @toggle-play-pause='handleTogglePlayPause'
      @toggle-repeat='handleToggleRepeat'
      @toggle-shuffle='handleToggleShuffle'
      @update:playlist='handleUpdatePlaylist'
      :current-time='playerStore.currentTime'
      :duration='playerStore.duration'
      :has-next='playerStore.hasNext'
      :has-previous='playerStore.hasPrevious'
      :is-playing='playerStore.isPlaying'
      :is-shuffled='playerStore.isShuffled'
      :playlist='playlist'
      :progress='playerStore.progress'
      :repeat-mode='playerStore.repeatMode'
      :show='isFullScreenPlayerOpen'
      :song='currentSong'
    />

    <WindowControls v-if='!isFullScreenPlayerOpen' class='fixed top-0 right-0 z-[100]' />
  </div>
</template>
