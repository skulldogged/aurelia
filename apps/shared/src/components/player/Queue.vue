<script setup lang="ts">
  import type { Song } from '../../lib/api/types'

  import VirtualizedQueue from './VirtualizedQueue.vue'

  const emit = defineEmits<{
    'remove-song':     [song: Song]
    'update:playlist': [playlist: Song[]]
  }>()
</script>

<template>
  <div class='w-64 lg:w-80 xl:w-96 2xl:w-md bg-background-dark flex flex-col h-full'>
    <div class='h-12 flex items-center px-4 shrink-0' data-drag-region>
      <h2 class='text-base font-semibold tracking-tight text-muted-foreground'>
        Up Next
      </h2>
    </div>
    <VirtualizedQueue
      @remove-song="emit('remove-song', $event)"
      @update:playlist="emit('update:playlist', $event)"
      class='grow min-h-0 px-2 py-3'
    />
  </div>
</template>
