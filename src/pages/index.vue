<script setup lang="ts">
  import type { Album, Song } from '@/bindings'

  import { useOrientation } from '@/composables/useOrientation'

  import HomePageDesktop from './HomePageDesktop.vue'
  import HomePageMobile from './HomePageMobile.vue'

  const { isPortrait } = useOrientation()

  defineProps<{
    currentSong: null | Song
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]],
    'select-album': [album: Album]
  }>()
</script>

<template>
  <component
    :is='isPortrait ? HomePageMobile : HomePageDesktop'
    @play-songs="(songs: Song[]) => emit('play-songs', songs)"
    @select-album="(album: Album) => emit('select-album', album)"
    :current-song='currentSong'
  />
</template>