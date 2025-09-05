<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { MusicItem } from '@/types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Play } from 'lucide-vue-next'
import Fuse from 'fuse.js'

const router = useRouter()

const props = defineProps<{
  songs: MusicItem[]
  albums: any[]
}>()

const emit = defineEmits<{
  'play-songs': [songs: MusicItem[]]
  'select-album': [album: any]
}>()

const searchQuery = ref('')

// Compute albums with songs
const albumsWithSongs = computed(() => {
  const albumMap = new Map<string, {
    name: string
    artist: string
    songCount: number
    albumArtUrl?: string
    songs: MusicItem[]
  }>()

  props.songs.forEach((song) => {
    const albumName = song.album || 'Unknown Album'
    const artist = song.artists?.join(', ') || 'Unknown Artist'

    if (!albumMap.has(albumName)) {
      albumMap.set(albumName, {
        name: albumName,
        artist,
        songCount: 0,
        albumArtUrl: song.albumArtUrl,
        songs: []
      })
    }

    albumMap.get(albumName)!.songs.push(song)
    albumMap.get(albumName)!.songCount++
  })

  return Array.from(albumMap.values())
    .sort((a, b) => a.name.localeCompare(b.name))
})

// Fuzzy search setup (Fuse.js)
const albumsFuse = ref(new Fuse(albumsWithSongs.value, {
  keys: ['name', 'artist'],
  threshold: 0.4,
  ignoreLocation: true,
  distance: 1000,
  findAllMatches: true,
  minMatchCharLength: 1,
  shouldSort: true,
}))

watch(albumsWithSongs, (newAlbums) => {
  albumsFuse.value.setCollection(newAlbums)
})

const filteredAlbums = computed(() => {
  if (!searchQuery.value) return albumsWithSongs.value
  return albumsFuse.value.search(searchQuery.value).map(result => result.item)
})

const playAlbum = (album: any) => {
  const albumSongs = album.songs
    .sort((a: MusicItem, b: MusicItem) => (a.trackNumber || 0) - (b.trackNumber || 0))

  if (albumSongs.length > 0) {
    emit('play-songs', albumSongs)
  }
}

const selectAlbum = (album: any) => {
  router.push(`/songs/album/${encodeURIComponent(album.name)}`)
}
</script>

<template>
  <div class="p-8">
    <div class="mb-8">
      <h1 class="text-4xl font-bold mb-4">Albums</h1>
      <Input v-model="searchQuery" type="text" placeholder="Search albums..." class="max-w-sm" />
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-6">
      <div v-for="album in filteredAlbums" :key="album.name" class="cursor-pointer group" @click="selectAlbum(album)">
        <div class="relative mb-4">
          <img v-if="album.albumArtUrl" :src="album.albumArtUrl" :alt="`${album.name} album art`"
            class="w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity" />
          <div v-else
            class="w-full aspect-square bg-muted rounded-lg flex items-center justify-center shadow-lg group-hover:opacity-75 transition-opacity">
            <span class="text-4xl">💿</span>
          </div>

          <!-- Play button overlay -->
          <div
            class="absolute inset-0 bg-black/50 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
            <Button @click.stop="playAlbum(album)" size="icon"
              class="bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20">
              <Play class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <h3 class="font-semibold truncate">{{ album.name }}</h3>
          <p class="text-sm text-muted-foreground truncate">{{ album.artist }}</p>
          <p class="text-xs text-muted-foreground">{{ album.songCount }} songs</p>
        </div>
      </div>
    </div>

    <div v-if="filteredAlbums.length === 0" class="text-center py-12">
      <p class="text-muted-foreground">No albums found</p>
    </div>
  </div>
</template>
