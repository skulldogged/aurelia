<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Play } from 'lucide-vue-next'
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, NameIdPair, Song } from '@/bindings'
  import AlbumGrid from '@/components/shared/AlbumGrid.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { Skeleton } from '@/components/ui/skeleton'
  import { logger } from '@/lib/logger'
  import { sortSongsByTrackOrder } from '@/lib/transforms'
  import { useAuthStore, useHomeStore } from '@/stores'

  const router = useRouter()
  const authStore = useAuthStore()
  const homeStore = useHomeStore()

  const credentials = computed(() => ({
    serverUrl: authStore.serverUrl,
    token:     authStore.token,
    userId:    authStore.userId,
    username:  authStore.username,
  }))

  const serverUrl = computed(() => credentials.value.serverUrl)
  const token = computed(() => credentials.value.token)

  defineProps<{
    currentSong: null | Song
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]],
    'select-album': [album: Album]
  }>()

  const isLoading = computed(() => homeStore.isLoading)
  const recentlyPlayed = computed(() => homeStore.recentlyPlayedSongs)
  const recentlyAdded = computed(() => homeStore.recentlyAddedAlbums)
  const randomAlbums = computed(() => homeStore.randomLibraryAlbums)
  const featuredAlbums = computed(() => homeStore.featuredLibraryAlbums)
  const currentFeaturedIndex = ref(0)

  const featuredAlbum = computed(() =>
    featuredAlbums.value[currentFeaturedIndex.value] || null,
  )

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

  const mostPlayed = computed(() =>
    recentlyPlayed.value.length > 0
      ? [...recentlyPlayed.value]
        .sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
        .slice(0, 10)
      : [],
  )

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

  const playSongs = (songs: Song[], startWith?: Song): void => {
    if (songs.length === 0) {
      logger.warn('No songs to play')
      return
    }

    const invalidSongs = songs.filter(song => !song || !song.id)

    if (invalidSongs.length > 0)
      logger.error('Found songs with invalid IDs:', invalidSongs)

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
      logger.warn('No featured album available')
      return
    }

    const albumSongs = featuredAlbum.value.songs || []
    if (albumSongs.length > 0) {
      emit('play-songs', sortSongsByTrackOrder(albumSongs))
      if (featuredAlbum.value.id) {
        router.push(`/albums/${featuredAlbum.value.id}`)
      }
    } else {
      logger.warn('No songs found for featured album')
    }
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='space-y-8'>
      <!-- Featured Album Section -->
      <div
        v-if='featuredAlbum || isLoading'
        :style="{
          minHeight: '400px',
          marginBottom: 'calc(-env(safe-area-inset-top) + 2rem)',
          position: 'relative',
          top: '-env(safe-area-inset-top)'
        }"
        class='relative isolate bg-sidebar mb-8 overflow-hidden rounded-none -mx-4 -mt-4'
      >
        <!-- Background Image -->
        <div
          class='absolute bg-cover bg-center bg-no-repeat -top-4 left-0 right-0 bottom-0'
        >
          <ImageLoader
            v-if='featuredAlbum && !isLoading'
            :item-id='featuredAlbum.id || featuredAlbum.name'
            :server-url='serverUrl'
            :token='token'
            class='size-full object-cover'
          />
          <div
            class='absolute inset-0 bg-black/50'
          />
          <div
            class='
              absolute bottom-0 left-0 right-0 h-24 bg-linear-to-t
              from-background via-background/80 to-transparent
            '
          />
        </div>

        <!-- Content -->
        <div
          class='z-10 absolute bottom-0 left-0 right-0 flex flex-col p-4'
        >
          <!-- Mobile Portrait Content -->
          <div
            class='flex-1 min-w-0 text-left'
          >
            <template v-if='isLoading'>
              <Skeleton class='h-12 md:h-14 w-3/4 mb-3' />
              <Skeleton class='h-8 md:h-9 w-1/2 mb-4' />
              <Skeleton class='h-6 md:h-6 w-1/4 mb-6' />
            </template>
            <template v-else-if='featuredAlbum'>
              <h1 class='text-4xl md:text-5xl font-bold mb-3 text-white drop-shadow-lg truncate'>
                <router-link
                  v-if='featuredAlbum.id'
                  :to='`/albums/${featuredAlbum.id}`'
                >
                  {{ featuredAlbum.name }}
                </router-link>
                <span v-else>{{ featuredAlbum.name }}</span>
              </h1>
              <p class='text-2xl md:text-2xl text-white/90 mb-3 drop-shadow-md'>
                <template v-if='featuredAlbumArtistPairs.length'>
                  <template v-for='(pair, index) in featuredAlbumArtistPairs' :key='pair.id'>
                    <router-link
                      :to='`/artists/${pair.id}`'
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
                    :to='`/artists/${featuredAlbum.artistId}`'
                    class='hover:underline'
                  >
                    {{ featuredAlbum.artist }}
                  </router-link>
                  <span v-else>{{ featuredAlbum.artist }}</span>
                </template>
              </p>
              <p class='text-lg md:text-lg text-white/80 mb-6 drop-shadow-md'>
                {{ featuredAlbum.songs?.length || 0 }} songs
              </p>
            </template>
          </div>

          <!-- Play Button - Mobile Portrait -->
          <div
            class='shrink-0'
          >
            <template v-if='isLoading'>
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
              <button
                @click='playFeaturedAlbum'
                class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white px-8 py-3
                       rounded-full font-semibold transition-colors border border-white/20 flex items-center gap-2'
              >
                <Play class='h-5 w-5 md:hidden' />
                <span class='hidden md:inline'>Play Album</span>
              </button>
            </template>
          </div>
        </div>

        <!-- Navigation Arrows -->
        <div
          v-if='featuredAlbums.length > 1'
          class='absolute z-20 flex space-x-2 bottom-4 right-4'
        >
          <button
            @click='prevFeaturedAlbum'
            :disabled='isLoading'
            class='flex items-center justify-center bg-white/20 p-2 text-white backdrop-blur-sm
                   transition-colors hover:bg-white/30 border border-white/20 rounded-full
                   disabled:opacity-50 disabled:cursor-not-allowed'
          >
            <ChevronLeft class='h-5 w-5' />
          </button>
          <button
            @click='nextFeaturedAlbum'
            :disabled='isLoading'
            class='flex items-center justify-center bg-white/20 p-2 text-white backdrop-blur-sm
                   transition-colors hover:bg-white/30 border border-white/20 rounded-full
                   disabled:opacity-50 disabled:cursor-not-allowed'
          >
            <ChevronRight class='h-5 w-5' />
          </button>
        </div>
      </div>

      <AlbumGrid
        @play-songs='playSongs'
        :is-loading='isLoading'
        :items='mostPlayed'
        :title="'Most Played'"
        :type="'song'"
      />

      <AlbumGrid
        @play-songs='playSongs'
        :is-loading='isLoading'
        :items='recentlyPlayed'
        :title="'Recently Played'"
        :type="'song'"
      />

      <AlbumGrid
        @play-songs='playSongs'
        @select-album="(album) => $emit('select-album', album)"
        :is-loading='isLoading'
        :items='recentlyAdded'
        :title="'Recently Added'"
        :type="'album'"
      />

      <AlbumGrid
        @play-songs='playSongs'
        @select-album="(album) => $emit('select-album', album)"
        :is-loading='isLoading'
        :items='randomAlbums'
        :title="'From Your Library'"
        :type="'album'"
      />
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
}

</style>