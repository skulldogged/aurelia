<script setup lang="ts">
  import { ref } from 'vue'
  import { Sortable } from 'sortablejs-vue3'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import QueueItem from './QueueItem.vue'
  import { Song } from '@/bindings'

  const props = defineProps<{
    playlist:    Song[]
    currentSong: Song | null
  }>()

  const emit = defineEmits<{
    'update:playlist': [playlist: Song[]]
    'remove-song':     [song: Song]
    'play-song':       [song: Song]
  }>()

  const isDragging = ref(false)

  const handleRemove = (song: Song) => {
    emit('remove-song', song)
  }

  const handlePlay = (song: Song) => {
    emit('play-song', song)
  }

  const handleDragStart = () => {
    isDragging.value = true
  }

  const handleDragEnd = (event: { oldIndex: number | undefined, newIndex: number | undefined }) => {
    isDragging.value = false
    const { oldIndex, newIndex } = event
    if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex)
      return

    const newList = [...props.playlist]
    const [item] = newList.splice(oldIndex, 1)
    newList.splice(newIndex, 0, item)

    emit('update:playlist', newList)
  }
</script>

<template>
  <div class='w-64 lg:w-72 xl:w-80 flex flex-col bg-sidebar'>
    <div class='p-4'>
      <h2 class='text-lg font-semibold'>
        Up Next
      </h2>
    </div>
    <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='flex-grow px-2' defer>
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
