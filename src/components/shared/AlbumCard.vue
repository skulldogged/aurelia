<script setup lang="ts">
  import { Disc, Play } from 'lucide-vue-next'

  import type { Album } from '@/bindings'

  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'

  interface NameIdPair {
    id:   null | string
    name: string
  }

  interface Props {
    album:          Album
    collaborators?: NameIdPair[]
    serverUrl:      string
    showSongCount?: boolean
    token:          string
    width?:         number
  }

  withDefaults(defineProps<Props>(), {
    collaborators: () => [],
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
    <div class='relative mb-3 overflow-hidden rounded-lg'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        :width='width'
        class='album-art-image'
      >
        <template #fallback>
          <ImagePlaceholder class='album-art-image' size='large' type='album' />
        </template>
      </ImageLoader>

      <div
        class='absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100
         transition-opacity duration-200 flex items-center justify-center'
      >
        <Button
          @click.stop='$emit("play", album)'
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
    <div class='grid grid-cols-[1fr_auto] gap-2 items-start'>
      <div class='min-w-0'>
        <p class='font-semibold text-sm truncate group-hover:text-accent transition-colors'>
          {{ album.name }}
        </p>
        <p class='text-xs text-muted-foreground truncate mt-1'>
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
      <div v-if='showSongCount' class='flex items-center gap-1'>
        <span class='text-xs text-muted-foreground'>{{ album.songs?.length || 0 }}</span>
        <Disc class='h-3 w-3 text-muted-foreground' />
      </div>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
}
</style>
