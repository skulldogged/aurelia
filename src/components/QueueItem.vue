<script setup lang="ts">
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { GripVertical, Play, Trash2, ListPlus } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'

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

defineProps<{
  song: MusicItem,
  isCurrent: boolean,
  isDragging: boolean
}>()

const emit = defineEmits(['remove', 'play'])

const formatDuration = (seconds: number) => {
  if (isNaN(seconds) || seconds < 0)
    return '0:00'
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = Math.floor(seconds % 60)
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}
</script>

<template>
  <ContextMenu>
    <ContextMenuTrigger>
      <div class="flex items-center p-2 rounded-lg transition-colors group border" :class="{
        'bg-muted border-border': isCurrent,
        'border-transparent': !isCurrent,
        'hover:bg-muted/50': !isCurrent && !isDragging,
      }">
        <Button variant="ghost" size="icon" class="handle cursor-grab">
          <GripVertical class="w-4 h-4 text-muted-foreground/40 transition-colors" :class="{
            'text-muted-foreground': isCurrent,
            'group-hover:text-muted-foreground': !isDragging,
          }" />
        </Button>
        <img :src="song.albumArtUrl || 'https://via.placeholder.com/40'" alt="Album Art"
          class="w-10 h-10 rounded-md mx-2">
        <div class="flex-grow cursor-pointer" @click="emit('play', song)">
          <p class="font-semibold text-sm">
            {{ song.name }}
          </p>
          <p class="text-xs text-muted-foreground">
            {{ song.artist }}
          </p>
        </div>
        <p v-if="song.duration" class="text-sm text-muted-foreground w-12 text-right">
          {{ formatDuration(song.duration) }}
        </p>
      </div>
    </ContextMenuTrigger>
    <ContextMenuContent>
      <ContextMenuItem>
        <Play class="w-4 h-4 mr-2" />
        Play Next
      </ContextMenuItem>
      <ContextMenuItem @click="emit('remove', song)">
        <Trash2 class="w-4 h-4 mr-2" />
        Remove from Queue
      </ContextMenuItem>
      <ContextMenuItem>
        <ListPlus class="w-4 h-4 mr-2" />
        Add to Playlist
      </ContextMenuItem>
    </ContextMenuContent>
  </ContextMenu>
</template>
