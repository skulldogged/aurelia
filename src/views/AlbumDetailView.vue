<script setup lang="ts">
  import { Music, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, ref } from 'vue'
  import { useRoute } from 'vue-router'

  import { Album, NameIdPair, Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import SongList from '@/components/shared/SongList.vue'
  import { Button } from '@/components/ui/button'
  import { Skeleton } from '@/components/ui/skeleton'

  const props = defineProps<{
    allAlbums:      Album[]
    currentSong:    null | Song
    isPlaying:      boolean
    libraryLoaded:  boolean
    libraryLoading: boolean
    serverUrl:      string
    token:          string
  }>()

  const emit = defineEmits<{
    'play-song':       [song: Song]
    'play-songs':      [songs: Song[]]
    'toggle-favorite': [song: Song]
  }>()

  const route = useRoute()
  const showSkeleton = ref(false) // Dev toggle for skeleton adjustment
  const showShareDialog = ref(false)

  const album = computed(() => {
    if (!props.libraryLoaded || !props.allAlbums.length) return null
    return props.allAlbums.find(a => a.name === decodeURIComponent(route.params.albumName as string)) || null
  })

  const albumSongs = computed(() => {
    if (!album.value) return []
    return [...album.value.songs || []].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
  })

  // Unique album artists aggregated from tracks
  const albumArtistPairs = computed<NameIdPair[]>(() => {
    const idToName = new Map<string, string>()
    for (const song of albumSongs.value)
      if (song.albumArtists)
        for (const pair of song.albumArtists)
          if (pair.id && pair.name) idToName.set(pair.id, pair.name)

    // Fallbacks if albumArtists are not provided by backend
    if (idToName.size === 0) {
      const first = albumSongs.value[0]
      if (first?.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, idx) => {
          const name = first.artists![idx]
          if (id && name) idToName.set(id, name)
        })
      } else if (album.value?.artist && album.value.artistId) {
        idToName.set(album.value.artistId, album.value.artist)
      }
    }

    return Array.from(idToName, ([id, name]) => ({ id, name }))
  })

  // Determine if any song in the album has multiple artists
  const hasMultipleArtists =
    computed(() =>
      albumSongs.value.length > 1 &&
      albumSongs.value.some(song =>
        song.artists?.length &&
        song.artists.length > 1,
      ),
    )

  // Aggregated album metadata
  const albumYear = computed(() => albumSongs.value.find(s => s.year)?.year || null)
  const totalDurationSec = computed(() => albumSongs.value.reduce((acc, s) => acc + (s.duration || 0), 0))
  const formattedTotalDuration = computed(() => {
    const total = totalDurationSec.value
    const hours = Math.floor(total / 3600)
    const minutes = Math.floor((total % 3600) / 60)
    if (hours > 0) return `${hours} hr ${minutes} min`
    return `${minutes} min`
  })
  const albumGenres = computed(() => {
    const set = new Set<string>()
    for (const song of albumSongs.value) {
      if (song.genres) for (const g of song.genres) set.add(g)
    }
    return Array.from(set)
  })

  const playAll = (): void => {
    if (albumSongs.value.length > 0) emit('play-songs', albumSongs.value)
  }

  const shuffleAll = (): void => {
    const songs = [...albumSongs.value]
    for (let i = songs.length - 1; i > 0; i -= 1) {
      const j = Math.floor(Math.random() * (i + 1))
      const temp = songs[i]
      songs[i] = songs[j]
      songs[j] = temp
    }
    if (songs.length > 0) emit('play-songs', songs)
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto space-y-8'>
    <div class='flex justify-end'>
      <Button
        @click='showSkeleton = !showSkeleton'
        :disabled='libraryLoading || !libraryLoaded || !album'
        size='sm'
        variant='outline'
      >
        {{ showSkeleton ? 'Hide' : 'Show' }} Skeleton (dev)
      </Button>
    </div>
    <div v-if='libraryLoading || !libraryLoaded || !album || showSkeleton' class='space-y-8'>
      <!-- Header Skeleton -->
      <div class='flex items-center space-x-6 p-8 blur-card rounded-2xl'>
        <Skeleton class='w-48 h-48 rounded-lg' />
        <div class='flex-1'>
          <Skeleton class='h-12 w-3/4 mb-3' />
          <Skeleton class='h-6 w-1/2 mb-4' />
          <div class='flex items-center gap-2 mb-4'>
            <Skeleton class='h-8 w-24 rounded-md' />
            <Skeleton class='h-8 w-28 rounded-md' />
          </div>
          <div class='flex flex-wrap gap-2'>
            <Skeleton class='h-5 w-20 rounded-full' />
            <Skeleton class='h-5 w-16 rounded-full' />
            <Skeleton class='h-5 w-24 rounded-full' />
          </div>
        </div>
      </div>

      <!-- Songs Skeleton -->
      <div>
        <Skeleton class='h-8 w-24 mb-4' />
        <div class='space-y-2'>
          <div v-for='i in 10' :key='`song-skeleton-${i}`' class='flex items-center space-x-4 p-2 rounded-md'>
            <Skeleton class='w-10 h-10 rounded-md' />
            <div class='flex-1 space-y-2'>
              <Skeleton class='h-4 w-3/4' />
              <Skeleton class='h-3 w-1/2' />
            </div>
            <Skeleton class='h-4 w-12' />
          </div>
        </div>
      </div>
    </div>
    <div v-else-if='album' class='space-y-8'>
      <!-- Header -->
      <div class='flex items-center space-x-6 p-8 blur-card rounded-2xl'>
        <div class='flex-shrink-0'>
          <ImageLoader
            :item-id='album.id || album.name'
            :server-url='serverUrl'
            :token='token'
            alt='Album art'
            class='w-48 h-48 rounded-lg object-cover'
          >
            <template #fallback>
              <div class='w-48 h-48 rounded-lg bg-muted flex items-center justify-center'>
                <Music class='w-24 h-24 text-muted-foreground' />
              </div>
            </template>
          </ImageLoader>
        </div>
        <div>
          <h1 class='text-5xl font-bold text-foreground select-text'>
            {{ album.name }}
          </h1>
          <p class='text-2xl text-muted-foreground mt-2 select-text'>
            <template v-if='albumArtistPairs.length'>
              <template v-for='(pair, index) in albumArtistPairs' :key='pair.id'>
                <router-link
                  :to="{ name: 'artist-detail', params: { artistId: pair.id } }"
                  class='hover:underline'
                >
                  {{ pair.name }}
                </router-link>
                <span v-if='index < albumArtistPairs.length - 1'>, </span>
              </template>
            </template>
            <template v-else>
              {{ album?.artist || 'Unknown Artist' }}
            </template>
          </p>
          <!-- Meta chips -->
          <div class='flex flex-wrap items-center gap-y-2 mt-3 text-sm text-muted-foreground'>
            <span v-if='albumYear' class='inline-flex items-center gap-1'>
              {{ albumYear }}
            </span>
            <span v-if='albumYear && albumSongs.length' class='mx-2 self-center'>•</span>
            <span v-if='albumSongs.length' class='inline-flex items-center'>
              {{ albumSongs.length }} track{{ albumSongs.length > 1 ? 's' : '' }}
            </span>
            <span v-if='albumSongs.length' class='mx-2 self-center'>•</span>
            <span v-if='albumSongs.length' class='inline-flex items-center gap-1'>
              {{ formattedTotalDuration }}
            </span>
            <span v-if='albumGenres.length && (albumYear || albumSongs.length)' class='mx-2 self-center'>•</span>
            <span v-if='albumGenres.length' class='inline-flex items-center'>
              {{ albumGenres.join(', ') }}
            </span>
          </div>
          <!-- Actions -->
          <div class='flex items-center gap-2 mt-4'>
            <Button @click='playAll'>
              <Play class='w-4 h-4 mr-2' />
              Play
            </Button>
            <Button @click='shuffleAll' variant='outline'>
              <Shuffle class='w-4 h-4 mr-2' />
              Shuffle
            </Button>
            <Button @click='showShareDialog = true' variant='outline'>
              <Share2 class='w-4 h-4 mr-2' />
              Share
            </Button>
          </div>
        </div>
      </div>

      <!-- Songs -->
      <div class='w-full'>
        <h2 class='text-2xl font-semibold text-foreground mb-4'>
          Songs
        </h2>
        <SongList
          @play-song='(song) => $emit("play-song", song)'
          @toggle-favorite='(song) => $emit("toggle-favorite", song)'
          :current-song='props.currentSong'
          :is-playing='props.isPlaying'
          :loading='libraryLoading || !libraryLoaded || !album || showSkeleton'
          :server-url='props.serverUrl'
          :show-album-art='false'
          :show-artist='hasMultipleArtists'
          :show-duration='true'
          :show-track-number='true'
          :songs='albumSongs'
          :token='props.token'
          layout='comfy'
        />
      </div>
    </div>
    <div v-else class='text-center py-12 text-muted-foreground'>
      Album not found.
    </div>

    <ShareDialog
      v-if='album'
      v-model:open='showShareDialog'
      :item-id='album.id || ""'
      :item-name='album.name'
      :item-type="'album'"
    />
  </div>
</template>
