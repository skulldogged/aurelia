<script setup lang="ts">
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { Sortable } from 'sortablejs-vue3'
  import { ref } from 'vue'

  import { Song } from '@/bindings'

  import QueueItem from './QueueItem.vue'

  const props = defineProps<{
    currentSong: null | Song
    playlist:    Song[]
  }>()

  const emit = defineEmits<{
    'play-song':       [song: Song]
    'remove-song':     [song: Song]
    'update:playlist': [playlist: Song[]]
  }>()

  const isDragging = ref(false)

  const handleRemove = (song: Song): void => {
    emit('remove-song', song)
  }

  const handlePlay = (song: Song): void => {
    emit('play-song', song)
  }

  const handleDragStart = (): void => {
    isDragging.value = true
  }

  const handleDragEnd = (event: { newIndex: number | undefined; oldIndex: number | undefined, }): void => {
    isDragging.value = false
    const { newIndex, oldIndex } = event
    if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex)
      return

    const newList = [...props.playlist]
    const [item] = newList.splice(oldIndex, 1)
    newList.splice(newIndex, 0, item)

    emit('update:playlist', newList)
  }
</script>

<template>
  <div class='w-64 lg:w-80 xl:w-96 2xl:w-[28rem] flex flex-col bg-background-dark h-full'>
    <div class='h-12 flex items-center px-4 flex-shrink-0' data-tauri-drag-region>
      <h2 class='text-base font-semibold tracking-tight text-muted-foreground'>
        Up Next
      </h2>
    </div>
    <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='flex-grow px-2 py-3' defer>
      <Sortable
        @end='handleDragEnd'
        @start='handleDragStart'
        :list='playlist'
        :options="{ animation: 150, ghostClass: 'ghost', dragClass: 'drag' }"
        handle='.handle'
        item-key='id'
      >
        <template #item='{ element: song }: { element: Song }'>
          <QueueItem
            @play='handlePlay'
            @remove='handleRemove'
            :is-current='currentSong?.id === song.id'
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
