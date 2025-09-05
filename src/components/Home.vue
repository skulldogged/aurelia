<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { MusicItem } from '@/types'
import Carousel from './Carousel.vue'

const router = useRouter()

const props = defineProps<{
  songs: MusicItem[],
  albums: any[]
}>()

const emit = defineEmits<{
  'play-songs': [songs: MusicItem[]],
  'select-album': [album: any]
}>()

const mostPlayed = computed(() => {
  return [...props.songs]
    .sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
    .slice(0, 10)
})

const recentlyPlayed = computed(() => {
  return [...props.songs]
    .filter(s => s.datePlayed)
    .sort((a, b) => new Date(b.datePlayed!).getTime() - new Date(a.datePlayed!).getTime())
    .slice(0, 10)
})

const newReleases = computed(() => {
  // Get albums that were recently added to the library
  // We'll use the most recent song's datePlayed or a fallback to track when albums were added
  const oneMonthAgo = new Date()
  oneMonthAgo.setMonth(oneMonthAgo.getMonth() - 1)

  return props.albums
    .filter(album => {
      // Find the most recent song from this album
      const albumSongs = props.songs.filter(s => s.album === album.name)
      if (albumSongs.length === 0) return false

      // Use datePlayed if available, otherwise use a fallback based on song order
      const mostRecentSong = albumSongs.reduce((latest, current) => {
        if (current.datePlayed && latest.datePlayed) {
          return new Date(current.datePlayed) > new Date(latest.datePlayed) ? current : latest
        }
        return latest
      })

      // If we have datePlayed data, use it; otherwise consider it recent if it's in the library
      if (mostRecentSong.datePlayed) {
        return new Date(mostRecentSong.datePlayed) > oneMonthAgo
      }

      // Fallback: consider albums with songs as "recently added" if we don't have datePlayed data
      return true
    })
    .sort((a, b) => {
      // Sort by the most recent activity (datePlayed or fallback to album name for consistency)
      const albumASongs = props.songs.filter(s => s.album === a.name)
      const albumBSongs = props.songs.filter(s => s.album === b.name)

      const getMostRecentDate = (songs: typeof props.songs) => {
        const withDates = songs.filter(s => s.datePlayed)
        if (withDates.length > 0) {
          return Math.max(...withDates.map(s => new Date(s.datePlayed!).getTime()))
        }
        return 0 // Fallback for albums without datePlayed data
      }

      const dateA = getMostRecentDate(albumASongs)
      const dateB = getMostRecentDate(albumBSongs)

      if (dateA === dateB) {
        // If dates are equal or both 0, sort alphabetically
        return a.name.localeCompare(b.name)
      }

      return dateB - dateA
    })
    .slice(0, 10)
})

const randomAlbums = computed(() => {
  return [...props.albums].sort(() => 0.5 - Math.random()).slice(0, 10)
})

const featuredAlbums = ref<any[]>([])
const currentFeaturedIndex = ref(0)

const featuredAlbum = computed(() => {
  return featuredAlbums.value[currentFeaturedIndex.value] || null
})

// Initialize featured albums with a randomized list
const initializeFeaturedAlbums = () => {
  featuredAlbums.value = [...props.albums].sort(() => 0.5 - Math.random())
}

const nextFeaturedAlbum = () => {
  if (featuredAlbums.value.length > 1) {
    currentFeaturedIndex.value = (currentFeaturedIndex.value + 1) % featuredAlbums.value.length
  }
}

const prevFeaturedAlbum = () => {
  if (featuredAlbums.value.length > 1) {
    currentFeaturedIndex.value = currentFeaturedIndex.value === 0
      ? featuredAlbums.value.length - 1
      : currentFeaturedIndex.value - 1
  }
}

// Watch for changes in albums and reinitialize
watch(() => props.albums, () => {
  initializeFeaturedAlbums()
}, { immediate: true })

function playSongs(songs: MusicItem[], startWith?: MusicItem) {
  if (startWith) {
    const startIndex = songs.indexOf(startWith)
    if (startIndex === -1) {
      emit('play-songs', songs)
      return
    }
    const reorderedSongs = [...songs.slice(startIndex), ...songs.slice(0, startIndex)]
    emit('play-songs', reorderedSongs)
  } else {
    emit('play-songs', songs)
  }
}

function playFeaturedAlbum() {
  if (!featuredAlbum.value) return

  // Get all songs from the featured album
  const albumSongs = props.songs
    .filter(song => song.album === featuredAlbum.value.name)
    .sort((a, b) => (a.trackNumber || 0) - (b.trackNumber || 0))

  if (albumSongs.length > 0) {
    // Play the songs and navigate to the album
    emit('play-songs', albumSongs)
    router.push(`/songs/album/${encodeURIComponent(featuredAlbum.value.name)}`)
  }
}
</script>

<template>
  <div class="p-8 space-y-12">
    <!-- Featured Album Section -->
    <div v-if="featuredAlbum" class="relative rounded-2xl p-8 mb-8 overflow-hidden blur-card">
      <!-- Blurred Background -->
      <div v-if="featuredAlbum.albumArtUrl" class="absolute inset-0 bg-cover bg-center bg-no-repeat scale-110"
        :style="{ backgroundImage: `url(${featuredAlbum.albumArtUrl})` }">
        <div class="absolute inset-0 bg-black/60 backdrop-blur-md"></div>
      </div>
      <div v-else class="absolute inset-0 bg-gradient-to-r from-muted/50 to-muted/20"></div>

      <!-- Content -->
      <div class="relative z-10 flex items-center space-x-6">
        <div class="flex-shrink-0">
          <img v-if="featuredAlbum.albumArtUrl" :src="featuredAlbum.albumArtUrl"
            :alt="`${featuredAlbum.name} album art`" class="w-48 h-48 rounded-xl shadow-2xl object-cover" />
          <div v-else
            class="w-48 h-48 bg-muted/80 backdrop-blur-sm rounded-xl shadow-2xl flex items-center justify-center">
            <span class="text-4xl">🎵</span>
          </div>
        </div>
        <div class="flex-1 min-w-0">
          <h1 class="text-4xl font-bold mb-2 text-white drop-shadow-lg">{{ featuredAlbum.name }}</h1>
          <p class="text-xl text-white/90 mb-4 drop-shadow-md">{{ featuredAlbum.artist }}</p>
          <p class="text-sm text-white/80 mb-6 drop-shadow-md">{{ featuredAlbum.songCount }} songs</p>
          <button @click="playFeaturedAlbum"
            class="bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white px-8 py-3 rounded-full font-semibold transition-colors border border-white/20">
            Play Album
          </button>
        </div>
      </div>

      <!-- Navigation Arrows -->
      <div v-if="featuredAlbums.length > 1" class="absolute bottom-4 right-4 flex space-x-2">
        <button @click="prevFeaturedAlbum"
          class="bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white p-2 rounded-full transition-colors border border-white/20">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
          </svg>
        </button>
        <button @click="nextFeaturedAlbum"
          class="bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white p-2 rounded-full transition-colors border border-white/20">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
          </svg>
        </button>
      </div>
    </div>

    <Carousel title="Most Played">
      <div v-for="song in mostPlayed" :key="song.id" @click="playSongs(mostPlayed, song)" class="cursor-pointer group">
        <img v-if="song.albumArtUrl" :src="song.albumArtUrl" alt="Album art"
          class="w-full h-auto rounded-lg mb-2 shadow-lg group-hover:opacity-75 aspect-square object-cover" />
        <div v-else class="w-full h-48 bg-muted rounded-lg mb-2"></div>
        <p class="font-semibold truncate">{{ song.name }}</p>
        <p class="text-sm text-muted-foreground truncate">{{ song.artists?.join(', ') }}</p>
      </div>
    </Carousel>

    <Carousel title="Recently Played">
      <div v-for="song in recentlyPlayed" :key="song.id" @click="playSongs(recentlyPlayed, song)"
        class="cursor-pointer group">
        <img v-if="song.albumArtUrl" :src="song.albumArtUrl" alt="Album art"
          class="w-full h-auto rounded-lg mb-2 shadow-lg group-hover:opacity-75 aspect-square object-cover" />
        <div v-else class="w-full h-48 bg-muted rounded-lg mb-2"></div>
        <p class="font-semibold truncate">{{ song.name }}</p>
        <p class="text-sm text-muted-foreground truncate">{{ song.artists?.join(', ') }}</p>
      </div>
    </Carousel>

    <Carousel title="New Releases">
      <div v-for="album in newReleases" :key="album.name" @click="$emit('select-album', album)"
        class="cursor-pointer group">
        <img v-if="album.albumArtUrl" :src="album.albumArtUrl" alt="Album art"
          class="w-full h-auto rounded-lg mb-2 shadow-lg group-hover:opacity-75 aspect-square object-cover" />
        <div v-else class="w-full h-48 bg-muted rounded-lg mb-2"></div>
        <p class="font-semibold truncate">{{ album.name }}</p>
        <p class="text-sm text-muted-foreground truncate">{{ album.artist }}</p>
      </div>
    </Carousel>

    <Carousel title="From Your Library">
      <div v-for="album in randomAlbums" :key="album.name" @click="$emit('select-album', album)"
        class="cursor-pointer group">
        <img v-if="album.albumArtUrl" :src="album.albumArtUrl" alt="Album art"
          class="w-full h-auto rounded-lg mb-2 shadow-lg group-hover:opacity-75 aspect-square object-cover" />
        <div v-else class="w-full h-48 bg-muted rounded-lg mb-2"></div>
        <p class="font-semibold truncate">{{ album.name }}</p>
        <p class="text-sm text-muted-foreground truncate">{{ album.artist }}</p>
      </div>
    </Carousel>
  </div>
</template>
