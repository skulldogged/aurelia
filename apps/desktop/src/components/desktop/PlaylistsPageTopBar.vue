<script setup lang="ts">
  import Button from '@shared/components/ui/Button.vue'
  import DraggableArea from '@shared/components/ui/DraggableArea.vue'
  import { Input } from '@shared/components/ui/input'
  import { Plus } from 'lucide-vue-next'

  defineProps<{
    searchQuery: string
  }>()

  defineEmits<{
    (e: 'update:searchQuery', value: string): void
    (e: 'create-playlist'): void
  }>()
</script>

<template>
  <div class='flex items-center justify-between w-full gap-3 h-full relative'>
    <!-- Full-width draggable area -->
    <DraggableArea />

    <h1 class='text-base font-medium truncate min-w-0'>
      Playlists
    </h1>

    <div class='flex items-center gap-1.5 min-w-0 relative z-10'>
      <Input
        @update:model-value='$emit("update:searchQuery", String($event))'
        :model-value='searchQuery'
        class='
          h-8 w-32 bg-background/60 border-0
          focus-visible:ring-1 focus-visible:ring-ring/50
        '
        placeholder='Search...'
        type='text'
      />

      <Button
        @click='$emit("create-playlist")'
        class='h-8 px-3 gap-2 text-xs'
        size='sm'
      >
        <Plus class='h-4 w-4' />
        Create
      </Button>
    </div>
  </div>
</template>
