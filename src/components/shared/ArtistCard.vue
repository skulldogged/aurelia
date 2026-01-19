<script setup lang="ts">
  import { Shuffle } from 'lucide-vue-next'

  import type { Artist } from '@/lib/api/bindings'

  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'

  interface Props {
    artist:      Artist
    compact?:    boolean
    isScrolling: boolean
    serverUrl:   string
    token:       string
    width?:      number
  }

  withDefaults(defineProps<Props>(), {
    compact: false,
    width:   400,
  })

  defineEmits<{
    click:   []
    shuffle: [artist: Artist]
  }>()
</script>

<template>
  <div
    @click="$emit('click')"
    class='artist-card cursor-pointer group'
  >
    <!-- Artist image (circular) -->
    <div class='artist-card-image relative' :class='compact ? "mb-2" : "mb-3"'>
      <div class='relative w-full aspect-square'>
        <ImageLoader
          :alt='`${artist.name} artist image`'
          :is-scrolling='isScrolling'
          :item-id='artist.id'
          :server-url='serverUrl'
          :token='token'
          :width='width'
          class='w-full aspect-square rounded-full object-cover shadow-lg'
        >
          <template #fallback>
            <ImagePlaceholder
              class='w-full aspect-square shadow-lg'
              size='large'
              type='artist'
            />
          </template>
        </ImageLoader>

        <!-- Subtle ring on hover -->
        <div
          class='
            absolute inset-0 rounded-full
            ring-2 ring-accent/0 group-hover:ring-accent/50
            transition-all duration-300
          '
        />
      </div>

      <!-- Hover overlay with shuffle button -->
      <div
        class='
          absolute inset-0 rounded-full bg-black/40 flex items-center justify-center
          opacity-0 group-hover:opacity-100 transition-all duration-200
        '
      >
        <Button
          @click.stop='$emit("shuffle", artist)'
          :size='compact ? "sm" : "icon"'
          class='
            bg-accent/90 hover:bg-accent text-accent-foreground
            shadow-lg hover:shadow-xl hover:scale-105
            transition-all duration-200 rounded-full
          '
        >
          <Shuffle :class='compact ? "size-3.5" : "size-4"' />
        </Button>
      </div>
    </div>

    <!-- Artist name -->
    <div class='text-center px-1'>
      <p
        :class='compact ? "text-xs" : "text-sm"'
        class='font-semibold truncate group-hover:text-accent transition-colors'
      >
        {{ artist.name }}
      </p>
      <p
        v-if='artist.songs?.length'
        :class='compact ? "text-[10px]" : "text-xs"'
        class='text-muted-foreground mt-0.5'
      >
        {{ artist.songs.length }} {{ artist.songs.length === 1 ? 'song' : 'songs' }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.artist-card-image {
  transition: transform 0.2s ease;
}

.artist-card:hover .artist-card-image {
  transform: translateY(-3px);
}
</style>
