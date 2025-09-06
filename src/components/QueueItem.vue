<script setup lang="ts">
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { GripVertical, Play, Trash2, ListPlus } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'
  import { MusicItem } from '@/types'

  defineProps<{
    song:       MusicItem,
    isCurrent:  boolean,
    isDragging: boolean
  }>()

  const emit = defineEmits<{
    remove: [song: MusicItem]
    play:   [song: MusicItem]
  }>()

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
      <div
        :class="{
          'bg-muted border-border': isCurrent,
          'border-transparent': !isCurrent,
          'hover:bg-muted/50': !isCurrent && !isDragging,
        }"
        class='flex items-center p-2 rounded-lg transition-colors group border'
      >
        <Button class='handle cursor-grab' size='icon' variant='ghost'>
          <GripVertical
            :class="{
              'text-muted-foreground': isCurrent,
              'group-hover:text-muted-foreground': !isDragging,
            }"
            class='w-4 h-4 text-muted-foreground/40 transition-colors'
          />
        </Button>
        <img
          :src="song.albumArtUrl || 'https://via.placeholder.com/40'"
          alt='Album Art'
          class='w-10 h-10 rounded-md mx-2'
        >
        <div @click="emit('play', song)" class='flex-grow cursor-pointer'>
          <p class='font-semibold text-sm'>
            {{ song.name }}
          </p>
          <p class='text-xs text-muted-foreground'>
            <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
              <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                <router-link
                  :to="{ name: 'artist-detail', params: { artistId: song.artistIds[index] } }"
                  class='hover:underline'
                >
                  {{ artist }}
                </router-link>
                <span v-if='index < song.artists.length - 1'>, </span>
              </template>
            </template>
            <template v-else>
              {{ song.artists?.join(', ') || 'Unknown Artist' }}
            </template>
          </p>
        </div>
        <p v-if='song.duration' class='text-sm text-muted-foreground w-12 text-right'>
          {{ formatDuration(song.duration) }}
        </p>
      </div>
    </ContextMenuTrigger>
    <ContextMenuContent>
      <ContextMenuItem>
        <Play class='w-4 h-4 mr-2' />
        Play Next
      </ContextMenuItem>
      <ContextMenuItem @click="emit('remove', song)">
        <Trash2 class='w-4 h-4 mr-2' />
        Remove from Queue
      </ContextMenuItem>
      <ContextMenuItem>
        <ListPlus class='w-4 h-4 mr-2' />
        Add to Playlist
      </ContextMenuItem>
    </ContextMenuContent>
  </ContextMenu>
</template>
