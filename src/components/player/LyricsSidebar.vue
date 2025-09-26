<script setup lang="ts">
  import { Mic2 } from 'lucide-vue-next'

  import { Song } from '@/bindings'
  import LyricsView from '@/components/shared/LyricsView.vue'

  defineProps<{
    currentSong: null | Song
    currentTime: number
    duration:    number
  }>()

  defineEmits<{
    (e: 'seek', time: number): void
  }>()
</script>

<template>
  <div class='w-64 lg:w-80 xl:w-96 2xl:w-[28rem] flex flex-col bg-background h-full pt-12'>
    <div class='flex items-center justify-center py-4 border-b border-border/50'>
      <div class='flex items-center gap-2'>
        <Mic2 class='w-5 h-5 text-muted-foreground' />
        <h3 class='text-lg font-semibold'>
          Lyrics
        </h3>
      </div>
    </div>

    <div class='flex-1 overflow-hidden'>
      <LyricsView
        @seek='$emit("seek", ($event / duration) * 100)'
        :current-time='currentTime'
        :duration='duration'
        :song='currentSong'
        :visible='true'
      />
    </div>
  </div>
</template>
