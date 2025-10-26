<script setup lang="ts">
  import { MoreHorizontal, Music, Pause, Play, Share2, Shuffle, Star } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import { Album, Artist, commands, Song } from '@/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import AlbumStack from '@/components/shared/AlbumStack.vue'
  import Carousel from '@/components/shared/Carousel.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { logger } from '@/lib/logger'
  import { usePlayerStore } from '@/stores'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const topSongsCount = 5

  const emit = defineEmits<{
    'play-song':     [song: Song],
    'play-songs':    [songs: Song[]],
    'select-album':  [album: Album],
    'select-artist': [artist: Artist],
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()
  const playerStore = usePlayerStore()

  // Create computed properties from stores
  const allArtists = computed(() => libraryStore.allArtistsWithSongs as Artist[])
  const allSongs = computed(() => libraryStore.allSongs as Song[])
  const libraryLoaded = computed(() => libraryStore.isLoaded)
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const route = useRoute()
  const id = computed(() => {
    const params = route.params
    if ('id' in params) {
      const param = params.id
      if (typeof param === 'string') return param
      if (Array.isArray(param)) return param[0] ?? ''
    }
    return ''
  })
  const showFullOverview = ref(false)
  const showShareDialog = ref(false)

  const artist = computed(() =>
    libraryLoaded.value && allArtists.value.length
      ? allArtists.value.find(a => a.id === id.value) || null
      : null,
  )

  const artistSongs = computed(() =>
    artist.value && artist.value.songs
      ? [...artist.value.songs].sort((a, b) => (b.playCount ?? 0) - (a.playCount ?? 0))
      : [],
  )

  const artistAlbums = computed(() => {
    if (!artist.value || !artist.value.songs) return []
    const albumIds = new Set(artist.value.songs.map(s => s.albumId).filter(Boolean))
    return libraryStore.allAlbums.filter(album => album.id && albumIds.has(album.id)) as Album[]
  })

  const relatedArtists = ref<Artist[]>([])

  watch(artist, async newArtist => {
    if (newArtist) {
      logger.info('Starting to get related artists')
      const start = Date.now()
      const result = await commands.getRelatedArtists(newArtist.id)
      if (result.status === 'ok') {
        relatedArtists.value = result.data as Artist[]
      }
      logger.info(`Got related artists in ${Date.now() - start}ms`)
    }
  })

  const primarySongs = computed(() => artistSongs.value.filter(song => song.artists?.[0] === artist.value?.name))
  const featuredSongs = computed(() => artistSongs.value.filter(song => song.artists?.[0] !== artist.value?.name))
  const isFeaturedOnlyArtist = computed(() => primarySongs.value.length === 0 && featuredSongs.value.length > 0)

  const playArtistShuffle = (): void => {
    if (artistSongs.value.length > 0) {
      const shuffledSongs = [...artistSongs.value].sort(() => 0.5 - Math.random())
      emit('play-songs', shuffledSongs)
    }
  }

  const artistGenres = computed(() => {
    if (!artistSongs.value.length)
      return []

    const genreCounts = new Map<string, number>()
    artistSongs.value.forEach(song => {
      song.genres?.forEach(genre => {
        genreCounts.set(genre, (genreCounts.get(genre) || 0) + 1)
      })
    })

    if (!genreCounts.size)
      return []

    const sortedGenres = [...genreCounts.entries()].sort((a, b) => b[1] - a[1])

    return sortedGenres.slice(0, 5).map(([genre]) => genre)
  })

  const isFeaturedOnSong = (song: Song): boolean =>
    song.artists?.[0] !== artist.value?.name && !!song.artists?.includes(artist.value?.name || '')

  type SimpleArtist = { id: null | string, name: string }

  const collaboratorsFor = (song: Song): SimpleArtist[] => {
    const current = artist.value?.name
    const artists = song.artists || []
    const ids = song.artistIds || []

    // Build pairs for as many mapped entries as possible
    const pairs: SimpleArtist[] = []
    for (let i = 0; i < artists.length; i++) {
      const name = artists[i]
      const id = ids[i] || null
      if (name && name !== current)
        pairs.push({ id, name })
    }
    return pairs
  }

  const albumTrackCountsById = computed(() => {
    const counts = new Map<string, number>()

    for (const s of allSongs.value)
      if (s.albumId)
        counts.set(s.albumId, (counts.get(s.albumId) || 0) + 1)

    return counts
  })

  const isSingle = (song: Song): boolean => {
    const sameName = (song.album || '').trim().toLowerCase() === (song.name || '').trim().toLowerCase()
    const trackCount = song.albumId ? (albumTrackCountsById.value.get(song.albumId) || 0) : 0
    return !!song.album && sameName && trackCount <= 1
  }

  const isAlbumSingle = (album: Album): boolean => {
    if (!album) return false

    const tracks = album.songs || []

    if (tracks.length === 1) {
      const only = tracks[0]
      const sameName = (album.name || '').trim().toLowerCase() === (only.name || '').trim().toLowerCase()
      return sameName
    }

    if (album.id) {
      const count = albumTrackCountsById.value.get(album.id) || 0
      if (count === 1) return true
    }

    return false
  }

  type NameId = { id: null | string, name: string }

  const albumArtistPairsFor = (album: Album): NameId[] => {
    const pairs = new Map<string, string>()
    const tracks = album.songs || []

    for (const s of tracks)
      if (s.albumArtists)
        for (const p of s.albumArtists)
          if (p.id && p.name) pairs.set(p.id, p.name)

    if (pairs.size === 0 && tracks.length) {
      const first = tracks[0]
      if (first.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, i) => {
          const name = first.artists![i]
          if (id && name) pairs.set(id, name)
        })
      } else if (album.artistId && album.artist) {
        pairs.set(album.artistId, album.artist)
      }
    }

    return Array.from(pairs, ([id, name]) => ({ id, name }))
  }

  // Album carousel: collaborators excluding the current artist only
  const albumCollaboratorsFor = (album: Album): NameId[] =>
    albumArtistPairsFor(album).filter(p => p.name !== artist.value?.name)

  const playSong = (song: Song): void => {
    emit('play-song', song)
  }

  const playAlbum = (album: Album): void => {
    if (album.songs && album.songs.length > 0)
      emit('play-songs', [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)))
  }
</script>

<template>
  <div class='px-4 pb-4' style='padding-top: env(safe-area-inset-top)'>
    <div v-if='libraryLoading || !libraryLoaded || !artist' class='space-y-6'>
      <!-- Header Skeleton -->
      <div class='flex flex-col items-center p-6 bg-sidebar rounded-lg gap-4'>
        <Skeleton class='w-32 h-32 rounded-lg' />
        <div class='text-center space-y-2'>
          <Skeleton class='h-8 w-48 mx-auto' />
          <Skeleton class='h-5 w-32 mx-auto' />
          <Skeleton class='h-5 w-40 mx-auto' />
          <div class='flex justify-center gap-2 pt-2'>
            <Button size='sm' disabled>
              <Shuffle class='size-4 mr-2' />
              Shuffle All
            </Button>
          </div>
        </div>
      </div>

      <!-- Top Songs Skeleton -->
      <div class='space-y-3'>
        <Skeleton class='h-6 w-32' />
        <div class='space-y-2'>
          <div
            v-for='i in topSongsCount'
            :key='`top-song-skeleton-${i}`'
            class='flex items-center py-2 px-2 rounded-md'
          >
            <Skeleton class='size-8 rounded-md mr-3' />
            <div class='flex-1 space-y-1'>
              <Skeleton class='h-4 w-3/4' />
              <Skeleton class='h-3 w-1/2' />
            </div>
            <Skeleton class='h-3 w-10 ml-3' />
          </div>
        </div>
      </div>

      <!-- Albums Skeleton -->
      <div class='space-y-3'>
        <Skeleton class='h-6 w-24' />
        <div class='grid grid-cols-2 gap-4'>
          <div v-for='i in 4' :key='`albums-skeleton-${i}`' class='cursor-pointer group'>
            <div class='relative mb-2'>
              <Skeleton class='w-full aspect-square rounded-lg' />
            </div>
            <Skeleton class='h-5 w-3/4 mb-1' />
            <Skeleton class='h-3 w-1/2' />
          </div>
        </div>
      </div>
    </div>

    <div v-else-if='artist' class='space-y-6'>
      <!-- Header -->
      <div class='flex flex-col items-center p-6 bg-sidebar rounded-lg gap-4'>
        <div class='shrink-0'>
          <ImageLoader
            :item-id='artist.id'
            :server-url='serverUrl'
            :token='token'
            alt='Artist art'
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
          <h2 class='text-2xl font-bold'>
            {{ artist.name }}
          </h2>
          <div class='flex flex-wrap justify-center items-center gap-x-3 gap-y-1 text-sm text-muted-foreground'>
            <div v-if='artist.communityRating' class='flex items-center gap-1'>
              <Star class='size-3 text-yellow-500' />
              <span>{{ artist.communityRating.toFixed(1) }} / 10</span>
            </div>
            <p v-if='isFeaturedOnlyArtist'>
              Featured on {{ featuredSongs.length }} {{ featuredSongs.length === 1 ? 'song' : 'songs' }}
            </p>
            <p v-else>
              {{ artistSongs.length }} songs • {{ artistAlbums.length }} albums
            </p>
          </div>
          <div v-if='artistGenres.length > 0' class='flex flex-wrap justify-center gap-1'>
            <span
              v-for='genre in artistGenres'
              :key='genre'
              class='px-2 py-1 text-xs font-semibold rounded-full bg-secondary/30 text-foreground'
            >
              {{ genre }}
            </span>
          </div>

          <!-- Actions -->
          <div class='flex items-center justify-center gap-2'>
            <Button @click='playArtistShuffle' size='sm'>
              <Shuffle class='size-4 mr-2' />
              Shuffle All
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button size='sm' variant='outline'>
                  <MoreHorizontal class='size-4' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='center'>
                <AddToPlaylistMenu :songs='artistSongs' type='dropdown' />
                <DropdownMenuItem @click='showShareDialog = true'>
                  <Share2 class='size-4 mr-2' />
                  Share
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>

      <!-- About Section -->
      <div v-if='artist.overview' class='bg-sidebar rounded-lg p-4'>
        <div class='prose dark:prose-invert max-w-none text-sm'>
          <p :class="{ 'line-clamp-3': !showFullOverview }" v-html='artist.overview' />
          <Button
            @click='showFullOverview = !showFullOverview'
            v-if='artist.overview.length > 150'
            class='px-0 text-xs'
            variant='link'
          >
            {{ showFullOverview ? 'Show Less' : 'Read More' }}
          </Button>
        </div>
      </div>

      <!-- Top Songs -->
      <div class='bg-sidebar rounded-lg p-4 space-y-3'>
        <h3 class='text-lg font-semibold'>
          {{ isFeaturedOnlyArtist ? 'Features' : 'Top Songs' }}
        </h3>
        <div class='space-y-1'>
          <div
            v-for='song in artistSongs.slice(0, topSongsCount)'
            @click='playSong(song)'
            :key='song.id'
            class='flex items-center p-2 hover:bg-muted/50 rounded-md cursor-pointer transition-colors group'
          >
            <div class='relative mr-3 shrink-0'>
              <ImageLoader
                :item-id='song.albumId || song.id'
                :server-url='serverUrl'
                :token='token'
                alt='Album art'
                class='size-8 rounded-md object-cover'
              >
                <template #fallback>
                  <ImagePlaceholder
                    class='size-8 rounded-md'
                    size='small'
                    type='album'
                  />
                </template>
              </ImageLoader>
              <div
                :class="[
                  'absolute inset-0 size-8 flex items-center justify-center transition-opacity',
                  playerStore.currentSong?.id === song.id && playerStore.isPlaying
                    ? 'opacity-100'
                    : 'opacity-0 group-hover:opacity-100',
                ]"
              >
                <Button
                  class='size-6 rounded-full bg-background/75 text-foreground hover:bg-background'
                  size='icon'
                  variant='ghost'
                >
                  <Pause v-if='playerStore.currentSong?.id === song.id && playerStore.isPlaying' class='size-3' />
                  <Play v-else class='size-3' />
                </Button>
              </div>
            </div>
            <div class='flex-1 min-w-0'>
              <p class='text-foreground font-medium text-sm truncate'>
                {{ song.name }}
              </p>
              <p
                v-if='(song.album && !isSingle(song)) || isFeaturedOnSong(song)'
                class='text-muted-foreground text-xs truncate'
              >
                <RouterLink
                  @click.stop
                  v-if='song.album && song.albumId && !isSingle(song)'
                  :to='`/albums/${song.albumId}`'
                  class='hover:underline'
                >
                  {{ song.album }}
                </RouterLink>
                <span v-if='(song.album && !isSingle(song)) && isFeaturedOnSong(song)' class='mx-1'>•</span>
                <span v-if='isFeaturedOnSong(song)'>
                  with
                  <template v-for='(collab, idx) in collaboratorsFor(song)' :key='collab.id || collab.name'>
                    <RouterLink
                      @click.stop
                      v-if='collab.id'
                      :to='`/artists/${collab.id}`'
                      class='hover:underline'
                    >
                      {{ collab.name }}
                    </RouterLink>
                    <span v-else>{{ collab.name }}</span>
                    <span v-if='idx < collaboratorsFor(song).length - 1'>, </span>
                  </template>
                </span>
              </p>
            </div>
            <p class='text-muted-foreground text-xs'>
              {{ song.playCount ?? 0 }}
            </p>
          </div>
        </div>
      </div>

      <!-- Albums Carousel -->
      <Carousel
        v-if='artistAlbums.length > 0'
        :disabled='libraryLoading || !libraryLoaded || !artist'
        :title="isFeaturedOnlyArtist ? 'Appears On' : 'Albums'"
      >
        <div
          v-for='album in artistAlbums'
          @click="$emit('select-album', { ...album })"
          :key='album.name'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-2'>
            <AlbumStack
              @play='playAlbum'
              :album='album'
              :disabled='libraryLoading'
              :server-url='serverUrl'
              :token='token'
              size='responsive'
            />
          </div>
          <div>
            <h3 class='font-medium text-sm truncate'>
              {{ album.name }}
            </h3>
            <p v-if='albumCollaboratorsFor(album).length' class='text-xs text-muted-foreground truncate'>
              with
              <template v-for='(pair, idx) in albumCollaboratorsFor(album)' :key='pair.id || pair.name'>
                <RouterLink
                  @click.stop
                  v-if='pair.id'
                  :to='`/artists/${pair.id}`'
                  class='hover:underline'
                >
                  {{ pair.name }}
                </RouterLink>
                <span v-else>{{ pair.name }}</span>
                <span v-if='idx < albumCollaboratorsFor(album).length - 1'>, </span>
              </template>
            </p>
            <p class='text-xs text-muted-foreground'>
              <span v-if='isAlbumSingle(album)'>Single</span>
              <span v-else>{{ album.songs?.length || 0 }} songs</span>
            </p>
          </div>
        </div>
      </Carousel>

      <!-- Related Artists -->
      <Carousel
        v-if='relatedArtists.length > 0'
        :disabled='libraryLoading || !libraryLoaded || !artist'
        title='Related Artists'
      >
        <div
          v-for='relatedArtist in relatedArtists'
          @click="$emit('select-artist', relatedArtist)"
          :key='relatedArtist.name'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-2'>
            <ImageLoader
              :item-id='relatedArtist.id'
              :server-url='serverUrl'
              :token='token'
              alt='Artist art'
              class='w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                  size='large'
                  type='artist'
                />
              </template>
            </ImageLoader>
          </div>
          <div class='text-center'>
            <h3 class='font-medium text-sm truncate'>
              {{ relatedArtist.name }}
            </h3>
            <p class='text-xs text-muted-foreground truncate'>
              {{ relatedArtist.songCount }} {{ relatedArtist.songCount === BigInt(1) ? 'song' : 'songs' }}
            </p>
          </div>
        </div>
      </Carousel>
    </div>

    <div v-else class='text-center py-12 text-muted-foreground'>
      Artist not found.
    </div>

    <ShareDialog
      v-if='artist'
      v-model:open='showShareDialog'
      :item-id='artist.id'
      :item-name='artist.name'
      :item-type='"artist"'
    />
  </div>
</template>

<style scoped>
@reference "tailwindcss";

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
}

.carousel-container::before,
.carousel-container::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  z-index: 2;
  pointer-events: none;
  width: 3rem;
  transition: opacity 0.2s ease-in-out;
}

.carousel-container::before {
  left: 0;
  background-image: linear-gradient(to right, var(--background), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.carousel-container::after {
  right: 0;
  background-image: linear-gradient(to left, var(--background), transparent);
  opacity: var(--right-fade-opacity, 0);
}

</style>