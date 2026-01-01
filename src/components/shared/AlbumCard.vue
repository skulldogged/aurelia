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
    <!-- Album art with overlay -->
    <div class='relative mb-3'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        :width='width'
        class='w-full aspect-square rounded-lg group-hover:opacity-75 transition-opacity'
      >
        <template #fallback>
          <ImagePlaceholder class='w-full aspect-square rounded-lg' size='large' type='album' />
        </template>
      </ImageLoader>

      <!-- Hover overlay with play button -->
      <div
        class='absolute inset-0 bg-black/50 rounded-lg flex items-center justify-center
               opacity-0 group-hover:opacity-100 transition-opacity'
      >
        <Button
          @click.stop='$emit("play", album)'
          class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20'
          size='icon'
        >
          <Play class='size-5 fill-current' />
        </Button>
      </div>
    </div>

    <!-- Album info -->
    <div class='flex items-start gap-2'>
      <div class='min-w-0 flex-1'>
        <p class='font-semibold text-sm truncate group-hover:text-accent transition-colors'>
          {{ album.name }}
        </p>
        <p class='text-xs text-muted-foreground truncate mt-0.5'>
          <template v-if='collaborators && collaborators.length > 0'>
            with
            <template v-for='(pair, idx) in collaborators' :key='pair.id || pair.name'>
              <RouterLink
                @click.stop
                v-if='pair.id'
                :to='`/artists/${pair.id}`'
                class='hover:underline'
              >{{ pair.name }}</RouterLink>
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
            >{{ album.artist }}</RouterLink>
            <span v-else>{{ album.artist }}</span>
          </template>
        </p>
      </div>
      <div v-if='showSongCount' class='flex items-center gap-1 text-xs text-muted-foreground'>
        <span>{{ album.songs?.length || 0 }}</span>
        <Disc class='size-3' />
      </div>
    </div>
  </div>
</template>
