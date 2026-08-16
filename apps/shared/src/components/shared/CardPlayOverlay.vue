<script setup lang="ts">
  import { Play } from 'lucide-vue-next'

  defineEmits<{
    play: []
  }>()
</script>

<template>
  <div
    class='
      absolute inset-0 rounded-lg bg-black/40
      flex items-center justify-center
      opacity-0 pointer-events-none
      group-hover:opacity-100 group-hover:pointer-events-auto
    '
  >
    <button
      @click.stop='$emit("play")'
      aria-label='Play'
      class='
        inline-flex size-9 items-center justify-center rounded-button
        bg-white/40 hover:bg-white/50 text-white border border-white/40 shadow-lg
        focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/70
      '
    >
      <Play class='size-5 fill-current' />
    </button>
  </div>
</template>
