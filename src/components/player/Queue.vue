<script setup lang="ts">
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { Sortable } from 'sortablejs-vue3'
  import { ref } from 'vue'

  import { Song } from '@/bindings'
  import QueueItem from '@/components/player/QueueItem.vue'
  import { usePlayerStore } from '@/stores'

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

  const handleDragEnd = (event: { newIndex: number | undefined; oldIndex: number | undefined, }): void => {
    isDragging.value = false
    const { newIndex, oldIndex } = event
    if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex)
      return

    const newList = [...playerStore.playlist]
    const [item] = newList.splice(oldIndex, 1)
    newList.splice(newIndex, 0, item)

    playerStore.setPlaylist(newList)
  }
</script>

<template>
  <div
    class='w-64 lg:w-80 xl:w-96 2xl:w-md bg-background-dark flex flex-col h-full'
  >
    <div
      class='h-12 flex items-center px-4 shrink-0'
      data-tauri-drag-region
    >
      <h2 class='text-base font-semibold tracking-tight text-muted-foreground'>
        Up Next
      </h2>
    </div>
    <OverlayScrollbarsComponent
      :options='{ scrollbars: { autoHide: "scroll" } }'
      class='grow px-2 py-3'
      defer
    >
      <Sortable
        @end='handleDragEnd'
        @start='handleDragStart'
        :list='playerStore.playlist'
        :options="{ animation: 150, ghostClass: 'ghost', dragClass: 'drag' }"
        handle='.handle'
        item-key='id'
      >
        <template #item='{ element: song, index }: { element: Song; index: number }'>
          <QueueItem
            @play='handlePlay'
            @remove='handleRemove'
            :index='index'
            :is-current='playerStore.currentIndex === index'
            :is-dragging='isDragging'
            :song='song'
            class='mb-1'
          />
        </template>
      </Sortable>
    </OverlayScrollbarsComponent>
  </div>
</template>

<style scoped>
.ghost {
  opacity: 0.5;
  background: var(--color-accent);
}

.drag {
  opacity: 0;
}
</style>
