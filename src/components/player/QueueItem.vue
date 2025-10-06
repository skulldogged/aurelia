<script setup lang="ts">
  import { GripVertical, ListPlus, Play, Share2, Trash2 } from 'lucide-vue-next'
  import { ref } from 'vue'

  import { Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import { Button } from '@/components/ui/button'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { formatDuration } from '@/lib/utils'

  defineProps<{
    class?:     string,
    isCurrent:  boolean,
    isDragging: boolean,
    serverUrl?: string,
    song:       Song,
    token?:     string
  }>()

  const emit = defineEmits<{
    play:   [song: Song]
    remove: [song: Song]
  }>()

  const showShareDialog = ref(false)
</script>

<template>
  <ContextMenu>
    <ContextMenuTrigger>
      <div
        :class="[
          {
            'bg-accent': isCurrent,
            'hover:bg-accent/20': !isCurrent && !isDragging,
          },
          $props.class
        ]"
        class='flex items-center p-2 rounded-lg transition-colors group'
      >
        <Button class='handle cursor-grab w-4 h-8 p-1' variant='ghost'>
          <GripVertical
            :class="[
              'w-2 h-4 transition-colors',
              isCurrent ? 'text-accent-foreground' : 'text-muted-foreground',
              !isDragging ? 'group-hover:text-muted-foreground' : ''
            ]"
          />
        </Button>
        <ImageLoader
          v-if='serverUrl && token'
          :item-id='song.id'
          :server-url='serverUrl'
          :token='token'
          alt='Album Art'
          class='w-10 h-10 rounded-md mx-2'
        />
        <div @click="emit('play', song)" class='flex-grow cursor-pointer min-w-0'>
          <p :class="['font-semibold text-sm truncate', isCurrent ? 'text-accent-foreground' : '']" :title='song.name'>
            {{ song.name }}
          </p>
          <p
            :class="['text-xs truncate', isCurrent ? 'text-accent-foreground' : 'text-muted-foreground']"
            :title='song.artists?.join(", ") || "Unknown Artist"'
          >
            <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
              <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                <router-link
                  @click.stop
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
        <p
          v-if='song.duration'
          :class="['text-sm w-12 text-right ml-2', isCurrent ? 'text-accent-foreground' : 'text-muted-foreground']"
        >
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
      <ContextMenuItem @click='showShareDialog = true'>
        <Share2 class='w-4 h-4 mr-2' />
        Share
      </ContextMenuItem>
    </ContextMenuContent>
  </ContextMenu>

  <ShareDialog
    v-model:open='showShareDialog'
    :item-id='song.id'
    :item-name='song.name'
    :item-type="'song'"
  />
</template>
