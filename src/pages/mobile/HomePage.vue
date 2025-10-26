<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Play } from 'lucide-vue-next'

  import type { Album, Song } from '@/bindings'

  import AlbumGrid from '@/components/shared/AlbumGrid.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import Button from '@/components/ui/Button.vue'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useHomePage } from '@/composables/useHomePage'

  defineProps<{
    currentSong: null | Song
  }>()

  const emit = defineEmits<{
    (e: 'play-songs', songs: Song[]): void
    (e: 'select-album', album: Album): void
  }>()

  const {
    featuredAlbum,
    featuredAlbumArtistPairs,
    featuredAlbums,
    isLoading,
    mostPlayed,
    nextFeaturedAlbum,
    playFeaturedAlbum,
    playSongs,
    prevFeaturedAlbum,
    randomAlbums,
    recentlyAdded,
    recentlyPlayed,
    serverUrl,
    token,
  } = useHomePage(emit)
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
                <RouterLink
                  v-if='featuredAlbum.id'
                  :to='`/albums/${featuredAlbum.id}`'
                >
                  {{ featuredAlbum.name }}
                </RouterLink>
                <span v-else>{{ featuredAlbum.name }}</span>
              </h1>
              <p class='text-2xl md:text-2xl text-white/90 mb-3 drop-shadow-md'>
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
              <p class='text-lg md:text-lg text-white/80 mb-6 drop-shadow-md'>
                {{ featuredAlbum.songs?.length || 0 }} songs
              </p>
            </template>
          </div>

          <!-- Play Button - Mobile Portrait -->
          <div class='shrink-0'>
            <template v-if='isLoading'>
              <Button
                class='bg-white/20 backdrop-blur-sm text-white rounded-full font-semibold border border-white/20'
                size='icon-lg'
                disabled
              >
                <Play class='size-5' />
              </Button>
            </template>
            <template v-else-if='featuredAlbum'>
              <Button
                @click='playFeaturedAlbum'
                class='
                  bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white
                  rounded-full font-semibold transition-colors border border-white/20
                '
                size='icon-lg'
              >
                <Play class='size-5' />
              </Button>
            </template>
          </div>
        </div>

        <!-- Navigation Arrows -->
        <div
          v-if='featuredAlbums.length > 1'
          class='absolute z-20 flex space-x-2 bottom-4 right-4'
        >
          <Button
            @click='prevFeaturedAlbum'
            :disabled='isLoading'
            class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20 rounded-full'
            size='icon-lg'
            variant='ghost'
          >
            <ChevronLeft class='size-5' />
          </Button>
          <Button
            @click='nextFeaturedAlbum'
            :disabled='isLoading'
            class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20 rounded-full'
            size='icon-lg'
            variant='ghost'
          >
            <ChevronRight class='size-5' />
          </Button>
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