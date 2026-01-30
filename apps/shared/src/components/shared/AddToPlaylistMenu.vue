<script setup lang="ts">
  import { ListPlus, Plus } from 'lucide-vue-next'
  import { useRouter } from 'vue-router'

  import { Song } from '../../lib/api/types'
  import {
    ContextMenuItem,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
  } from '../ui/context-menu'
  import {
    DropdownMenuItem,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger,
  } from '../ui/dropdown-menu'
  import { logger } from '../../lib/logger'
  import { usePlaylistStore } from '../../stores/playlists'

  const props = defineProps<{
    songs: Song[]
    type?: 'context' | 'dropdown' | 'flat'
  }>()

  const router = useRouter()
  const playlistStore = usePlaylistStore()

  const menuType = props.type || 'context'

  const addToPlaylist = async (playlistId: string): Promise<void> => {
    if (await playlistStore.addSongsToPlaylist(playlistId, props.songs)) {
      await playlistStore.loadPlaylists()
      logger.info(`Added ${props.songs.length} song(s) to playlist`)
    }
  }

  const createNewPlaylist = (): void => {
    router.push({
      path:  '/playlists/create',
      query: { songs: props.songs.map(s => s.id).join(',') },
    })
  }
</script>

<template>
  <!-- Context Menu Version -->
  <ContextMenuSub v-if="menuType === 'context'">
    <ContextMenuSubTrigger>
      <ListPlus class='size-4 mr-2' />
      Add to Playlist
    </ContextMenuSubTrigger>
    <ContextMenuSubContent>
      <ContextMenuItem
        v-for='playlist in playlistStore.playlists'
        @click='addToPlaylist(playlist.id)'
        :key='playlist.id'
      >
        {{ playlist.name }}
      </ContextMenuItem>
      <ContextMenuItem @click='createNewPlaylist'>
        <Plus class='size-4 mr-2' />
        Create New Playlist
      </ContextMenuItem>
    </ContextMenuSubContent>
  </ContextMenuSub>

  <!-- Dropdown Menu Version (Submenu) -->
  <DropdownMenuSub v-else-if="menuType === 'dropdown'">
    <DropdownMenuSubTrigger>
      <ListPlus class='size-4 mr-2' />
      Add to Playlist
    </DropdownMenuSubTrigger>
    <DropdownMenuSubContent>
      <DropdownMenuItem
        v-for='playlist in playlistStore.playlists'
        @click='addToPlaylist(playlist.id)'
        :key='playlist.id'
      >
        {{ playlist.name }}
      </DropdownMenuItem>
      <DropdownMenuItem @click='createNewPlaylist'>
        <Plus class='size-4 mr-2' />
        Create New Playlist
      </DropdownMenuItem>
    </DropdownMenuSubContent>
  </DropdownMenuSub>

  <!-- Flat Dropdown Version (Direct items, no submenu) -->
  <template v-else-if="menuType === 'flat'">
    <DropdownMenuItem
      v-for='playlist in playlistStore.playlists'
      @click='addToPlaylist(playlist.id)'
      :key='playlist.id'
    >
      {{ playlist.name }}
    </DropdownMenuItem>
    <DropdownMenuItem @click='createNewPlaylist'>
      <Plus class='size-4 mr-2' />
      Create New Playlist
    </DropdownMenuItem>
  </template>
</template>
