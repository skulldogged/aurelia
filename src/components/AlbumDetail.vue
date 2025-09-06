<script setup lang="ts">
  import { computed } from 'vue'
  import { useRoute } from 'vue-router'
  import SongList from './SongList.vue'
  import { MusicItem, AlbumInfo } from '@/types'
  import { useImageCache } from '@/composables/useImageCache'

  const props = defineProps<{
    albums:      AlbumInfo[]
    songs:       MusicItem[]
    currentSong: MusicItem | null
    isPlaying:   boolean
  }>()

  defineEmits<{
    'play-song':       [song: MusicItem]
    'toggle-favorite': [song: MusicItem]
  }>()

  const route = useRoute()
  const albumName = computed(() => decodeURIComponent(route.params.albumName as string))
  const album = computed(() => props.albums.find(a => a.name === albumName.value))

  const imageUrls = computed(() => {
    const urls = new Set<string>()
    if (album.value && album.value.albumArtUrl) {
      urls.add(album.value.albumArtUrl)
    }
    return Array.from(urls)
  })

  const { cachedUrls } = useImageCache(() => imageUrls.value)

  const albumSongs = computed(() => {
    if (!album.value) return []
    return props.songs
      .filter(song => song.album === album.value!.name)
      .sort((a, b) => (a.trackNumber || 0) - (b.trackNumber || 0))
  })

  const displayedArtist = computed(() => {
    // Find the most common artist for this album's songs
    if (!albumSongs.value.length) return album.value?.artist // Fallback

    const artistCounts = new Map<string, number>()
    albumSongs.value.forEach(song => {
      song.artists?.forEach(artist => {
        artistCounts.set(artist, (artistCounts.get(artist) || 0) + 1)
      })
    })

    if (artistCounts.size === 0) return album.value?.artist // Fallback

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
  <div v-if='album' class='space-y-8 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8'>
    <!-- Header -->
    <div class='flex items-center space-x-6'>
      <img
        v-if='album.albumArtUrl'
        :src='cachedUrls[album.albumArtUrl] || album.albumArtUrl'
        alt='Album art'
        class='w-32 h-32 rounded-md'
      >
      <div v-else class='w-32 h-32 rounded-md bg-muted flex-shrink-0' />
      <div>
        <h1 class='text-5xl font-bold text-foreground'>
          {{ album.name }}
        </h1>
        <p class='text-2xl text-muted-foreground mt-2'>
          {{ displayedArtist }}
        </p>
      </div>
    </div>

    <!-- Songs -->
    <div>
      <h2 class='text-2xl font-semibold text-foreground mb-4'>
        Songs
      </h2>
      <SongList
        @play-song="(song) => $emit('play-song', song)"
        @toggle-favorite="(song) => $emit('toggle-favorite', song)"
        :current-song='props.currentSong'
        :is-playing='props.isPlaying'
        :show-duration='true'
        :show-track-number='true'
        :songs='albumSongs'
      />
    </div>
  </div>
  <div v-else class='text-center py-12 text-muted-foreground'>
    Album not found.
  </div>
</template>
