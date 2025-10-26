<script setup lang="ts">
  import { MoreHorizontal, Music, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, ref } from 'vue'
  import { useRoute } from 'vue-router'

  import { Album, NameIdPair, Song } from '@/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import SongList from '@/components/shared/SongList.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const emit = defineEmits<{
    'play-instant-mix': [song: Song]
    'play-songs':       [songs: Song[]]
    'toggle-favorite':  [song: Song]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  // Create computed properties from stores
  const allAlbums = computed(() => libraryStore.allAlbums as Album[])
  const libraryLoaded = computed(() => libraryStore.isLoaded)
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const route = useRoute()
  const showShareDialog = ref(false)

  const id = computed(() => {
    const params = route.params
    if ('id' in params) {
      const param = params.id
      if (typeof param === 'string') return param
      if (Array.isArray(param)) return param[0] ?? ''
    }
    return ''
  })

  const album = computed(() =>
    id.value
    && libraryLoaded.value
    && allAlbums.value.length > 0
      ? allAlbums.value.find(a => a.id === id.value) || null
      : null,
  )

  const albumSongs = computed(() =>
    album.value
      ? [...album.value.songs || []].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
      : [],
  )

  // Unique album artists aggregated from songs
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
  const hasMultipleArtists = computed(() =>
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

    for (const song of albumSongs.value)
      if (song.genres)
        for (const g of song.genres)
          set.add(g)

    return Array.from(set)
  })

  const playAll = (): void => {
    if (albumSongs.value.length > 0)
      emit('play-songs', albumSongs.value)
  }

  const playSongWithQueue = (song: Song): void => {
    const songIndex = albumSongs.value.findIndex(s => s.id === song.id)
    if (songIndex === -1) return

    // Queue current song and all songs after it
    const songsToQueue = albumSongs.value.slice(songIndex)
    emit('play-songs', songsToQueue)
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
  <div class='px-4 pb-4' style='padding-top: env(safe-area-inset-top)'>
    <div v-if='libraryLoading || !libraryLoaded || !album' class='space-y-6'>
      <!-- Header Skeleton -->
      <div class='flex flex-col items-center space-y-4 p-6 bg-sidebar rounded-lg'>
        <Skeleton class='w-32 h-32 rounded-lg' />
        <div class='text-center space-y-2'>
          <Skeleton class='h-8 w-48 mx-auto' />
          <Skeleton class='h-5 w-32 mx-auto' />
          <div class='flex justify-center gap-2'>
            <Skeleton class='h-6 w-16 rounded-md' />
            <Skeleton class='h-6 w-20 rounded-md' />
          </div>
          <div class='flex justify-center gap-1'>
            <Skeleton class='h-4 w-16 rounded-full' />
            <Skeleton class='h-4 w-12 rounded-full' />
            <Skeleton class='h-4 w-20 rounded-full' />
          </div>
          <div class='flex justify-center gap-2 pt-2'>
            <Skeleton class='h-10 w-24 rounded-md' />
            <Skeleton class='h-10 w-10 rounded-md' />
          </div>
        </div>
      </div>

      <!-- Songs Skeleton -->
      <div>
        <Skeleton class='h-6 w-20 mb-3' />
        <div class='space-y-2'>
          <div v-for='i in 8' :key='`song-skeleton-${i}`' class='flex items-center space-x-3 p-2 rounded-md'>
            <Skeleton class='size-8 rounded-md' />
            <div class='flex-1 space-y-1'>
              <Skeleton class='h-4 w-3/4' />
              <Skeleton class='h-3 w-1/2' />
            </div>
            <Skeleton class='h-4 w-10' />
          </div>
        </div>
      </div>
    </div>

    <div v-else-if='album' class='space-y-6'>
      <!-- Header -->
      <div class='flex flex-col items-center space-y-4 p-6 bg-sidebar rounded-lg'>
        <div class='shrink-0'>
          <ImageLoader
            :item-id='album.id || album.name'
            :server-url='serverUrl'
            :token='token'
            alt='Album art'
            class='w-32 h-32 rounded-lg object-cover'
          >
            <template #fallback>
              <div class='w-32 h-32 rounded-lg bg-muted flex items-center justify-center'>
                <Music class='size-16 text-muted-foreground' />
              </div>
            </template>
          </ImageLoader>
        </div>

        <div class='text-center space-y-2'>
          <h1 class='text-2xl font-bold text-foreground select-text'>
            {{ album.name }}
          </h1>
          <p class='text-lg text-muted-foreground select-text'>
            <template v-if='albumArtistPairs.length'>
              <template v-for='(pair, index) in albumArtistPairs' :key='pair.id'>
                <RouterLink
                  :to='`/artists/${pair.id}`'
                  class='hover:underline'
                >
                  {{ pair.name }}
                </RouterLink>
                <span v-if='index < albumArtistPairs.length - 1'>, </span>
              </template>
            </template>
            <template v-else>
              {{ album?.artist || 'Unknown Artist' }}
            </template>
          </p>

          <!-- Meta chips -->
          <div class='flex flex-wrap justify-center items-center gap-x-2 gap-y-1 text-sm text-muted-foreground'>
            <span v-if='albumYear'>{{ albumYear }}</span>
            <span v-if='albumYear && albumSongs.length'>•</span>
            <span v-if='albumSongs.length'>{{ albumSongs.length }} songs</span>
            <span v-if='albumSongs.length'>•</span>
            <span v-if='albumSongs.length'>{{ formattedTotalDuration }}</span>
          </div>

          <!-- Genres -->
          <div v-if='albumGenres.length > 0' class='flex flex-wrap justify-center gap-1'>
            <span
              v-for='genre in albumGenres'
              :key='genre'
              class='px-2 py-1 text-xs font-semibold rounded-full bg-secondary/30 text-foreground'
            >
              {{ genre }}
            </span>
          </div>

          <!-- Actions -->
          <div class='flex items-center justify-center gap-2 pt-2'>
            <Button @click='playAll' size='sm'>
              <Play class='size-4 mr-2' />
              Play
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button size='sm' variant='outline'>
                  <MoreHorizontal class='size-4' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='center'>
                <DropdownMenuItem @click='shuffleAll'>
                  <Shuffle class='size-4 mr-2' />
                  Shuffle
                </DropdownMenuItem>
                <AddToPlaylistMenu :songs='albumSongs' type='dropdown' />
                <DropdownMenuItem @click='showShareDialog = true'>
                  <Share2 class='size-4 mr-2' />
                  Share
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>

      <!-- Songs -->
      <div class='w-full'>
        <h2 class='text-xl font-semibold text-foreground mb-3'>
          Songs
        </h2>
        <SongList
          @play-instant-mix='$emit("play-instant-mix", $event)'
          @play-song='playSongWithQueue'
          @toggle-favorite='(song) => $emit("toggle-favorite", song)'
          :loading='libraryLoading || !libraryLoaded || !album'
          :server-url='serverUrl'
          :show-album-art='false'
          :show-artist='hasMultipleArtists'
          :show-duration='true'
          :show-track-number='true'
          :songs='albumSongs'
          :token='token'
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
      :item-type='"album"'
    />
  </div>
</template>