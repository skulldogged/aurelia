<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Play } from 'lucide-vue-next'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, NameIdPair, Song } from '@/bindings'
  import { commands } from '@/bindings'
  import Carousel from '@/components/shared/Carousel.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import { Button } from '@/components/ui/button'
  import { Skeleton } from '@/components/ui/skeleton'
  import { uiLogger } from '@/lib/logger'
  import { withCustomState } from '@/lib/result'

  const router = useRouter()
  const { getRecentlyPlayed } = commands

  const props = defineProps<{
    allAlbums:      Album[],
    libraryLoaded:  boolean,
    libraryLoading: boolean,
    serverUrl:      string,
    token:          string,
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]],
    'select-album': [album: Album]
  }>()

  const recentlyPlayedSongs = ref<Song[]>([])

  const fetchRecentlyPlayed = async (): Promise<void> => {
    if (!props.serverUrl || !props.token) {
      uiLogger.error('Missing serverUrl or token props')
      return
    }

    await withCustomState(
      () => getRecentlyPlayed(props.serverUrl, props.token),
      {
        onError: error => {
          uiLogger.error('Failed to fetch recently played:', error)
        },
        onSuccess: songs => {
          recentlyPlayedSongs.value = songs
        },
      },
    )
  }

  onMounted(fetchRecentlyPlayed)

  // Refetch recently played songs when navigating back to this view
  watch(() => props.libraryLoaded, loaded => {
    if (loaded && props.serverUrl && props.token && recentlyPlayedSongs.value.length === 0)
      fetchRecentlyPlayed()
  })

  const mostPlayed = computed(() =>
    recentlyPlayedSongs.value.length > 0
      ? [...recentlyPlayedSongs.value]
        .sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
        .slice(0, 10)
      : [],
  )

  const recentlyPlayed = computed(() => recentlyPlayedSongs.value)

  const recentlyAdded = computed(() =>
    [...props.allAlbums]
      .filter(album => album.name && album.name.trim().length > 0)
      .sort((a, b) => {
        // Sort by date created descending (most recent first)
        const dateA = a.dateCreated ? new Date(a.dateCreated).getTime() : 0
        const dateB = b.dateCreated ? new Date(b.dateCreated).getTime() : 0
        return dateB - dateA
      })
      .slice(0, 10),
  )

  const randomAlbums = computed(() =>
    [...props.allAlbums]
      .filter(album => album.name && album.name.trim().length > 0)
      .sort(() => Math.random() - 0.5)
      .slice(0, 10),
  )

  const featuredAlbums = ref<Album[]>([])
  const currentFeaturedIndex = ref(0)

  const featuredAlbum = computed(() =>
    featuredAlbums.value[currentFeaturedIndex.value] || null,
  )

  // Extract all unique album artists from the featured album's songs
  const featuredAlbumArtistPairs = computed<NameIdPair[]>(() => {
    const album = featuredAlbum.value
    if (!album) return []

    const idToName = new Map<string, string>()
    const albumSongs = album.songs || []

    for (const song of albumSongs)
      if (song.albumArtists)
        for (const pair of song.albumArtists)
          if (pair.id && pair.name) idToName.set(pair.id, pair.name)

    // Fallbacks if albumArtists are not provided by backend
    if (idToName.size === 0) {
      const first = albumSongs[0]
      if (first?.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, idx) => {
          const name = first.artists![idx]
          if (id && name) idToName.set(id, name)
        })
      } else if (album.artist && album.artistId) {
        idToName.set(album.artistId, album.artist)
      }
    }

    return Array.from(idToName, ([id, name]) => ({ id, name }))
  })

  const isValidAlbumName = (name: null | string | undefined): boolean =>
    !!(name && name.trim().length > 0)

  const initializeFeaturedAlbums = (): void => {
    featuredAlbums.value = [...props.allAlbums].sort(() => 0.5 - Math.random())
  }

  const sortSongsByTrackOrder = (songs: Song[]): Song[] =>
    [...songs].sort((a, b) => {
      const trackA = a.trackNumber ?? Number.MAX_SAFE_INTEGER
      const trackB = b.trackNumber ?? Number.MAX_SAFE_INTEGER
      if (trackA !== trackB) return trackA - trackB
      return a.name.localeCompare(b.name)
    })

  const nextFeaturedAlbum = (): void => {
    if (featuredAlbums.value.length > 1)
      currentFeaturedIndex.value = (currentFeaturedIndex.value + 1) % featuredAlbums.value.length
  }

  const prevFeaturedAlbum = (): void => {
    if (featuredAlbums.value.length > 1)
      currentFeaturedIndex.value = currentFeaturedIndex.value === 0
        ? featuredAlbums.value.length - 1
        : currentFeaturedIndex.value - 1
  }

  watch(() => props.allAlbums, () => {
    initializeFeaturedAlbums()
  }, { immediate: true })

  const playSongs = (songs: Song[], startWith?: Song): void => {
    if (songs.length === 0) {
      uiLogger.warn('No songs to play')
      return
    }

    const invalidSongs = songs.filter(song => !song || !song.id)

    if (invalidSongs.length > 0)
      uiLogger.error('Found songs with invalid IDs:', invalidSongs)

    if (startWith) {
      const startIndex = songs.findIndex(song => song.id === startWith.id)
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

  const playFeaturedAlbum = (): void => {
    if (!featuredAlbum.value) {
      uiLogger.warn('No featured album available')
      return
    }

    const albumSongs = featuredAlbum.value.songs || []
    if (albumSongs.length > 0) {
      emit('play-songs', sortSongsByTrackOrder(albumSongs))
      if (isValidAlbumName(featuredAlbum.value.name)) {
        router.push(`/songs/album/${encodeURIComponent(featuredAlbum.value.name)}`)
      }
    } else {
      uiLogger.warn('No songs found for featured album')
    }
  }

  const playAlbumSongs = (album: Album): void => {
    // Use the album's songs array if available (more efficient)
    if (album.songs && album.songs.length > 0) {
      emit('play-songs', sortSongsByTrackOrder(album.songs))
    } else {
      uiLogger.warn('No songs found for album', album.name)
    }
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <h1 class='text-4xl font-bold'>
        Home
      </h1>
    </div>

    <!-- Featured Album Section -->
    <div
      v-if='featuredAlbum || libraryLoading'
      class='relative isolate rounded-2xl p-8 mb-8 overflow-hidden blur-card'
    >
      <!-- Blurred Background -->
      <div class='absolute inset-0 bg-cover bg-center bg-no-repeat rounded-2xl blur-md scale-105 overflow-hidden'>
        <ImageLoader
          v-if='featuredAlbum && !libraryLoading'
          :item-id='featuredAlbum.id || featuredAlbum.name'
          :server-url='serverUrl'
          :token='token'
          class='size-full object-cover'
        />
        <div class='absolute inset-0 bg-black/60 rounded-2xl' />
      </div>

      <!-- Content -->
      <div class='relative z-10 flex items-center space-x-6'>
        <div class='flex-shrink-0'>
          <template v-if='libraryLoading'>
            <Skeleton class='size-48 rounded-xl' />
          </template>
          <template v-else-if='featuredAlbum'>
            <ImageLoader
              :alt='`${featuredAlbum.name} album art`'
              :item-id='featuredAlbum.id || featuredAlbum.name'
              :server-url='serverUrl'
              :token='token'
              class='size-48 rounded-xl shadow-2xl object-cover'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='size-48 rounded-xl shadow-2xl'
                  size='large'
                  type='album'
                />
              </template>
            </ImageLoader>
          </template>
        </div>
        <div class='flex-1 min-w-0'>
          <template v-if='libraryLoading'>
            <Skeleton class='h-10 w-3/4 mb-2' />
            <Skeleton class='h-7 w-1/2 mb-4' />
            <Skeleton class='h-5 w-1/4 mb-6' />
            <button
              class='
                bg-white/20 backdrop-blur-sm text-white px-8
                py-3 rounded-full font-semibold border
                border-white/20 opacity-50 cursor-not-allowed
              '
              disabled
            >
              Play Album
            </button>
          </template>
          <template v-else-if='featuredAlbum'>
            <h1 class='text-4xl font-bold mb-2 text-white drop-shadow-lg truncate'>
              <router-link
                v-if='isValidAlbumName(featuredAlbum.name)'
                :to="{ name: 'album-detail', params: { albumName: featuredAlbum.name } }"
              >
                {{ featuredAlbum.name }}
              </router-link>
              <span v-else>{{ featuredAlbum.name }}</span>
            </h1>
            <p class='text-xl text-white/90 mb-4 drop-shadow-md'>
              <template v-if='featuredAlbumArtistPairs.length'>
                <template v-for='(pair, index) in featuredAlbumArtistPairs' :key='pair.id'>
                  <router-link
                    :to="{ name: 'artist-detail', params: { artistId: pair.id } }"
                    class='hover:underline'
                  >
                    {{ pair.name }}
                  </router-link>
                  <span v-if='index < featuredAlbumArtistPairs.length - 1'>, </span>
                </template>
              </template>
              <template v-else>
                <router-link
                  v-if='featuredAlbum.artistId'
                  :to="{ name: 'artist-detail', params: { artistId: featuredAlbum.artistId } }"
                  class='hover:underline'
                >
                  {{ featuredAlbum.artist }}
                </router-link>
                <span v-else>{{ featuredAlbum.artist }}</span>
              </template>
            </p>
            <p class='text-sm text-white/80 mb-6 drop-shadow-md'>
              {{ featuredAlbum.songs?.length || 0 }} songs
            </p>
            <button
              @click='playFeaturedAlbum'
              :disabled='libraryLoading'
              class='
                  bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white px-8
                  py-3 rounded-full font-semibold transition-colors border
                  border-white/20 disabled:opacity-50 disabled:cursor-not-allowed
                '
            >
              Play Album
            </button>
          </template>
        </div>
      </div>

      <!-- Navigation Arrows -->
      <div v-if='featuredAlbums.length > 1' class='absolute bottom-4 right-4 z-20 flex space-x-2'>
        <button
          @click='prevFeaturedAlbum'
          :disabled='libraryLoading'
          class='
              flex items-center justify-center bg-white/20 p-2 text-white
              backdrop-blur-sm transition-colors hover:bg-white/30
              border border-white/20 rounded-full disabled:opacity-50 disabled:cursor-not-allowed
            '
        >
          <ChevronLeft class='h-5 w-5' />
        </button>
        <button
          @click='nextFeaturedAlbum'
          :disabled='libraryLoading'
          class='
            flex items-center justify-center bg-white/20 p-2 text-white
            backdrop-blur-sm transition-colors hover:bg-white/30
            border border-white/20 rounded-full disabled:opacity-50 disabled:cursor-not-allowed
          '
        >
          <ChevronRight class='h-5 w-5' />
        </button>
      </div>
    </div>

    <Carousel
      :disabled='libraryLoading || !libraryLoaded || recentlyPlayedSongs.length === 0'
      class='mb-8'
      title='Most Played'
    >
      <template v-if='libraryLoading || !libraryLoaded || recentlyPlayedSongs.length === 0'>
        <div
          v-for='n in 10'
          :key='`most-played-skeleton-${n}`'
          class='cursor-pointer group'
        >
          <div class='relative mb-2'>
            <Skeleton class='album-art-image' />
          </div>
          <Skeleton class='h-6 w-3/4 mb-1' />
          <Skeleton class='h-4 w-1/2' />
        </div>
      </template>
      <template v-else>
        <div
          v-for='song in mostPlayed'
          @click='playSongs(mostPlayed, song)'
          :key='song.id'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-2'>
            <ImageLoader
              :item-id='song.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='album-art-image'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='album-art-image'
                  size='large'
                  type='album-art'
                />
              </template>
            </ImageLoader>

            <!-- Play button overlay -->
            <div
              class='
                     absolute inset-0 bg-black/50 rounded-lg opacity-0
                     group-hover:opacity-100 transition-opacity flex items-center
                     justify-center
                   '
            >
              <Button
                @click.stop='playSongs(mostPlayed, song)'
                :disabled='libraryLoading'
                class='
                       bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                       border-white/20 disabled:opacity-50 disabled:cursor-not-allowed
                     '
                size='icon'
              >
                <Play class='h-4 w-4' />
              </Button>
            </div>
          </div>
          <p class='font-semibold truncate'>
            {{ song.name }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
              <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                <router-link
                  @click.stop
                  :to="{ name: 'artist-detail', params: { artistId: song.artistIds[index] } }"
                  class='hover:underline'
                >
                  {{ artist }}
                </router-link>
                <span v-if='index < song.artists.length - 1'>, </span>
              </template>
            </template>
            <template v-else>
              {{ song.artists?.join(', ') }}
            </template>
          </p>
        </div>
      </template>
    </Carousel>

    <Carousel
      :disabled='libraryLoading || !libraryLoaded || recentlyPlayedSongs.length === 0'
      class='mb-8'
      title='Recently Played'
    >
      <template v-if='libraryLoading || !libraryLoaded || recentlyPlayedSongs.length === 0'>
        <div
          v-for='n in 10'
          :key='`recently-played-skeleton-${n}`'
          class='cursor-pointer group'
        >
          <div class='relative mb-2'>
            <Skeleton class='album-art-image' />
          </div>
          <Skeleton class='h-6 w-3/4 mb-1' />
          <Skeleton class='h-4 w-1/2' />
        </div>
      </template>
      <template v-else>
        <div
          v-for='song in recentlyPlayed'
          @click='playSongs(recentlyPlayed, song)'
          :key='song.id'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-2'>
            <ImageLoader
              :item-id='song.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='album-art-image'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='album-art-image'
                  size='large'
                  type='album-art'
                />
              </template>
            </ImageLoader>

            <!-- Play button overlay -->
            <div
              class='
                  absolute inset-0 bg-black/50 rounded-lg opacity-0
                  group-hover:opacity-100 transition-opacity flex items-center
                  justify-center
                '
            >
              <Button
                @click.stop='playSongs(recentlyPlayed, song)'
                :disabled='libraryLoading'
                class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                    border-white/20 disabled:opacity-50 disabled:cursor-not-allowed
                  '
                size='icon'
              >
                <Play class='h-4 w-4' />
              </Button>
            </div>
          </div>
          <p class='font-semibold truncate'>
            {{ song.name }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
              <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                <router-link
                  @click.stop
                  :to="{ name: 'artist-detail', params: { artistId: song.artistIds[index] } }"
                  class='hover:underline'
                >
                  {{ artist }}
                </router-link>
                <span v-if='index < song.artists.length - 1'>, </span>
              </template>
            </template>
            <template v-else>
              {{ song.artists?.join(', ') }}
            </template>
          </p>
        </div>
      </template>
    </Carousel>

    <Carousel :disabled='libraryLoading || !libraryLoaded' class='mb-8' title='Recently Added'>
      <template v-if='libraryLoading || !libraryLoaded'>
        <div
          v-for='n in 10'
          :key='`recently-added-skeleton-${n}`'
          class='cursor-pointer group'
        >
          <div class='relative mb-2'>
            <Skeleton class='album-art-image' />
          </div>
          <Skeleton class='h-6 w-3/4 mb-1' />
          <Skeleton class='h-4 w-1/2' />
        </div>
      </template>
      <template v-else>
        <div
          v-for='album in recentlyAdded'
          @click="$emit('select-album', album)"
          :key='album.name'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-2'>
            <ImageLoader
              :item-id='album.id || album.name'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='album-art-image'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='album-art-image'
                  size='large'
                  type='album'
                />
              </template>
            </ImageLoader>

            <!-- Play button overlay -->
            <div
              class='
                  absolute inset-0 bg-black/50 rounded-lg opacity-0
                  group-hover:opacity-100 transition-opacity flex items-center
                  justify-center
                '
            >
              <Button
                @click.stop='playAlbumSongs(album)'
                :disabled='libraryLoading'
                class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                    border-white/20 disabled:opacity-50 disabled:cursor-not-allowed
                  '
                size='icon'
              >
                <Play class='h-4 w-4' />
              </Button>
            </div>
          </div>
          <p class='font-semibold truncate'>
            {{ album.name }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            <router-link
              @click.stop
              v-if='album.artistId'
              :to="{ name: 'artist-detail', params: { artistId: album.artistId } }"
              class='hover:underline'
            >
              {{ album.artist }}
            </router-link>
            <span v-else>{{ album.artist }}</span>
          </p>
        </div>
      </template>
    </Carousel>

    <Carousel :disabled='libraryLoading || !libraryLoaded' class='mb-8' title='From Your Library'>
      <template v-if='libraryLoading || !libraryLoaded'>
        <div
          v-for='n in 10'
          :key='`library-skeleton-${n}`'
          class='cursor-pointer group'
        >
          <div class='relative mb-2'>
            <Skeleton class='album-art-image' />
          </div>
          <Skeleton class='h-6 w-3/4 mb-1' />
          <Skeleton class='h-4 w-1/2' />
        </div>
      </template>
      <template v-else>
        <div
          v-for='album in randomAlbums'
          @click="$emit('select-album', album)"
          :key='album.name'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-2'>
            <ImageLoader
              :item-id='album.id || album.name'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='album-art-image'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='album-art-image'
                  size='large'
                  type='album'
                />
              </template>
            </ImageLoader>

            <!-- Play button overlay -->
            <div
              class='
                  absolute inset-0 bg-black/50 rounded-lg opacity-0
                  group-hover:opacity-100 transition-opacity flex items-center
                  justify-center
                '
            >
              <Button
                @click.stop='playAlbumSongs(album)'
                :disabled='libraryLoading'
                class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                    border-white/20 disabled:opacity-50 disabled:cursor-not-allowed
                  '
                size='icon'
              >
                <Play class='h-4 w-4' />
              </Button>
            </div>
          </div>
          <p class='font-semibold truncate'>
            {{ album.name }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            <router-link
              @click.stop
              v-if='album.artistId'
              :to="{ name: 'artist-detail', params: { artistId: album.artistId } }"
              class='hover:underline'
            >
              {{ album.artist }}
            </router-link>
            <span v-else>{{ album.artist }}</span>
          </p>
        </div>
      </template>
    </Carousel>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

.album-art-image {
  @apply w-full h-auto rounded-lg shadow-lg group-hover:opacity-75 aspect-square object-cover transition-opacity;
}
</style>
