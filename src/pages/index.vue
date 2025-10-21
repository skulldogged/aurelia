<script setup lang="ts">
  import type { Album, Song } from '@/bindings'

  import HomeView from '@/views/HomeView.vue'

  const props = defineProps<{
    currentSong: null | Song
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()
</script>

<template>
  <HomeView
    @play-songs='songs => emit("play-songs", songs)'
    @select-album='album => emit("select-album", album)'
    v-bind='$attrs'
    :current-song='props.currentSong'
  />
</template>
