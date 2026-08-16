<script setup lang="ts">
  import type { Song } from '../../lib/api/types'

  import VirtualizedQueue from './VirtualizedQueue.vue'

  const emit = defineEmits<{
    'remove-song':     [song: Song]
    'update:playlist': [playlist: Song[]]
  }>()
</script>

<template>
  <VirtualizedQueue
    @remove-song="emit('remove-song', $event)"
    @update:playlist="emit('update:playlist', $event)"
    class='h-full w-full p-4'
  />
</template>
