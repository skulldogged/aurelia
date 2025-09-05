<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Sortable } from 'sortablejs-vue3'
import QueueItem from './QueueItem.vue'

interface MusicItem {
  id: string
  name: string
  item_type: string
  album?: string
  artist?: string
  path?: string
  duration?: number
  albumArtUrl?: string
  trackNumber?: number
}

const props = defineProps<{
  modelValue: boolean
  playlist: MusicItem[]
  currentSong: MusicItem | null
}>()

const emit = defineEmits(['update:modelValue', 'update:playlist', 'remove-song', 'play-song'])

const isDragging = ref(false)

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
})

const handleRemove = (song: MusicItem) => {
  emit('remove-song', song)
}

const handlePlay = (song: MusicItem) => {
  emit('play-song', song)
}

const handleDragStart = () => {
  isDragging.value = true
}

const handleDragEnd = (event: any) => {
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
  <Sheet v-model:open="isOpen">
    <SheetContent class="w-[400px] sm:w-[540px] flex flex-col bg-card">
      <SheetHeader>
        <SheetTitle>Up Next</SheetTitle>
      </SheetHeader>
      <div class="flex-grow overflow-y-auto custom-scrollbar px-4">
        <Sortable :list="playlist" item-key="id" handle=".handle"
          :options="{ animation: 150, ghostClass: 'ghost', dragClass: 'drag' }" @start="handleDragStart"
          @end="handleDragEnd">
          <template #item="{ element: song }: { element: MusicItem }">
            <QueueItem :song="song" :is-current="currentSong?.id === song.id" :is-dragging="isDragging"
              @remove="handleRemove" @play="handlePlay" class="mb-1" />
          </template>
        </Sortable>
      </div>
    </SheetContent>
  </Sheet>
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
