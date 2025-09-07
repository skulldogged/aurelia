<script setup lang="ts">
  import { ref, onMounted, watch, onUnmounted } from 'vue'
  import { useColorMode } from '@vueuse/core'
  import { useRouter, useRoute } from 'vue-router'
  import Login from './views/LoginView.vue'
  import MusicPlayer from './components/player/MusicPlayer.vue'
  import Queue from './components/player/Queue.vue'
  import MainLayout from './components/layout/MainLayout.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { MusicItem, ArtistInfo, AlbumWithSongs, ArtistWithSongs } from './types'
  import { useAccentColor } from '@/composables/useAccentColor'
  import { useTheme } from '@/composables/useTheme'
  import SearchResults from '@/views/SearchResultsView.vue'
  import FullscreenPlayer from './components/player/FullscreenPlayer.vue'
  import { usePlayerState } from './composables/usePlayerState'

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
  useTheme()
  useAccentColor()

  // Router setup
  const router = useRouter()
  const route = useRoute()

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

  const playerState = usePlayerState()

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
      const songs = await invoke<MusicItem[]>('get_music_library', {
        serverUrl: credentials.value.serverUrl,
        token:     credentials.value.token,
      })
      console.log(`DEBUG: Loaded ${songs.length} songs`)
      allSongs.value = songs

      // Since the other data is derived from songs, let's get them with the new commands
      console.log('DEBUG: Fetching albums and artists...')
      const [albums, artistsWithSongs, albumArtists] = await Promise.all([
        invoke<AlbumWithSongs[]>('get_albums_with_songs', {
          serverUrl: credentials.value.serverUrl,
          token:     credentials.value.token,
        }),
        invoke<ArtistWithSongs[]>('get_artists_with_songs', {
          serverUrl:        credentials.value.serverUrl,
          token:            credentials.value.token,
          albumArtistsOnly: false,
        }),
        invoke<ArtistWithSongs[]>('get_artists_with_songs', {
          serverUrl:        credentials.value.serverUrl,
          token:            credentials.value.token,
          albumArtistsOnly: true,
        }),
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
    invoke<number>('get_saved_volume')
      .then(savedVolume => {
        if (savedVolume !== null)
          volume.value = savedVolume
      })
      .catch(err => console.error('Failed to get saved volume:', err))

    invoke<Credentials>('get_saved_credentials')
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

  watch(volume, newVolume => {
    invoke('save_volume', { volume: newVolume })
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
  }

  const handlePlaySong = (song: MusicItem) => {
    currentSong.value = song
    if (!playlist.value.find((s: MusicItem) => s.id === song.id)) {
      playlist.value.push(song)
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

  const handleToggleFullScreenPlayer = () => {
    isFullScreenPlayerOpen.value = !isFullScreenPlayerOpen.value
  }

  const handleTogglePlayPause = () => musicPlayerRef.value?.togglePlayPause()
  const handlePreviousSong = () => musicPlayerRef.value?.previousSong()
  const handleNextSong = () => musicPlayerRef.value?.nextSong()
  const handleToggleShuffle = () => musicPlayerRef.value?.toggleShuffle()
  const handleToggleRepeat = () => musicPlayerRef.value?.toggleRepeat()
  const handleSeek = (value: number[]) => musicPlayerRef.value?.onSeek(value)

  // Handle song change from player
  const handleSongChanged = (song: MusicItem) => {
    currentSong.value = song
  }

  // Handle player state updates
  const handleUpdateCurrentSong = (song: MusicItem | null) => {
    currentSong.value = song
  }

  // Favorite a song
  const handleToggleFavorite = (song: MusicItem) => {
    if (!credentials.value)
      return

    invoke<boolean>('toggle_favorite_status', {
      serverUrl:  credentials.value.serverUrl,
      token:      credentials.value.token,
      userId:     credentials.value.userId,
      itemId:     song.id,
      isFavorite: !song.isFavorite,
    })
      .then(newStatus => {
        const songInLibrary = allSongs.value.find(s => s.id === song.id)
        if (songInLibrary)
          songInLibrary.isFavorite = newStatus
      })
      .catch(err => {
        console.error('Failed to toggle favorite status:', err)
      })
  }

  const handleClearCache = async () => {
    console.log('DEBUG: Starting cache clear from UI...')
    try {
      await invoke('clear_music_cache')
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

      <template #player>
        <MusicPlayer
          @song-changed='handleSongChanged'
          @toggle-fullscreen='handleToggleFullScreenPlayer'
          @toggle-queue='handleToggleQueue'
          @update-current-song='handleUpdateCurrentSong'
          @volume-changed='handleVolumeChange'
          v-if='currentSong'
          :key='currentSong.id'
          ref='musicPlayerRef'
          :current-song='currentSong'
          :playlist='playlist'
          :server-url='credentials!.serverUrl'
          :token='credentials!.token'
          :volume='volume'
        />
        <Queue
          @play-song='handlePlaySong'
          @remove-song='handleRemoveSong'
          @update:playlist='handleUpdatePlaylist'
          v-model='isQueueOpen'
          :current-song='currentSong'
          :playlist='playlist'
        />
      </template>
    </MainLayout>

    <FullscreenPlayer
      @close='handleToggleFullScreenPlayer'
      @next-song='handleNextSong'
      @previous-song='handlePreviousSong'
      @seek='handleSeek'
      @toggle-play-pause='handleTogglePlayPause'
      @toggle-repeat='handleToggleRepeat'
      @toggle-shuffle='handleToggleShuffle'
      :current-time='playerState.currentTime'
      :duration='playerState.duration'
      :has-next='playerState.hasNext'
      :has-previous='playerState.hasPrevious'
      :is-playing='playerState.isPlaying'
      :is-shuffled='playerState.isShuffled'
      :progress='playerState.progress'
      :repeat-mode='playerState.repeatMode'
      :show='isFullScreenPlayerOpen'
      :song='currentSong'
    />
  </div>
</template>
