<script setup lang="ts">
  import { ChevronLeft, ChevronRight, Sparkles } from 'lucide-vue-next'
  import { computed, ref } from 'vue'

  import type { Album, Song } from '@/lib/api/bindings'

  import AlbumCard from '@/components/shared/AlbumCard.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import { sortSongsByTrackOrder } from '@/lib/transforms'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()

  // Get recently added albums (sorted by dateCreated descending)
  const recentAlbums = computed<Album[]>(() => {
    const albums = [...(libraryStore.allAlbums || [])] as Album[]
    return albums
      .filter(a => a.dateCreated)
      .sort((a, b) => {
        const dateA = new Date(a.dateCreated!).getTime()
        const dateB = new Date(b.dateCreated!).getTime()
        return dateB - dateA
      })
      .slice(0, 20)
  })

  const scrollContainer = ref<HTMLElement | null>(null)
  const canScrollLeft = ref(false)
  const canScrollRight = ref(true)

  const updateScrollState = (): void => {
    if (!scrollContainer.value) return
    const { scrollLeft, scrollWidth, clientWidth } = scrollContainer.value
    canScrollLeft.value = scrollLeft > 0
    canScrollRight.value = scrollLeft + clientWidth < scrollWidth - 10
  }

  const scroll = (direction: 'left' | 'right'): void => {
    if (!scrollContainer.value) return
    const scrollAmount = scrollContainer.value.clientWidth * 0.8
    scrollContainer.value.scrollBy({
      behavior: 'smooth',
      left:     direction === 'left' ? -scrollAmount : scrollAmount,
    })
  }

  const playAlbum = (album: Album): void => {
    if (album.songs?.length) {
      emit('play-songs', sortSongsByTrackOrder(album.songs))
    }
  }

  const selectAlbum = (album: Album): void => {
    emit('select-album', album)
  }
</script>

<template>
  <section v-if='recentAlbums.length > 0' class='relative overflow-hidden'>
    <!-- Header -->
    <div class='flex items-center justify-between mb-4'>
      <div class='flex items-center gap-2'>
        <Sparkles class='size-5 text-accent' />
        <h2 class='text-lg font-semibold'>Recently Added</h2>
      </div>

      <!-- Scroll controls -->
      <div class='flex items-center gap-1'>
        <button
          @click='scroll("left")'
          :disabled='!canScrollLeft'
          class='
            p-1.5 rounded-full bg-muted/50 hover:bg-muted
            disabled:opacity-30 disabled:cursor-not-allowed
            transition-colors
          '
        >
          <ChevronLeft class='size-4' />
        </button>
        <button
          @click='scroll("right")'
          :disabled='!canScrollRight'
          class='
            p-1.5 rounded-full bg-muted/50 hover:bg-muted
            disabled:opacity-30 disabled:cursor-not-allowed
            transition-colors
          '
        >
          <ChevronRight class='size-4' />
        </button>
      </div>
    </div>

    <!-- Scrollable row -->
    <div
      ref='scrollContainer'
      @scroll='updateScrollState'
      class='flex gap-4 overflow-x-auto pt-1 pb-3 scrollbar-none snap-x snap-mandatory'
    >
      <ContextMenu v-for='album in recentAlbums' :key='album.id || album.name'>
        <ContextMenuTrigger as-child>
          <div class='shrink-0 w-36 md:w-40 lg:w-44 snap-start'>
            <AlbumCard
              @click='selectAlbum(album)'
              @play='playAlbum'
              :album='album'
              :server-url='serverUrl'
              :token='token'
              :width='180'
              compact
            />
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem @click='playAlbum(album)'>
            Play Album
          </ContextMenuItem>
          <AddToPlaylistMenu
            :songs='album.songs ? sortSongsByTrackOrder(album.songs) : []'
            type='context'
          />
        </ContextMenuContent>
      </ContextMenu>
    </div>

    <!-- Fade edges -->
    <div
      v-if='canScrollLeft'
      class='absolute left-0 top-12 bottom-0 w-8 bg-gradient-to-r from-background to-transparent pointer-events-none'
    />
    <div
      v-if='canScrollRight'
      class='absolute right-0 top-12 bottom-0 w-8 bg-gradient-to-l from-background to-transparent pointer-events-none'
    />
  </section>
</template>

<style scoped>
.scrollbar-none {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
.scrollbar-none::-webkit-scrollbar {
  display: none;
}
</style>
