<script setup lang="ts">
  import { Disc } from 'lucide-vue-next'

  import type { Album } from '../../lib/api/types'

  import CardPlayOverlay from './CardPlayOverlay.vue'
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
    isScrolling?:   boolean
    serverUrl:      string
    showSongCount?: boolean
    token:          string
    width?:         number
  }

  withDefaults(defineProps<Props>(), {
    collaborators: () => [],
    compact:       false,
    isScrolling:   false,
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
    class='cursor-pointer group'
  >
    <div :class='compact ? "mb-2" : "mb-3"' class='relative'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :is-scrolling='isScrolling'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        :width='width'
        class='w-full aspect-square rounded-lg shadow-md object-cover'
      >
        <template #fallback>
          <ImagePlaceholder class='w-full aspect-square rounded-lg shadow-md' size='large' type='album' />
        </template>
      </ImageLoader>

      <CardPlayOverlay @play='$emit("play", album)' />
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
        <span class='leading-none'>{{ album.songs?.length ?? album.songCount ?? 0 }}</span>
        <Disc :class='compact ? "size-2.5" : "size-3"' />
      </div>
    </div>
  </div>
</template>
