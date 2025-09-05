<template>
  <div class="h-full flex flex-col">
    <main class="flex-grow overflow-y-auto custom-scrollbar">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <ArtistDetail v-if="currentView === 'artist' && selectedArtist" :artist="selectedArtist" :songs="props.songs"
          :albums="albums" :all-artists="artists" :current-song="currentSong" :is-playing="isPlaying"
          :server-url="props.serverUrl" :token="props.token" :user-id="props.userId" @play-song="playSong"
          @select-album="selectAlbum" @play-album="handlePlayAlbum" @play-artist-shuffle="handlePlayArtistShuffle"
          @select-artist="selectArtist" />
        <AlbumDetail v-else-if="currentView === 'album' && selectedAlbum" :album="selectedAlbum" :songs="props.songs"
          :current-song="currentSong" :is-playing="isPlaying" @play-song="playSong"
          @toggle-favorite="handleToggleFavorite" />
        <div v-else>
          <div class="mb-8">
            <h1 class="text-4xl font-bold mb-4">Songs</h1>
            <Input v-model="searchQuery" type="text" placeholder="Search songs..." class="max-w-sm" />
          </div>

          <div v-if="props.loading" class="flex justify-center items-center py-12">
            <div class="text-muted-foreground">Loading songs...</div>
          </div>
          <div v-else-if="props.error" class="text-center py-12">
            <p class="text-destructive mb-4">{{ props.error }}</p>
            <Button @click="$emit('reload-library')" variant="destructive">Try Again</Button>
          </div>
          <div v-else>
            <SongList :songs="filteredSongs" :current-song="currentSong" :is-playing="isPlaying" :show-artist="true"
              :show-album="true" :show-year="true" :show-duration="true" :show-track-number="true" @play-song="playSong"
              @toggle-favorite="handleToggleFavorite" />
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Fuse from 'fuse.js'
import ArtistDetail from './ArtistDetail.vue'
import AlbumDetail from './AlbumDetail.vue'
import SongList from './SongList.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { MusicItem, ArtistInfo } from '@/types'

const route = useRoute()
const router = useRouter()


// Define props from parent
const props = defineProps<{
  serverUrl: string
  token: string
  userId: string
  songs: MusicItem[]
  artists: ArtistInfo[]
  loading: boolean
  error: string
}>()

// Define emits for parent
const emit = defineEmits<{
  logout: []
  playSong: [song: MusicItem]
  'play-album-songs': [songs: MusicItem[]]
  'toggle-favorite': [song: MusicItem]
  'reload-library': []
}>()

// Component state
const searchQuery = ref('')
const currentSong = ref<MusicItem | null>(null)
const isPlaying = ref(false)
const selectedArtist = ref<{ id: string, name: string; imageUrl?: string } | null>(null)
const selectedAlbum = ref<{ name: string; artist: string; songCount: number; albumArtUrl?: string } | null>(null)
// Current view based on route
const currentView = computed(() => {
  if (route.name === 'artist-detail') return 'artist'
  if (route.name === 'album-detail') return 'album'
  return 'main'
})

// Computed properties for artists and albums from props.songs
const artists = computed(() => {
  const artistMap = new Map<string, { id: string; name: string; songCount: number; imageUrl?: string }>()
  const allArtistsMap = new Map<string, ArtistInfo>(props.artists.map(a => [a.Name, a]))

  props.songs.forEach((song) => {
    if (!song.artists || song.artists.length === 0) {
      const unknown = 'Unknown Artist'
      if (!artistMap.has(unknown)) {
        artistMap.set(unknown, { id: '', name: unknown, songCount: 0, imageUrl: undefined })
      }
      artistMap.get(unknown)!.songCount++
      return
    }
    song.artists.forEach(artistName => {
      const artistInfo = allArtistsMap.get(artistName)
      if (!artistMap.has(artistName)) {
        artistMap.set(artistName, {
          id: artistInfo?.Id || '',
          name: artistName,
          songCount: 0,
          imageUrl: artistInfo?.imageUrl
        })
      }
      artistMap.get(artistName)!.songCount++
    })
  })
  return Array.from(artistMap.values()).filter(a => a.songCount > 0).sort((a, b) => a.name.localeCompare(b.name))
})

const albums = computed(() => {
  const albumMap = new Map<string, { name: string; artist: string; songCount: number; albumArtUrl?: string }>()
  props.songs.forEach((song) => {
    const albumName = song.album || 'Unknown Album'
    const artist = song.artists?.join(', ') || 'Unknown Artist'
    if (!albumMap.has(albumName)) {
      albumMap.set(albumName, { name: albumName, artist, songCount: 0, albumArtUrl: song.albumArtUrl })
    }
    albumMap.get(albumName)!.songCount++
  })
  return Array.from(albumMap.values()).sort((a, b) => a.name.localeCompare(b.name))
})

// Fuzzy Search setup
const songFuse = ref(new Fuse(props.songs, { keys: ['name', 'artists', 'album'], threshold: 0.4 }))
const artistFuse = ref(new Fuse(artists.value, { keys: ['name'], threshold: 0.4 }))
const albumFuse = ref(new Fuse(albums.value, { keys: ['name', 'artist'], threshold: 0.4 }))

watch(() => props.songs, (newSongs) => songFuse.value.setCollection(newSongs))
watch(artists, (newArtists) => artistFuse.value.setCollection(newArtists))
watch(albums, (newAlbums) => albumFuse.value.setCollection(newAlbums))

// Computed properties for filtering
const filteredSongs = computed(() => {
  if (!searchQuery.value) return props.songs
  return songFuse.value.search(searchQuery.value).map(result => result.item)
})

// Load artist/album data based on route parameters
const loadDataFromRoute = () => {
  if (route.name === 'artist-detail' && route.params.artistId) {
    const artistId = route.params.artistId as string
    const artist = artists.value.find(a => a.id === artistId)
    if (artist) {
      selectedArtist.value = artist
    }
  } else if (route.name === 'album-detail' && route.params.albumName) {
    const albumName = decodeURIComponent(route.params.albumName as string)
    const album = albums.value.find(a => a.name === albumName)
    if (album) {
      selectedAlbum.value = album
    }
  } else {
    // Reset when not on detail pages
    selectedArtist.value = null
    selectedAlbum.value = null
  }
}

// Watch for route changes
watch(() => route.params, loadDataFromRoute, { immediate: true })

// Methods
const playSong = (song: MusicItem) => {
  emit('playSong', song)
}

const handlePlayAlbum = (album: { name: string }) => {
  const albumSongs = props.songs
    .filter(s => s.album === album.name)
    .sort((a, b) => (a.trackNumber || 0) - (b.trackNumber || 0))
  if (albumSongs.length > 0) {
    emit('play-album-songs', albumSongs)
  }
}

const handlePlayArtistShuffle = (artist: { name: string }) => {
  const artistSongs = props.songs.filter(s => s.artists?.includes(artist.name))
  if (artistSongs.length > 0) {
    for (let i = artistSongs.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [artistSongs[i], artistSongs[j]] = [artistSongs[j], artistSongs[i]]
    }
    emit('play-album-songs', artistSongs)
  }
}

const selectArtist = (artist: { id: string; name: string; imageUrl?: string }) => {
  // Navigate to artist detail route
  router.push(`/songs/artist/${artist.id}`)
}

const selectAlbum = (album: { name: string; artist: string; songCount: number; albumArtUrl?: string }) => {
  // Navigate to album detail route
  router.push(`/songs/album/${encodeURIComponent(album.name)}`)
}

const handleToggleFavorite = (song: MusicItem) => {
  emit('toggle-favorite', song)
}


// Method for parent to update current song
const updateCurrentSong = (song: MusicItem | null, playing: boolean) => {
  currentSong.value = song
  isPlaying.value = playing
}

// Expose methods to parent
defineExpose({ updateCurrentSong, selectAlbum, selectArtist })
</script>
