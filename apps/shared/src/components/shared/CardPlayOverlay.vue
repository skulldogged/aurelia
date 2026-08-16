<script setup lang="ts">
  import { Play } from 'lucide-vue-next'

  import Button from '../ui/Button.vue'

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
      transition-opacity duration-200
    '
  >
    <Button
      @click.stop='$emit("play")'
      class='
        bg-white/30 hover:bg-white/40 backdrop-blur-sm
        text-white border border-white/40 shadow-lg
      '
      size='icon'
    >
      <Play class='size-5 fill-current' />
    </Button>
  </div>
</template>
