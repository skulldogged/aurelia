<script setup lang="ts">
  import { Song } from '@/bindings'
  import LyricsView from '@/components/shared/LyricsView.vue'

  defineProps<{
    currentSong: null | Song
    currentTime: number
    duration:    number
  }>()

  const emit = defineEmits<{
    (e: 'lyrics-loaded', hasLyrics: boolean): void
    (e: 'seek', time: number): void
  }>()

  const handleLyricsLoaded = (hasLyrics: boolean): void => {
    emit('lyrics-loaded', hasLyrics)
  }
</script>

<template>
  <div class='w-64 lg:w-80 xl:w-96 2xl:w-md flex flex-col bg-background-dark h-full'>
    <div class='flex-1 overflow-hidden'>
      <LyricsView
        @lyrics-loaded='handleLyricsLoaded'
        @seek='$emit("seek", ($event / duration) * 100)'
        :current-time='currentTime'
        :duration='duration'
        :is-in-sidebar='true'
        :song='currentSong'
        :visible='true'
      />
    </div>
  </div>
</template>
