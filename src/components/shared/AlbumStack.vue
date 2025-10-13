<script setup lang="ts">
  import { Play } from 'lucide-vue-next'

  import { Album } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'

  interface Props {
    album:           Album
    disabled?:       boolean
    serverUrl:       string
    showPlayButton?: boolean
    size?:           'fixed' | 'responsive'
    token:           string
  }

  withDefaults(defineProps<Props>(), {
    disabled:       false,
    showPlayButton: true,
    size:           'fixed',
  })

  defineEmits<{
    play: [album: Album]
  }>()
</script>

<template>
  <div
    :class="[
      'relative album-stack-container group',
      size === 'responsive' ? 'aspect-square w-full' : ''
    ]"
  >
    <!-- Album stack effect - multiple layers -->
    <div class='album-stack-layer album-stack-layer-3'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        class='album-art-image'
      >
        <template #fallback>
          <ImagePlaceholder
            class='album-art-image'
            size='large'
            type='album'
          />
        </template>
      </ImageLoader>

      <!-- Darkening overlay -->
      <div
        class='
          absolute inset-0 bg-black/25 opacity-0
          group-hover:opacity-100
        '
        style='border-radius: 0.625rem;'
      />
    </div>
    <div class='album-stack-layer album-stack-layer-2'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        class='album-art-image'
      >
        <template #fallback>
          <ImagePlaceholder
            class='album-art-image'
            size='large'
            type='album'
          />
        </template>
      </ImageLoader>

      <!-- Darkening overlay -->
      <div
        class='
          absolute inset-0 bg-black/25 opacity-0
          group-hover:opacity-100
        '
        style='border-radius: 0.625rem;'
      />
    </div>
    <div class='album-stack-layer album-stack-layer-1'>
      <ImageLoader
        :alt='`${album.name} album art`'
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        class='album-art-image'
      >
        <template #fallback>
          <ImagePlaceholder
            class='album-art-image'
            size='large'
            type='album'
          />
        </template>
      </ImageLoader>

      <!-- Darkening overlay -->
      <div
        class='
          absolute inset-0 bg-black/25 opacity-0
          group-hover:opacity-100
        '
        style='border-radius: 0.625rem;'
      />
    </div>

    <!-- Play button overlay -->
    <div
      v-if='showPlayButton'
      class='
        absolute inset-0 rounded-lg opacity-0
        group-hover:opacity-100 transition-opacity
        flex items-center justify-center z-10
      '
    >
      <Button
        @click.stop='$emit("play", album)'
        :disabled='disabled'
        class='
          bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
          border-white/20 disabled:opacity-50 disabled:cursor-not-allowed
        '
        size='icon'
      >
        <Play class='h-4 w-4' />
      </Button>
    </div>
  </div>
</template>

<style scoped>
  @reference "tailwindcss";

  :deep(.album-art-image) {
    @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
  }

  /* Album stack effect */
  .album-stack-container {
    @apply relative;
  }

  .album-stack-container:not(.aspect-square) {
    height: 11rem;
    width: 11rem;
  }

  .group:hover .album-stack-layer {
    box-shadow:
      0 4px 12px rgba(0, 0, 0, 0.2),
      0 2px 4px rgba(0, 0, 0, 0.1);
    transition: box-shadow 0.2s ease;
  }

  .album-stack-layer {
    position: absolute;
    border-radius: 0.625rem;
    overflow: hidden;
    width: 100%;
    height: 100%;
    left: 0;
    top: 0;
    background-color: rgba(255, 255, 255, 0.01);
    box-shadow:
      0 4px 12px rgba(0, 0, 0, 0.2),
      0 2px 4px rgba(0, 0, 0, 0.1),
      inset 0 0 0 1px rgba(255, 255, 255, 0.1);
  }

  .album-stack-layer-1 {
    transform: rotate(-4deg) translateX(-4px) translateY(6px) scale(0.95);
    z-index: 1;
    opacity: 0.8;
  }

  .album-stack-layer-2 {
    transform: rotate(4deg) translateX(2px) translateY(2px) scale(0.97);
    z-index: 2;
    opacity: 0.9;
  }

  .album-stack-layer-3 {
    transform: rotate(0deg) scale(1);
    z-index: 3;
    opacity: 1;
  }

  /* Remove transition from stack layer overlays for static effect */
  .album-stack-layer > div:last-child {
    transition: none;
    border-radius: 0.625rem;
    border: 1px solid transparent !important;
  }
</style>