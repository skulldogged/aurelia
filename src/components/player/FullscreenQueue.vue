<script setup lang="ts">
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { Sortable } from 'sortablejs-vue3'
  import { ref } from 'vue'

  import { Song } from '@/bindings'
  import { usePlayerStore } from '@/stores'

  import QueueItem from './QueueItem.vue'

  const playerStore = usePlayerStore()

  const emit = defineEmits<{
    'remove-song':     [song: Song]
    'update:playlist': [playlist: Song[]]
  }>()

  const isDragging = ref(false)

  const handleRemove = (song: Song): void => {
    emit('remove-song', song)
  }

  const handlePlay = (song: Song, index: number): void => {
    playerStore.playSongAtIndex(index)
  }

  const handleDragStart = (): void => {
    isDragging.value = true
  }

  const handleDragEnd = (event: { newIndex: number | undefined; oldIndex: number | undefined }): void => {
    isDragging.value = false
    const { newIndex, oldIndex } = event
    if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex)
      return

    const newList = [...playerStore.playlist]
    const [item] = newList.splice(oldIndex, 1)
    newList.splice(newIndex, 0, item)

    playerStore.setPlaylist(newList)
    emit('update:playlist', newList)
  }
</script>

<template>
  <div class='flex flex-col h-full w-full'>
    <!-- Queue content with padding -->
    <OverlayScrollbarsComponent
      :class="'flex-1 p-4'"
      :options='{ scrollbars: { autoHide: "scroll" } }'
      defer
    >
      <div v-if='playerStore.playlist.length === 0' class='flex items-center justify-center h-full'>
        <div class='text-center space-y-2'>
          <p class='text-sm text-muted-foreground'>
            No songs in queue
          </p>
        </div>
      </div>

      <div v-else class='space-y-1'>
        <Sortable
          @end='handleDragEnd'
          @start='handleDragStart'
          :list='playerStore.playlist'
          class='space-y-1'
          handle='.drag-handle'
          item-key='id'
        >
          <template #item='{ element, index }'>
            <QueueItem
              @play='handlePlay(element, index)'
              @remove='handleRemove(element)'
              :index='index'
              :is-current='index === playerStore.currentIndex'
              :is-dragging='isDragging'
              :song='element'
            />
          </template>
        </Sortable>
      </div>
    </OverlayScrollbarsComponent>
  </div>
</template>