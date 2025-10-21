<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Play, Shuffle } from 'lucide-vue-next'
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, NameIdPair, Song } from '@/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import AlbumStack from '@/components/shared/AlbumStack.vue'
  import Carousel from '@/components/shared/Carousel.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useSongInteractions } from '@/composables/useSongInteractions'
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

  const { playInstantMix } = useSongInteractions(credentials)

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
        router.push(`/songs/album/${featuredAlbum.value.id}`)
      }
    } else {
      logger.warn('No songs found for featured album')
    }
  }

  const playAlbumSongs = (album: Album): void => {
    if (album.songs && album.songs.length > 0)
      emit('play-songs', sortSongsByTrackOrder(album.songs))
    else
      logger.warn('No songs found for album', album.name)
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='mb-8 hidden md:block'>
      <h1 class='text-4xl font-bold'>
        Home
      </h1>
    </div>

    <div class='space-y-8'>
      <!-- Featured Album Section -->
      <div
        v-if='featuredAlbum || isLoading'
        :style="{
          marginBottom: 'calc(-3rem - env(safe-area-inset-top) + 2rem)',
          minHeight: '400px',
          position: 'relative',
          top: 'calc(-3rem - env(safe-area-inset-top))'
        }"
        class='relative isolate bg-sidebar rounded-t-none md:rounded-lg p-4 md:p-8 mb-8 overflow-hidden
               -mx-4 md:mx-0 -mt-4 md:mt-0 md:mb-8'
      >
        <!-- Background Image (extends to top on mobile) -->
        <div class='absolute -top-4 md:top-0 left-0 right-0 bottom-0 bg-cover bg-center bg-no-repeat md:rounded-lg'>
          <ImageLoader
            v-if='featuredAlbum && !isLoading'
            :item-id='featuredAlbum.id || featuredAlbum.name'
            :server-url='serverUrl'
            :token='token'
            class='size-full object-cover'
          />
          <!-- Dark overlay with gradient fade -->
          <div class='absolute inset-0 bg-gradient-to-b from-black/70 via-black/60 to-background md:rounded-lg' />
        </div>

        <!-- Content -->
        <div
          class='absolute bottom-0 left-0 right-0 z-10 flex flex-col md:flex-row md:items-center
                 md:space-x-6 space-y-2 md:space-y-0 p-4 md:p-8'
        >
          <!-- Album Info - Left Side -->
          <div class='flex-1 min-w-0 text-left px-4 md:px-0'>
            <template v-if='isLoading'>
              <Skeleton class='h-12 md:h-14 w-3/4 mb-3' />
              <Skeleton class='h-8 md:h-9 w-1/2 mb-4' />
              <Skeleton class='h-6 md:h-6 w-1/4 mb-6' />
            </template>
            <template v-else-if='featuredAlbum'>
              <h1 class='text-4xl md:text-5xl font-bold mb-3 text-white drop-shadow-lg truncate'>
                <router-link
                  v-if='featuredAlbum.id'
                  :to="{ name: 'album-detail', params: { albumId: featuredAlbum.id } }"
                >
                  {{ featuredAlbum.name }}
                </router-link>
                <span v-else>{{ featuredAlbum.name }}</span>
              </h1>
              <p class='text-2xl md:text-2xl text-white/90 mb-3 drop-shadow-md'>
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
              <p class='text-lg md:text-lg text-white/80 mb-6 drop-shadow-md'>
                {{ featuredAlbum.songs?.length || 0 }} songs
              </p>
            </template>
          </div>

          <!-- Play Button - Right Side -->
          <div class='flex-shrink-0 px-4 md:px-0'>
            <template v-if='isLoading'>
              <button
                class='
                  bg-white/20 backdrop-blur-sm text-white px-6 md:px-8
                  py-3 md:py-3 rounded-full font-semibold border
                  border-white/20 opacity-50 cursor-not-allowed
                '
                disabled
              >
                <Play class='h-5 w-5 md:hidden' />
                <span class='hidden md:inline'>Play Album</span>
              </button>
            </template>
            <template v-else-if='featuredAlbum'>
              <button
                @click='playFeaturedAlbum'
                class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white px-6 md:px-8 py-3 md:py-3
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
          class='absolute bottom-4 right-4 z-20 flex space-x-2'
        >
          <button
            @click='prevFeaturedAlbum'
            :disabled='isLoading'
            class='flex items-center justify-center bg-white/20 p-3 md:p-2 text-white backdrop-blur-sm
                   transition-colors hover:bg-white/30 border border-white/20 rounded-full
                   disabled:opacity-50 disabled:cursor-not-allowed'
          >
            <ChevronLeft class='h-6 w-6 md:h-5 md:w-5' />
          </button>
          <button
            @click='nextFeaturedAlbum'
            :disabled='isLoading'
            class='flex items-center justify-center bg-white/20 p-3 md:p-2 text-white backdrop-blur-sm
                   transition-colors hover:bg-white/30 border border-white/20 rounded-full
                   disabled:opacity-50 disabled:cursor-not-allowed'
          >
            <ChevronRight class='h-6 w-6 md:h-5 md:w-5' />
          </button>
        </div>
      </div>

      <Carousel :disabled='isLoading' class='mb-8' title='Most Played'>
        <template v-if='isLoading || mostPlayed.length === 0'>
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
        <template v-for='song in mostPlayed' v-else :key='song.id'>
          <ContextMenu>
            <ContextMenuTrigger as-child>
              <div
                @click='playSongs(mostPlayed, song)'
                class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
              >
                <div class='relative mb-2'>
                  <ImageLoader
                    :item-id='song.albumId || song.id'
                    :server-url='serverUrl'
                    :token='token'
                    alt='Album art'
                    class='album-art-image'
                  >
                    <template #fallback>
                      <ImagePlaceholder class='album-art-image' size='large' type='album-art' />
                    </template>
                  </ImageLoader>

                  <!-- Play button overlay -->
                  <div
                    class='absolute inset-0 bg-black/50 rounded-lg opacity-0 group-hover:opacity-100
                           transition-opacity flex items-center justify-center'
                  >
                    <Button
                      @click.stop='playSongs(mostPlayed, song)'
                      class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20'
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
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playSongs([song])'>
                <Play class='size-4 mr-2' />Play
              </ContextMenuItem>
              <ContextMenuItem @click='playInstantMix(song)'>
                <Shuffle class='size-4 mr-2' />Instant Mix
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>
        </template>
      </Carousel>

      <Carousel :disabled='isLoading' class='mb-8' title='Recently Played'>
        <template v-if='isLoading || recentlyPlayed.length === 0'>
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
        <template v-for='song in recentlyPlayed' v-else :key='song.id'>
          <ContextMenu>
            <ContextMenuTrigger as-child>
              <div
                @click='playSongs(recentlyPlayed, song)'
                class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
              >
                <div class='relative mb-2'>
                  <ImageLoader
                    :item-id='song.albumId || song.id'
                    :server-url='serverUrl'
                    :token='token'
                    alt='Album art'
                    class='album-art-image'
                  >
                    <template #fallback>
                      <ImagePlaceholder class='album-art-image' size='large' type='album-art' />
                    </template>
                  </ImageLoader>

                  <!-- Play button overlay -->
                  <div
                    class='absolute inset-0 bg-black/50 rounded-lg opacity-0 group-hover:opacity-100
                           transition-opacity flex items-center justify-center'
                  >
                    <Button
                      @click.stop='playSongs(recentlyPlayed, song)'
                      class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20'
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
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playSongs([song])'>
                <Play class='size-4 mr-2' />Play
              </ContextMenuItem>
              <ContextMenuItem @click='playInstantMix(song)'>
                <Shuffle class='size-4 mr-2' />Instant Mix
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>
        </template>
      </Carousel>

      <Carousel :disabled='isLoading' class='mb-8' title='Recently Added'>
        <template v-if='isLoading || recentlyAdded.length === 0'>
          <div
            v-for='n in 10'
            :key='`recently-added-skeleton-${n}`'
            class='cursor-pointer group'
          >
            <div class='relative mb-2 album-stack-container'>
              <Skeleton class='album-stack-layer album-stack-layer-3 album-art-image' />
              <Skeleton class='album-stack-layer album-stack-layer-2 album-art-image' />
              <Skeleton class='album-stack-layer album-stack-layer-1 album-art-image' />
            </div>
            <Skeleton class='h-6 w-3/4 mb-1' />
            <Skeleton class='h-4 w-1/2' />
          </div>
        </template>
        <template v-else>
          <ContextMenu v-for='album in recentlyAdded' :key='album.name'>
            <ContextMenuTrigger as-child>
              <div
                @click="$emit('select-album', album)"
                class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
              >
                <div class='relative mb-2'>
                  <AlbumStack
                    @play='playAlbumSongs'
                    :album='album'
                    :disabled='isLoading'
                    :server-url='serverUrl'
                    :size="'responsive'"
                    :token='token'
                  />
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
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playAlbumSongs(album)'>
                <Play class='size-4 mr-2' />Play Album
              </ContextMenuItem>
              <AddToPlaylistMenu
                :songs='album.songs ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)) : []'
                type='context'
              />
            </ContextMenuContent>
          </ContextMenu>
        </template>
      </Carousel>

      <Carousel :disabled='isLoading' title='From Your Library'>
        <template v-if='isLoading || randomAlbums.length === 0'>
          <div
            v-for='n in 10'
            :key='`library-skeleton-${n}`'
            class='cursor-pointer group'
          >
            <div class='relative mb-2 album-stack-container'>
              <Skeleton class='album-stack-layer album-stack-layer-3 album-art-image' />
              <Skeleton class='album-stack-layer album-stack-layer-2 album-art-image' />
              <Skeleton class='album-stack-layer album-stack-layer-1 album-art-image' />
            </div>
            <Skeleton class='h-6 w-3/4 mb-1' />
            <Skeleton class='h-4 w-1/2' />
          </div>
        </template>
        <template v-else>
          <ContextMenu v-for='album in randomAlbums' :key='album.name'>
            <ContextMenuTrigger as-child>
              <div
                @click="$emit('select-album', album)"
                class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
              >
                <div class='relative mb-2'>
                  <AlbumStack
                    @play='playAlbumSongs'
                    :album='album'
                    :disabled='isLoading'
                    :server-url='serverUrl'
                    :size="'responsive'"
                    :token='token'
                  />
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
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playAlbumSongs(album)'>
                <Play class='size-4 mr-2' />Play Album
              </ContextMenuItem>
              <AddToPlaylistMenu
                :songs='album.songs ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)) : []'
                type='context'
              />
            </ContextMenuContent>
          </ContextMenu>
        </template>
      </Carousel>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
}

</style>
