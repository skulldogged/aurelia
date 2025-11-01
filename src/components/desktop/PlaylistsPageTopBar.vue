<script setup lang="ts">
  import DraggableArea from '@/components/ui/DraggableArea.vue'
</script>

<template>
  <div class='flex items-center justify-between w-full gap-3 h-full relative'>
    <!-- Full-width draggable area -->
    <DraggableArea />

    <h1 class='text-base font-medium truncate min-w-0'>
      Playlists
    </h1>

    <!-- Spacer to balance left side -->
    <div class='min-w-0' />
  </div>
</template>