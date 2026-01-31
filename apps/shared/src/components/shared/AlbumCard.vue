<script setup lang="ts">
  import { Disc, Play } from 'lucide-vue-next'

  import type { Album } from '../../lib/api/types'

  import Button from '../ui/Button.vue'
  import ImageLoader from './ImageLoader.vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'

  interface NameIdPair {
    id:   null | string
    name: string
  }

  interface Props {
    album:          Album
    collaborators?: NameIdPair[]
    compact?:       boolean
    serverUrl:      string
    showSongCount?: boolean
    token:          string
    width?:         number
  }

  withDefaults(defineProps<Props>(), {
    collaborators: () => [],
    compact:       false,
    showSongCount: true,
    width:         400,
  })

  defineEmits<{
    click: []
    play:  [album: Album]
  }>()
</script>

<template>
  <div
    @click="$emit('click')"
    class='album-card cursor-pointer group'
  >
    <!-- Album art with overlay -->
    <div :class='compact ? "mb-2" : "mb-3"' class='album-card-image relative'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        :width='width'
        class='w-full aspect-square rounded-lg shadow-md'
      >
        <template #fallback>
          <ImagePlaceholder class='w-full aspect-square rounded-lg shadow-md' size='large' type='album' />
        </template>
      </ImageLoader>

      <!-- Hover overlay with play button -->
      <div
        class='
          absolute inset-0 bg-black/40 rounded-lg flex items-center justify-center
          opacity-0 group-hover:opacity-100 transition-all duration-200
        '
      >
        <Button
          @click.stop='$emit("play", album)'
          class='
            bg-accent/90 hover:bg-accent text-accent-foreground
            shadow-lg hover:shadow-xl hover:scale-105
            transition-all duration-200
          '
          size='icon'
        >
          <Play class='size-5 fill-current' />
        </Button>
      </div>
    </div>

    <!-- Album info -->
    <div class='flex items-start gap-2'>
      <div class='min-w-0 flex-1'>
        <p
          :class='compact ? "text-xs" : "text-sm"'
          class='font-semibold truncate group-hover:text-accent transition-colors'
        >
          {{ album.name }}
        </p>
        <p :class='compact ? "text-[11px]" : "text-xs"' class='text-muted-foreground truncate mt-0.5'>
          <template v-if='collaborators && collaborators.length > 0'>
            with
            <template v-for='(pair, idx) in collaborators' :key='pair.id || pair.name'>
              <RouterLink
                @click.stop
                v-if='pair.id'
                :to='`/artists/${pair.id}`'
                class='hover:underline'
              >
                {{ pair.name }}
              </RouterLink>
              <span v-else>{{ pair.name }}</span>
              <span v-if='idx < collaborators.length - 1'>, </span>
            </template>
          </template>
          <template v-else>
            <RouterLink
              @click.stop
              v-if='album.artistId'
              :to='`/artists/${album.artistId}`'
              class='hover:underline'
            >
              {{ album.artist }}
            </RouterLink>
            <span v-else>{{ album.artist }}</span>
          </template>
        </p>
      </div>
      <div
        v-if='showSongCount'
        :class='compact ? "text-[10px]" : "text-xs"'
        class='flex items-center gap-1 text-muted-foreground shrink-0 mt-0.5'
      >
        <span class='leading-none'>{{ album.songs?.length || 0 }}</span>
        <Disc :class='compact ? "size-2.5" : "size-3"' />
      </div>
    </div>
  </div>
</template>

<style scoped>
.album-card-image {
  transition: transform 0.2s ease, filter 0.2s ease;
}

.album-card:hover .album-card-image {
  transform: translateY(-3px);
  filter: drop-shadow(0 8px 16px rgba(0, 0, 0, 0.25));
}
</style>
