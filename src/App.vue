<script setup lang="ts">
  import { ref, onMounted, watch, onUnmounted, computed } from 'vue'
  import { useColorMode } from '@vueuse/core'
  import { useRouter, useRoute } from 'vue-router'
  import Login from './components/Login.vue'
  import MusicPlayer from './components/MusicPlayer.vue'
  import Queue from './components/Queue.vue'
  import MainLayout from './components/MainLayout.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { MusicItem, AlbumInfo, ArtistInfo } from './types'
  import SearchResults from '@/components/SearchResults.vue'

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

  // App state
  const isLoggedIn = ref(false)
  const credentials = ref<Credentials | null>(null)
  const currentSong = ref<MusicItem | null>(null)
  const playlist = ref<MusicItem[]>([])
  const volume = ref(0.5)
  const isQueueOpen = ref(false)
  const allSongs = ref<MusicItem[]>([])
  const allAlbums = ref<AlbumInfo[]>([])
  const allArtists = ref<ArtistInfo[]>([])
  const libraryLoading = ref(false)
  const libraryError = ref('')
  const searchQuery = ref('')
  const isSearchVisible = ref(false)

  const allArtistSummaries = computed(() => {
    const artistMap = new Map<string, ArtistSummary>()
    const allArtistsMap = new Map<string, ArtistInfo>(allArtists.value.map(a => [a.Name, a]))

    allSongs.value.forEach(song => {
      if (!song.artists || song.artists.length === 0) {
        const unknown = 'Unknown Artist'
        if (!artistMap.has(unknown))
          artistMap.set(unknown, { id: '', name: unknown, songCount: 0, imageUrl: undefined })
        artistMap.get(unknown)!.songCount++
        return
      }
      song.artists.forEach(artistName => {
        const artistInfo = allArtistsMap.get(artistName)
        if (!artistMap.has(artistName)) {
          artistMap.set(artistName, {
            id:        artistInfo?.Id || '',
            name:      artistName,
            songCount: 0,
            imageUrl:  artistInfo?.imageUrl,
          })
        }
        artistMap.get(artistName)!.songCount++
      })
    })
    return Array.from(artistMap.values()).filter(a => a.songCount > 0).sort((a, b) => a.name.localeCompare(b.name))
  })

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

  const handleSelectAlbum = (album: AlbumInfo) => {
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
          token:     credentials.value.token,
        }),
        invoke<ArtistInfo[]>('get_all_artists', {
          serverUrl: credentials.value.serverUrl,
          token:     credentials.value.token,
        }),
      ])
      allSongs.value = songs
      allArtists.value = artists

      // Compute albums from songs
      const albumMap = new Map<string, AlbumInfo>()
      songs.forEach(song => {
        const albumName = song.album || 'Unknown Album'
        // Find the primary artist ID from the song
        const primaryArtistId = song.artistIds?.[0]
        // Find the primary artist name from the song
        const primaryArtistName = song.artists?.[0] || 'Unknown Artist'
        // Find the corresponding artist object to get the ID
        const artistInfo = primaryArtistId ? allArtists.value.find(a => a.Id === primaryArtistId) : null

        if (!albumMap.has(albumName)) {
          albumMap.set(albumName, {
            name:        albumName,
            artist:      primaryArtistName,
            // Add artistId to AlbumInfo
            artistId:    artistInfo?.Id,
            songCount:   0,
            albumArtUrl: song.albumArtUrl,
          })
        }
        const album = albumMap.get(albumName)
        if (album)
          album.songCount++
      })
      allAlbums.value = Array.from(albumMap.values()).sort((a, b) => a.name.localeCompare(b.name))

    } catch (err) {
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
          isLoggedIn.value = true
        }
      })
      .catch(err => console.error('Failed to get saved credentials:', err))
  })

  watch(isLoggedIn, (loggedIn: boolean) => {
    if (loggedIn)
      loadLibrary()
  })

  watch(volume, newVolume => {
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

</script>

<template>
  <div id='app' class='h-screen bg-background text-foreground'>
    <Login @login='handleLogin' v-if='!isLoggedIn' />
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
        <component
          :is='Component'
          @logout='handleLogout'
          @play-song='handlePlaySong'
          @play-songs='handlePlaySongs'
          @reload-library='loadLibrary'
          @select-album='handleSelectAlbum'
          @select-artist='handleSelectArtist'
          @toggle-favorite='handleToggleFavorite'
          :albums='allAlbums'
          :artists='allArtistSummaries'
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
          @toggle-queue='handleToggleQueue'
          @update-current-song='handleUpdateCurrentSong'
          @volume-changed='handleVolumeChange'
          v-if='currentSong'
          :key='currentSong.id'
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
  </div>
</template>
