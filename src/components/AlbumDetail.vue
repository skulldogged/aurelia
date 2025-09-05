<script setup lang="ts">
import { computed } from 'vue'
import { Button } from '@/components/ui/button'
import SongList from './SongList.vue'
import { MusicItem } from '@/types'

interface Album {
  name: string
  artist: string
  songCount: number
  albumArtUrl?: string
}

const props = defineProps<{
  album: Album
  songs: MusicItem[]
  currentSong: MusicItem | null
  isPlaying: boolean
}>()

defineEmits<{
  'play-song': [song: MusicItem]
  'toggle-favorite': [song: MusicItem]
}>()

const albumSongs = computed(() => {
  return props.songs.filter(song => song.album === props.album.name)
})

const displayedArtist = computed(() => {
  // Find the most common artist for this album's songs
  if (!albumSongs.value.length) return props.album.artist // Fallback

  const artistCounts = new Map<string, number>()
  albumSongs.value.forEach(song => {
    song.artists?.forEach(artist => {
      artistCounts.set(artist, (artistCounts.get(artist) || 0) + 1)
    })
  })

  if (artistCounts.size === 0) return props.album.artist // Fallback

  // Get the artist with the highest count
  let maxCount = 0
  let primaryArtist = ''
  for (const [artist, count] of artistCounts.entries()) {
    if (count > maxCount) {
      maxCount = count
      primaryArtist = artist
    }
  }
  return primaryArtist
})
</script>

<template>
  <div class="space-y-8">
    <!-- Header -->
    <div class="flex items-center space-x-6">
      <img v-if="album.albumArtUrl" :src="album.albumArtUrl" alt="Album art" class="w-32 h-32 rounded-md" />
      <div v-else class="w-32 h-32 rounded-md bg-muted flex-shrink-0"></div>
      <div>
        <h1 class="text-5xl font-bold text-foreground">{{ album.name }}</h1>
        <p class="text-2xl text-muted-foreground mt-2">{{ displayedArtist }}</p>
      </div>
    </div>

    <!-- Songs -->
    <div>
      <h2 class="text-2xl font-semibold text-foreground mb-4">Songs</h2>
      <SongList :songs="albumSongs" :current-song="props.currentSong" :is-playing="props.isPlaying"
        :show-track-number="true" :show-duration="true" @play-song="(song) => $emit('play-song', song)"
        @toggle-favorite="(song) => $emit('toggle-favorite', song)" />
    </div>
  </div>
</template>
