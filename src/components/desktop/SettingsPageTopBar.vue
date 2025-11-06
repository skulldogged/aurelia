<script setup lang="ts">
  import DraggableArea from '@/components/ui/DraggableArea.vue'

  defineProps<{
    activeTabLabel: string
  }>()
</script>

<template>
  <div class='flex items-center justify-between w-full gap-3 h-full relative'>
    <!-- Full-width draggable area -->
    <DraggableArea />

    <!-- Title on left -->
    <h1 class='text-base font-medium truncate min-w-0'>
      Settings
    </h1>
  </div>
</template>