<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Play, Shuffle } from 'lucide-vue-next'
  import { onMounted, onUnmounted } from 'vue'

  import type { Album, Song } from '@/bindings'

  import HomePageTopBar from '@/components/desktop/HomePageTopBar.vue'
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
  import { useHomePage } from '@/composables/useHomePage'
  import { useTopBar } from '@/composables/useTopBar'

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
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='space-y-8'>
      <!-- Featured Album Section -->
      <div
        v-if='featuredAlbum || isLoading'
        class='relative isolate bg-sidebar mb-8 overflow-hidden rounded-lg p-8'
      >
        <!-- Background Image -->
        <div
          class='absolute bg-cover bg-center bg-no-repeat inset-0 rounded-lg blur-md scale-105 overflow-hidden'
        >
          <ImageLoader
            v-if='featuredAlbum && !isLoading'
            :item-id='featuredAlbum.id || featuredAlbum.name'
            :server-url='serverUrl'
            :token='token'
            class='size-full object-cover'
          />
          <div
            class='absolute inset-0 bg-black/60 rounded-lg'
          />
        </div>

        <!-- Content -->
        <div
          class='z-10 relative flex items-start space-x-6'
        >
          <!-- Album Art - Desktop -->
          <div
            class='shrink-0'
          >
            <template v-if='isLoading'>
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

          <!-- Text and Button Container - Desktop -->
          <div
            class='flex-1 min-w-0 flex flex-col space-y-4'
          >
            <template v-if='isLoading'>
              <Skeleton class='h-10 w-3/4 mb-2' />
              <Skeleton class='h-7 w-1/2 mb-4' />
              <Skeleton class='h-5 w-1/4 mb-6' />
            </template>
            <template v-else-if='featuredAlbum'>
              <h1 class='text-4xl font-bold mb-2 text-white drop-shadow-lg truncate'>
                <RouterLink
                  v-if='featuredAlbum.id'
                  :to='`/albums/${featuredAlbum.id}`'
                >
                  {{ featuredAlbum.name }}
                </RouterLink>
                <span v-else>{{ featuredAlbum.name }}</span>
              </h1>
              <p class='text-xl text-white/90 mb-3 drop-shadow-md'>
                <template v-if='featuredAlbumArtistPairs.length'>
                  <template v-for='(pair, index) in featuredAlbumArtistPairs' :key='pair.id'>
                    <RouterLink
                      :to='`/artists/${pair.id}`'
                      class='hover:underline'
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
                    class='hover:underline'
                  >
                    {{ featuredAlbum.artist }}
                  </RouterLink>
                  <span v-else>{{ featuredAlbum.artist }}</span>
                </template>
              </p>
              <p class='text-sm text-white/80 mb-6 drop-shadow-md'>
                {{ featuredAlbum.songs?.length || 0 }} songs
              </p>
            </template>

            <!-- Play Button - Desktop -->
            <div class='shrink-0'>
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

      <Carousel :disabled='isLoading' class='mb-8' title='Recently Added'>
        <template v-if='isLoading || recentlyAdded.length === 0'>
          <div
            v-for='n in 10'
            :key='`recently-added-skeleton-${n}`'
            class='cursor-pointer group'
          >
            <div class='relative mb-2 album-card'>
              <Skeleton class='album-cover-wrapper' />
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
                  <RouterLink
                    @click.stop
                    v-if='album.artistId'
                    :to='`/artists/${album.artistId}`'
                    class='hover:underline'
                  >
                    {{ album.artist }}
                  </RouterLink>
                  <span v-else>{{ album.artist }}</span>
                </p>
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playAlbumSongs(album)'>
                <Play class='size-4 mr-2' />Play Album
              </ContextMenuItem>
              <AddToPlaylistMenu
                :songs='
                  album.songs
                    ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
                    : []
                '
                type='context'
              />
            </ContextMenuContent>
          </ContextMenu>

          <!-- Load More Button -->
          <div
            @click='loadMoreData'
            v-if='hasMoreData.recentlyAdded && loadingStage !== "full"'
            class='
              cursor-pointer group hover:bg-muted/50 rounded-md
              transition-colors p-2 flex flex-col items-center
              justify-center min-h-44
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

      <Carousel :disabled='isLoading' title='From Your Library'>
        <template v-if='isLoading || randomAlbums.length === 0'>
          <div
            v-for='n in 10'
            :key='`library-skeleton-${n}`'
            class='cursor-pointer group'
          >
            <div class='relative mb-2 album-card'>
              <Skeleton class='album-cover-wrapper' />
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
                  <RouterLink
                    @click.stop
                    v-if='album.artistId'
                    :to='`/artists/${album.artistId}`'
                    class='hover:underline'
                  >
                    {{ album.artist }}
                  </RouterLink>
                  <span v-else>{{ album.artist }}</span>
                </p>
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playAlbumSongs(album)'>
                <Play class='size-4 mr-2' />Play Album
              </ContextMenuItem>
              <AddToPlaylistMenu
                :songs='
                  album.songs
                    ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
                    : []
                '
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

:deep(.album-cover-image) {
  @apply w-full h-auto rounded-xl shadow-lg aspect-square object-cover transition-all;
}

:deep(.album-cover-wrapper) {
  @apply w-full h-auto rounded-xl aspect-square;
}

</style>