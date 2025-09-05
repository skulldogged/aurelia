<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { MusicItem } from '@/types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Shuffle } from 'lucide-vue-next'
import Fuse from 'fuse.js'

const router = useRouter()

const props = defineProps<{
  songs: MusicItem[]
  artists: any[]
}>()

const emit = defineEmits<{
  'play-song': [song: MusicItem]
  'play-songs': [songs: MusicItem[]]
  'select-artist': [artist: any]
}>()

const searchQuery = ref('')

// Compute artists from songs with additional metadata
const artistsWithSongs = computed(() => {
  const artistMap = new Map<string, {
    id: string
    name: string
    songCount: number
    imageUrl?: string
    songs: MusicItem[]
  }>()

  props.songs.forEach((song) => {
    if (!song.artists || song.artists.length === 0) {
      const unknown = 'Unknown Artist'
      if (!artistMap.has(unknown)) {
        artistMap.set(unknown, {
          id: '',
          name: unknown,
          songCount: 0,
          imageUrl: undefined,
          songs: []
        })
      }
      artistMap.get(unknown)!.songs.push(song)
      artistMap.get(unknown)!.songCount++
      return
    }

    song.artists.forEach(artistName => {
      const artistInfo = props.artists.find(a => a.Name === artistName)
      if (!artistMap.has(artistName)) {
        artistMap.set(artistName, {
          id: artistInfo?.Id || '',
          name: artistName,
          songCount: 0,
          imageUrl: artistInfo?.imageUrl,
          songs: []
        })
      }
      artistMap.get(artistName)!.songs.push(song)
      artistMap.get(artistName)!.songCount++
    })
  })

  return Array.from(artistMap.values())
    .filter(artist => artist.songCount > 0)
    .sort((a, b) => a.name.localeCompare(b.name))
})

// Fuzzy search setup (Fuse.js)
const artistsFuse = ref(new Fuse(artistsWithSongs.value, {
  keys: ['name'],
  threshold: 0.4,
  ignoreLocation: true,
  distance: 1000,
  findAllMatches: true,
  minMatchCharLength: 1,
  shouldSort: true,
}))

watch(artistsWithSongs, (newArtists) => {
  artistsFuse.value.setCollection(newArtists)
})

const filteredArtists = computed(() => {
  if (!searchQuery.value) return artistsWithSongs.value
  return artistsFuse.value.search(searchQuery.value).map(result => result.item)
})

const playArtistShuffle = (artist: any) => {
  const artistSongs = artist.songs
  if (artistSongs.length > 0) {
    // Shuffle the songs
    const shuffledSongs = [...artistSongs].sort(() => 0.5 - Math.random())
    emit('play-songs', shuffledSongs)
  }
}

const selectArtist = (artist: any) => {
  router.push(`/songs/artist/${artist.id}`)
}
</script>

<template>
  <div class="p-8">
    <div class="mb-8">
      <h1 class="text-4xl font-bold mb-4">Artists</h1>
      <Input v-model="searchQuery" type="text" placeholder="Search artists..." class="max-w-sm" />
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-6">
      <div v-for="artist in filteredArtists" :key="artist.name" class="cursor-pointer group"
        @click="selectArtist(artist)">
        <div class="relative mb-4">
          <img v-if="artist.imageUrl" :src="artist.imageUrl" :alt="`${artist.name} artist image`"
            class="w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity" />
          <div v-else
            class="w-full aspect-square bg-muted rounded-lg flex items-center justify-center shadow-lg group-hover:opacity-75 transition-opacity">
            <span class="text-4xl">🎤</span>
          </div>

          <!-- Play button overlay -->
          <div
            class="absolute inset-0 bg-black/50 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
            <Button @click.stop="playArtistShuffle(artist)" size="icon"
              class="bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20">
              <Shuffle class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div class="text-center">
          <h3 class="font-semibold truncate">{{ artist.name }}</h3>
          <p class="text-sm text-muted-foreground">{{ artist.songCount }} songs</p>
        </div>
      </div>
    </div>

    <div v-if="filteredArtists.length === 0" class="text-center py-12">
      <p class="text-muted-foreground">No artists found</p>
    </div>
  </div>
</template>
