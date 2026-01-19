<script setup lang="ts">
  import { Disc, Play } from 'lucide-vue-next'
  import { computed } from 'vue'

  import { Album } from '@/lib/api/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'

  interface Props {
    album:           Album
    disabled?:       boolean
    isScrolling?:    boolean
    quality?:        number
    serverUrl:       string
    showPlayButton?: boolean
    size?:           'fixed' | 'responsive'
    token:           string
    width?:          number
  }

  const props = withDefaults(defineProps<Props>(), {
    disabled:       false,
    isScrolling:    false,
    quality:        90,
    showPlayButton: true,
    size:           'fixed',
    width:          400,
  })

  defineEmits<{
    play: [album: Album]
  }>()

  // Memoize album identifier to prevent unnecessary re-renders
  const albumKey = computed(() => props.album.id || props.album.name)

  // shouldComponentUpdate optimization - only re-render if album actually changes
  const shouldUpdate = computed(() => albumKey.value)

  // Album-specific styling - different from song covers
  const songCount = computed(() => props.album.songs?.length || 0)
</script>

<template>
  <div
    :key='shouldUpdate'
    :class="[
      'relative album-card',
      !isScrolling && 'group',
      size === 'responsive' ? 'aspect-square w-full' : ''
    ]"
  >
    <!-- Album cover with distinctive styling -->
    <div class='album-cover-wrapper'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :is-scrolling='isScrolling'
        :item-id='albumKey'
        :quality='quality'
        :server-url='serverUrl'
        :token='token'
        :width='width'
        class='album-cover-image'
      >
        <template #fallback>
          <ImagePlaceholder
            class='album-cover-image'
            size='large'
            type='album'
          />
        </template>
      </ImageLoader>

      <!-- Album indicator overlay -->
      <div class='album-indicator'>
        <Disc class='album-icon' />
        <span class='song-count'>{{ songCount }} songs</span>
      </div>

      <!-- Play button overlay -->
      <div
        v-if='showPlayButton'
        class='play-button-overlay'
      >
        <Button
          @click.stop='$emit("play", album)'
          :disabled='disabled'
          class='play-button'
          size='icon'
        >
          <Play class='h-4 w-4' />
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
  @reference "tailwindcss";

  :deep(.album-cover-image) {
    @apply w-full h-auto rounded-xl shadow-lg aspect-square object-cover transition-all;
  }

  /* Album card styling - clean and distinctive */
  .album-card {
    @apply relative;
    /* Optimize for performance */
    transform: translateZ(0);
    contain: layout style paint;
  }

  .album-card:not(.aspect-square) {
    height: 11rem;
    width: 11rem;
  }

  .album-cover-wrapper {
    @apply relative overflow-hidden rounded-xl;
    background: linear-gradient(135deg, var(--muted) 0%, var(--muted-foreground/10) 100%);
    box-shadow:
      0 4px 6px -1px rgba(0, 0, 0, 0.1),
      0 2px 4px -1px rgba(0, 0, 0, 0.06);
    /* Ensure child overlays are properly clipped */
    isolation: isolate;
  }

  /* Album indicator - distinctive from songs */
  .album-indicator {
    @apply absolute bottom-0 left-0 right-0 p-3;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: white;
    /* Ensure proper coverage */
    border-radius: 0 0 0.75rem 0.75rem;
  }

  .album-icon {
    @apply h-4 w-4;
    opacity: 0.9;
  }

  .album-text {
    @apply text-xs font-medium;
    opacity: 0.95;
  }

  .song-count {
    @apply text-xs ml-auto;
    opacity: 0.8;
  }

  /* Play button overlay */
  .play-button-overlay {
    @apply absolute inset-0 flex items-center justify-center;
    background: rgba(0, 0, 0, 0.5);
    opacity: 0;
    transition: opacity 0.2s ease;
    backdrop-filter: blur(2px);
    /* Match the rounded corners of the album art */
    border-radius: 0.75rem;
  }

  .group:hover .play-button-overlay {
    opacity: 1;
  }

  .play-button {
    @apply bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border;
    border-color: rgba(255, 255, 255, 0.2);
    transition: all 0.2s ease;
  }

  .play-button:hover {
    @apply bg-white/40;
    transform: scale(1.05);
  }

  .play-button:disabled {
    @apply opacity-50 cursor-not-allowed;
    transform: none;
  }

  /* Performance optimizations */
  .album-cover-wrapper {
    /* Optimize for GPU acceleration */
    transform: translateZ(0);
    backface-visibility: hidden;
  }

  /* Reduce layout thrashing during hover */
  .album-card {
    contain: layout style paint;
  }
</style>