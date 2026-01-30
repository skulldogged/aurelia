<script setup lang="ts">
  import { Disc3, Library, Mic2, Music } from 'lucide-vue-next'
  import { computed } from 'vue'

  import { useLibraryStore } from '../../stores/library'

  const libraryStore = useLibraryStore()

  const stats = computed(() => {
    const albums = libraryStore.allAlbums?.length || 0
    const artists = libraryStore.allArtistsWithSongs?.length || 0
    const songs = libraryStore.allSongs?.length || 0

    // Calculate total duration
    const totalSeconds = libraryStore.allSongs?.reduce(
      (acc, song) => acc + (song.duration || 0),
      0,
    ) || 0

    const hours = Math.floor(totalSeconds / 3600)
    const days = Math.floor(hours / 24)

    let duration = ''
    if (days > 0) {
      duration = `${days}d ${hours % 24}h`
    } else if (hours > 0) {
      duration = `${hours}h ${Math.floor((totalSeconds % 3600) / 60)}m`
    } else {
      duration = `${Math.floor(totalSeconds / 60)}m`
    }

    return { albums, artists, duration, songs }
  })
</script>

<template>
  <div class='flex flex-wrap items-center gap-x-6 gap-y-2 text-sm text-muted-foreground'>
    <div class='flex items-center gap-1.5'>
      <Disc3 class='size-4' />
      <span>{{ stats.albums.toLocaleString() }} albums</span>
    </div>
    <div class='flex items-center gap-1.5'>
      <Mic2 class='size-4' />
      <span>{{ stats.artists.toLocaleString() }} artists</span>
    </div>
    <div class='flex items-center gap-1.5'>
      <Music class='size-4' />
      <span>{{ stats.songs.toLocaleString() }} songs</span>
    </div>
    <div class='flex items-center gap-1.5'>
      <Library class='size-4' />
      <span>{{ stats.duration }}</span>
    </div>
  </div>
</template>
