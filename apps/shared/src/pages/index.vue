<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Play, Shuffle } from 'lucide-vue-next'
  import { computed, onMounted, onUnmounted, ref } from 'vue'

  import type { Album, Song } from '../lib/api/types'

  import HomePageTopBar from '../components/layout/desktop/HomePageTopBar.vue'
  import AddToPlaylistMenu from '../components/shared/AddToPlaylistMenu.vue'
  import AlbumCard from '../components/shared/AlbumCard.vue'
  import Carousel from '../components/shared/Carousel.vue'
  import ImageLoader from '../components/shared/ImageLoader.vue'
  import ImagePlaceholder from '../components/shared/ImagePlaceholder.vue'
  import Button from '../components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '../components/ui/context-menu'
  import { Skeleton } from '../components/ui/skeleton'
  import { useHomePage } from '../composables/useHomePage'
  import { useTopBar } from '../composables/useTopBar'

  defineProps<{
    currentSong: null | Song
  }>()

  const emit = defineEmits<{
    (e: 'play-songs', songs: Song[]): void
    (e: 'select-album', album: Album): void
  }>()

  // Use top bar for title display
  const { clearTopBarContent, setTopBarContent } = useTopBar()

  const {
    featuredAlbum,
    featuredAlbumArtistPairs,
    featuredAlbums,
    hasMoreData,
    isLoading,
    loadingStage,
    loadMoreData,
    mostPlayed,
    nextFeaturedAlbum,
    playAlbumSongs,
    playFeaturedAlbum,
    playInstantMix,
    playSongs,
    prevFeaturedAlbum,
    randomAlbums,
    recentlyAdded,
    recentlyPlayed,
    serverUrl,
    token,
  } = useHomePage(emit)

  // Local state for featured album index
  const currentFeaturedIndex = ref(0)

  // Compute current featured album index
  const computedFeaturedIndex = computed(() => {
    if (!featuredAlbum.value || featuredAlbums.value.length === 0) return 0
    return Math.max(
      0,
      featuredAlbums.value.findIndex(a => a.id === featuredAlbum.value?.id),
    )
  })

  // Helper to format song count - show "Single" for albums with only 1 song
  const formatSongCount = (album: Album | null): string => {
    if (!album) return '0 songs'
    const count = album.songs?.length ?? Number(album.songCount || 0)
    if (count === 1) return 'Single'
    return `${count} songs`
  }

  // Memoize sorted album songs to avoid recalculation on every render
  const getSortedAlbumSongs = (album: Album): Song[] => {
    if (!album.songs) return []
    return [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
  }

  // Set up top bar content when component mounts
  onMounted(() => {
    setTopBarContent({
      component: HomePageTopBar,
      id:        'home-page',
    })
  })

  // Clean up top bar content when component unmounts
  onUnmounted(() => {
    clearTopBarContent()
  })
</script>

<template>
  <div class='flex flex-col'>
    <section
      v-if='featuredAlbum || isLoading'
      class='
        relative isolate overflow-hidden min-h-[400px]
        bg-linear-to-b from-sidebar via-sidebar to-background
      '
    >
      <div class='absolute inset-0 overflow-hidden'>
        <div class='absolute inset-0 opacity-20'>
          <ImageLoader
            v-if='featuredAlbum && !isLoading'
            :item-id='featuredAlbum.id || featuredAlbum.name'
            :server-url='serverUrl'
            :token='token'
            :width='600'
            class='size-full object-cover blur-2xl scale-110'
          />
        </div>

        <div
          class='
            absolute bottom-0 left-0 right-0 h-40 pointer-events-none
            bg-linear-to-t from-background to-transparent
          '
        />
      </div>

      <div class='relative z-10 flex flex-col items-center py-12 px-6 md:px-10 lg:px-16'>
        <div class='flex items-center justify-between w-full gap-8 lg:gap-12 max-w-7xl mb-8'>
          <div class='flex-1 min-w-0 space-y-6'>
            <template v-if='isLoading'>
              <Skeleton class='h-12 w-3/4 rounded-lg' />
              <Skeleton class='h-8 w-1/2 rounded-lg' />
              <Skeleton class='h-5 w-2/3 rounded-lg' />
              <div class='flex gap-3 pt-2'>
                <Skeleton class='h-10 w-32 rounded-lg' />
                <Skeleton class='h-10 w-32 rounded-lg' />
              </div>
            </template>
            <template v-else-if='featuredAlbum'>
              <div class='flex-1 min-w-0 flex flex-col gap-6'>
                <span class='inline-block px-3 py-1 bg-accent/20 text-accent text-xs font-semibold rounded-full w-fit'>
                  Featured Album
                </span>

                <div>
                  <h1 class='text-5xl lg:text-6xl font-black text-white truncate max-w-2xl'>
                    <RouterLink
                      v-if='featuredAlbum.id'
                      :to='`/albums/${featuredAlbum.id}`'
                      class='hover:text-accent transition-colors duration-200'
                    >
                      {{ featuredAlbum.name }}
                    </RouterLink>
                    <span v-else class='hover:text-accent/80 transition-colors duration-200 cursor-default'>
                      {{ featuredAlbum.name }}
                    </span>
                  </h1>

                  <div class='flex flex-wrap gap-4 mt-2'>
                    <div class='flex items-center gap-2'>
                      <span class='text-sm text-white/90 font-semibold'>
                        <template v-if='featuredAlbumArtistPairs.length'>
                          <template v-for='(pair, index) in featuredAlbumArtistPairs' :key='pair.id'>
                            <RouterLink
                              :to='`/artists/${pair.id}`'
                              class='hover:text-accent transition-colors'
                            >
                              {{ pair.name }}
                            </RouterLink>
                            <span v-if='index < featuredAlbumArtistPairs.length - 1'>, </span>
                          </template>
                        </template>
                        <template v-else>
                          <RouterLink
                            v-if='featuredAlbum.artistId'
                            :to='`/artists/${featuredAlbum.artistId}`'
                            class='hover:text-accent transition-colors'
                          >
                            {{ featuredAlbum.artist }}
                          </RouterLink>
                          <span v-else>{{ featuredAlbum.artist }}</span>
                        </template>
                      </span>
                    </div>
                    <div class='w-px bg-white/10' />
                    <div class='flex items-center gap-2'>
                      <span class='text-sm text-white/70'>{{ formatSongCount(featuredAlbum) }}</span>
                    </div>
                  </div>
                </div>

                <div class='flex items-center gap-3'>
                  <button
                    @click='playFeaturedAlbum'
                    class='
                      px-6 py-3 bg-accent hover:bg-accent/90 text-sidebar font-bold rounded-lg
                      transition-all duration-200 flex items-center gap-2 shadow-lg
                      hover:shadow-xl
                    '
                  >
                    <Play class='h-5 w-5 fill-current' />
                    <span>Play</span>
                  </button>
                  <button
                    @click='playFeaturedAlbum'
                    class='
                      px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-semibold rounded-lg
                      border border-white/20 transition-all duration-200 flex items-center gap-2
                      backdrop-blur-sm hover:backdrop-blur-md
                    '
                  >
                    <Shuffle class='h-5 w-5' />
                    <span>Shuffle</span>
                  </button>
                </div>
              </div>
            </template>
          </div>

          <div class='hidden lg:flex shrink-0 items-center justify-end'>
            <template v-if='isLoading'>
              <Skeleton class='w-64 h-64 rounded-2xl' />
            </template>
            <template v-else-if='featuredAlbum'>
              <div class='relative group'>
                <div
                  class='
                    absolute -inset-4 rounded-3xl blur-xl opacity-0
                    group-hover:opacity-100 transition-opacity duration-300
                    bg-linear-to-br from-accent/30 to-accent/10
                  '
                />

                <ImageLoader
                  :alt='`${featuredAlbum.name} album art`'
                  :item-id='featuredAlbum.id || featuredAlbum.name'
                  :server-url='serverUrl'
                  :token='token'
                  :width='400'
                  class='
                    relative w-64 h-64 rounded-2xl shadow-2xl object-cover
                    transition-shadow duration-300 group-hover:shadow-2xl
                  '
                >
                  <template #fallback>
                    <ImagePlaceholder
                      class='w-64 h-64 rounded-2xl shadow-2xl'
                      size='large'
                      type='album'
                    />
                  </template>
                </ImageLoader>
              </div>
            </template>
          </div>
        </div>

        <div
          v-if='featuredAlbums.length > 1'
          class='flex w-full items-center justify-between max-w-7xl'
        >
          <div class='flex gap-2'>
            <template v-for='(_, idx) in featuredAlbums' :key='idx'>
              <button
                @click='() => currentFeaturedIndex = idx'
                :class='[
                  "h-2 rounded-full transition-all duration-300",
                  idx === computedFeaturedIndex
                    ? "w-8 bg-accent"
                    : "w-2 bg-white/40 hover:bg-white/60"
                ]'
              />
            </template>
          </div>

          <div class='flex gap-2'>
            <button
              @click='prevFeaturedAlbum'
              :disabled='isLoading'
              class='
                flex items-center justify-center bg-white/10 p-2 text-white backdrop-blur-sm transition-all
                hover:bg-white/20 border border-white/20 rounded-full
                disabled:opacity-50 disabled:cursor-not-allowed group
              '
            >
              <ChevronLeft class='h-4 w-4 transition-transform group-hover:-translate-x-0.5' />
            </button>
            <button
              @click='nextFeaturedAlbum'
              :disabled='isLoading'
              class='
                flex items-center justify-center bg-white/10 p-2 text-white backdrop-blur-sm transition-all
                hover:bg-white/20 border border-white/20 rounded-full
                disabled:opacity-50 disabled:cursor-not-allowed group
              '
            >
              <ChevronRight class='h-4 w-4 transition-transform group-hover:translate-x-0.5' />
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class='flex flex-col px-6 md:px-10 lg:px-16'>
      <section class='flex justify-center'>
        <div class='w-full max-w-7xl'>
          <Carousel :disabled='isLoading' title='Most Played'>
            <template v-if='isLoading || mostPlayed.length === 0'>
              <div
                v-for='n in 10'
                :key='`most-played-skeleton-${n}`'
                class='cursor-pointer group'
              >
                <div class='relative mb-3'>
                  <Skeleton class='album-art-image' />
                </div>
                <Skeleton class='h-5 w-4/5 mb-2' />
                <Skeleton class='h-4 w-3/4' />
              </div>
            </template>
            <template v-for='song in mostPlayed' v-else :key='song.id'>
              <ContextMenu>
                <ContextMenuTrigger as-child>
                  <div
                    @click='playSongs(mostPlayed, song)'
                    class='cursor-pointer group'
                  >
                    <div class='relative mb-3 overflow-hidden rounded-lg'>
                      <ImageLoader
                        :item-id='song.albumId || song.id'
                        :server-url='serverUrl'
                        :token='token'
                        :width='400'
                        alt='Album art'
                        class='album-art-image'
                      >
                        <template #fallback>
                          <ImagePlaceholder class='album-art-image' size='large' type='album-art' />
                        </template>
                      </ImageLoader>

                      <div
                        class='absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100
                         transition-opacity duration-200 flex items-center justify-center'
                      >
                        <Button
                          @click.stop='playSongs(mostPlayed, song)'
                          class='
                            bg-white/30 hover:bg-white/40 backdrop-blur-sm
                            text-white border border-white/40 shadow-lg
                          '
                          size='icon'
                        >
                          <Play class='h-5 w-5 fill-current' />
                        </Button>
                      </div>
                    </div>
                    <p class='font-semibold text-sm truncate group-hover:text-accent transition-colors'>
                      {{ song.name }}
                    </p>
                    <p class='text-xs text-muted-foreground truncate mt-1'>
                      <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
                        <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                          <RouterLink
                            @click.stop
                            :to='`/artists/${song.artistIds[index]}`'
                            class='hover:underline'
                          >
                            {{ artist }}
                          </RouterLink>
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
        </div>
      </section>

      <section class='flex justify-center'>
        <div class='w-full max-w-7xl'>
          <Carousel :disabled='isLoading' title='Recently Played'>
            <template v-if='isLoading || recentlyPlayed.length === 0'>
              <div
                v-for='n in 10'
                :key='`recently-played-skeleton-${n}`'
                class='cursor-pointer group'
              >
                <div class='relative mb-3'>
                  <Skeleton class='album-art-image' />
                </div>
                <Skeleton class='h-5 w-4/5 mb-2' />
                <Skeleton class='h-4 w-3/4' />
              </div>
            </template>
            <template v-for='song in recentlyPlayed' v-else :key='song.id'>
              <ContextMenu>
                <ContextMenuTrigger as-child>
                  <div
                    @click='playSongs(recentlyPlayed, song)'
                    class='cursor-pointer group'
                  >
                    <div class='relative mb-3 overflow-hidden rounded-lg'>
                      <ImageLoader
                        :item-id='song.albumId || song.id'
                        :server-url='serverUrl'
                        :token='token'
                        :width='400'
                        alt='Album art'
                        class='album-art-image'
                      >
                        <template #fallback>
                          <ImagePlaceholder class='album-art-image' size='large' type='album-art' />
                        </template>
                      </ImageLoader>

                      <div
                        class='absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100
                         transition-opacity duration-200 flex items-center justify-center'
                      >
                        <Button
                          @click.stop='playSongs(recentlyPlayed, song)'
                          class='
                            bg-white/30 hover:bg-white/40 backdrop-blur-sm
                            text-white border border-white/40 shadow-lg
                          '
                          size='icon'
                        >
                          <Play class='h-5 w-5 fill-current' />
                        </Button>
                      </div>
                    </div>
                    <p class='font-semibold text-sm truncate group-hover:text-accent transition-colors'>
                      {{ song.name }}
                    </p>
                    <p class='text-xs text-muted-foreground truncate mt-1'>
                      <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
                        <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                          <RouterLink
                            @click.stop
                            :to='`/artists/${song.artistIds[index]}`'
                            class='hover:underline'
                          >
                            {{ artist }}
                          </RouterLink>
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
        </div>
      </section>

      <section class='flex justify-center'>
        <div class='w-full max-w-7xl'>
          <Carousel :disabled='isLoading' title='Recently Added'>
            <template v-if='isLoading || recentlyAdded.length === 0'>
              <div
                v-for='n in 10'
                :key='`recently-added-skeleton-${n}`'
                class='cursor-pointer group'
              >
                <div class='relative mb-3'>
                  <Skeleton class='album-art-image' />
                </div>
                <Skeleton class='h-5 w-4/5 mb-2' />
                <Skeleton class='h-4 w-3/4' />
              </div>
            </template>
            <template v-else>
              <ContextMenu v-for='album in recentlyAdded' :key='album.id || album.name'>
                <ContextMenuTrigger as-child>
                  <AlbumCard
                    @click="$emit('select-album', album)"
                    @play='playAlbumSongs'
                    :album='album'
                    :server-url='serverUrl'
                    :token='token'
                  />
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem @click='playAlbumSongs(album)'>
                    <Play class='size-4 mr-2' />Play Album
                  </ContextMenuItem>
                  <AddToPlaylistMenu
                    :songs='getSortedAlbumSongs(album)'
                    type='context'
                  />
                </ContextMenuContent>
              </ContextMenu>

              <div
                @click='loadMoreData'
                v-if='hasMoreData.recentlyAdded && loadingStage !== "full"'
                class='
            flex flex-col items-center justify-center min-h-48 rounded-lg border
            border-dashedborder-muted-foreground/30 group cursor-pointer
            hover:border-muted-foreground/50 transition-colors
          '
              >
                <div class='text-center'>
                  <Button
                    :disabled='isLoading'
                    class='mb-2'
                    variant='outline'
                  >
                    Load More
                  </Button>
                  <p class='text-sm text-muted-foreground'>
                    {{ recentlyAdded.length }} albums loaded
                  </p>
                </div>
              </div>
            </template>
          </Carousel>
        </div>
      </section>

      <section class='flex justify-center'>
        <div class='w-full max-w-7xl'>
          <Carousel :disabled='isLoading' title='From Your Library'>
            <template v-if='isLoading || randomAlbums.length === 0'>
              <div
                v-for='n in 10'
                :key='`library-skeleton-${n}`'
                class='cursor-pointer group'
              >
                <div class='relative mb-3'>
                  <Skeleton class='album-art-image' />
                </div>
                <Skeleton class='h-5 w-4/5 mb-2' />
                <Skeleton class='h-4 w-3/4' />
              </div>
            </template>
            <template v-else>
              <ContextMenu v-for='album in randomAlbums' :key='album.id || album.name'>
                <ContextMenuTrigger as-child>
                  <AlbumCard
                    @click="$emit('select-album', album)"
                    @play='playAlbumSongs'
                    :album='album'
                    :server-url='serverUrl'
                    :token='token'
                  />
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem @click='playAlbumSongs(album)'>
                    <Play class='size-4 mr-2' />Play Album
                  </ContextMenuItem>
                  <AddToPlaylistMenu
                    :songs='getSortedAlbumSongs(album)'
                    type='context'
                  />
                </ContextMenuContent>
              </ContextMenu>
            </template>
          </Carousel>
        </div>
      </section>
    </section>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
}

:deep(.album-cover-image) {
  @apply w-full h-auto rounded-xl shadow-lg aspect-square object-cover transition-all;
}

:deep(.album-cover-wrapper) {
  @apply w-full h-auto rounded-xl aspect-square;
}
</style>