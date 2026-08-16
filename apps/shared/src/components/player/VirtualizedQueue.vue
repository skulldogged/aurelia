<script setup lang="ts">
  import { useVirtualizer } from '@tanstack/vue-virtual'
  import { Sortable } from 'sortablejs-vue3'
  import { computed, ref } from 'vue'

  import type { Song } from '../../lib/api/types'

  import { usePlayerStore } from '../../stores'
  import QueueItem from './QueueItem.vue'

  interface QueueEntry {
    index: number
    key:   string
    song:  Song
  }

  const QUEUE_ITEM_HEIGHT = 60

  const playerStore = usePlayerStore()
  const scrollElement = ref<HTMLElement | null>(null)
  const isDragging = ref(false)
  const dragEntries = ref<QueueEntry[]>([])

  const emit = defineEmits<{
    'remove-song':     [song: Song]
    'update:playlist': [playlist: Song[]]
  }>()

  const rowVirtualizer = useVirtualizer(computed(() => ({
    count:            playerStore.playlist.length,
    estimateSize:     () => QUEUE_ITEM_HEIGHT,
    getScrollElement: () => scrollElement.value,
    overscan:         8,
  })))

  const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())
  const visibleEntries = computed<QueueEntry[]>(() =>
    virtualRows.value.flatMap(row => {
      const song = playerStore.playlist[row.index]
      if (!song) return []
      return [{
        index: row.index,
        key:   `${song.id}-${row.index}`,
        song,
      }]
    }),
  )
  const visibleOffset = computed(() => virtualRows.value[0]?.start ?? 0)

  const handlePlay = (_song: Song, index: number): void => {
    playerStore.playSongAtIndex(index)
  }

  const handleDragStart = (): void => {
    isDragging.value = true
    dragEntries.value = [...visibleEntries.value]
  }

  const handleDragEnd = (event: { newIndex: number | undefined; oldIndex: number | undefined }): void => {
    isDragging.value = false
    const { newIndex, oldIndex } = event
    if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) {
      dragEntries.value = []
      return
    }

    const oldQueueIndex = dragEntries.value[oldIndex]?.index
    const newQueueIndex = visibleEntries.value[newIndex]?.index
    dragEntries.value = []
    if (oldQueueIndex === undefined || newQueueIndex === undefined)
      return

    const newList = [...playerStore.playlist]
    const [item] = newList.splice(oldQueueIndex, 1)
    if (!item) return
    newList.splice(newQueueIndex, 0, item)

    playerStore.setPlaylist(newList)
    emit('update:playlist', newList)
  }
</script>

<template>
  <div ref='scrollElement' class='overflow-y-auto overflow-x-hidden'>
    <div v-if='playerStore.playlist.length === 0' class='flex items-center justify-center h-full'>
      <p class='text-sm text-muted-foreground'>
        No songs in queue
      </p>
    </div>

    <div
      v-else
      :style='{ height: `${rowVirtualizer.getTotalSize()}px` }'
      class='relative'
    >
      <Sortable
        @end='handleDragEnd'
        @start='handleDragStart'
        :list='visibleEntries'
        :options="{ animation: 150, ghostClass: 'ghost', dragClass: 'drag' }"
        :style='{ transform: `translateY(${visibleOffset}px)` }'
        class='absolute top-0 left-0 right-0'
        handle='.handle'
        item-key='key'
      >
        <template #item='{ element }: { element: QueueEntry }'>
          <QueueItem
            @play='handlePlay'
            @remove="emit('remove-song', $event)"
            :index='element.index'
            :is-current='playerStore.currentIndex === element.index'
            :is-dragging='isDragging'
            :song='element.song'
            class='mb-1'
          />
        </template>
      </Sortable>
    </div>
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
