<script setup lang="ts">
  import { refDebounced } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { Play } from 'lucide-vue-next'
  import { computed, onMounted, onUnmounted, ref } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, Song } from '@/bindings'
  import AlbumsPageTopBar from '@/components/desktop/AlbumsPageTopBar.vue'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import AlbumStack from '@/components/shared/AlbumStack.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useLayoutPreference } from '@/composables/useLayoutPreference'
  import { useTopBar } from '@/composables/useTopBar'
  import { useAuthStore } from '@/stores'
  import { useLibraryStore } from '@/stores/library'

  const router = useRouter()
  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  defineProps<{
    currentSong?: null | Song
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()

  const searchQuery = ref('')
  const debouncedSearchQuery = refDebounced(searchQuery, 300)

  const { layout: viewLayout } = useLayoutPreference('albums-layout', 'comfy')

  // Use top bar for title display
  const { clearTopBarContent, setTopBarContent } = useTopBar()

  const allAlbums = computed(() => libraryStore.allAlbums as Album[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const albumsFuse = computed(() => new Fuse(allAlbums.value, {
    includeScore: true,
    keys:         [
      { name: 'name', weight: 0.6 },
      { name: 'artist', weight: 0.4 },
    ],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  const filteredAlbums = computed(() =>
    debouncedSearchQuery.value && debouncedSearchQuery.value.length >= 2
      ? albumsFuse.value.search(debouncedSearchQuery.value).map(result => result.item)
      : [...allAlbums.value].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())),
  )

  const playAlbum = (album: Album): void => {
    if (album.songs && album.songs.length > 0) {
      const sortedSongs = [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
      emit('play-songs', sortedSongs)
    }
  }

  const selectAlbum = (album: Album): void => {
    if (album.id)
      router.push(`/albums/${album.id}`)
  }

  // Set up top bar content when component mounts
  onMounted(() => {
    setTopBarContent({
      component: AlbumsPageTopBar,
      id:        'albums-page',
      props:     {
        'onUpdate:searchQuery': (value: string) => {
          searchQuery.value = value
        },
        'onUpdate:viewLayout': (value: string) => {
          viewLayout.value = value as 'comfy' | 'compact'
        },
        searchQuery: searchQuery.value,
        viewLayout:  viewLayout.value,
      },
    })
  })

  // Clean up top bar content when component unmounts
  onUnmounted(() => {
    clearTopBarContent()
  })
</script>

<template>
  <div class='px-4 md:px-6 lg:px-8 py-4 md:py-6 lg:py-8'>
    <div
      v-if='libraryLoading'
      :class='viewLayout === "compact"
        ? "grid grid-cols-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8 gap-4"
        : "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-6"'
    >
      <div
        v-for='n in 20'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-4'
      >
        <Skeleton class='w-full aspect-square rounded-lg' name='album-art' />
        <div class='flex flex-col gap-1'>
          <Skeleton :class='viewLayout === "compact" ? "h-4 w-3/4" : "h-6 w-3/4"' name='album-title' />
          <Skeleton :class='viewLayout === "compact" ? "h-3 w-20" : "h-4 w-20"' name='artist' />
          <Skeleton :class='viewLayout === "compact" ? "h-3 w-16" : "h-4 w-16"' name='song-count' />
        </div>
      </div>
    </div>
    <div
      v-else
      :class='viewLayout === "compact"
        ? "grid grid-cols-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8 gap-4"
        : "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-6"'
    >
      <ContextMenu v-for='album in filteredAlbums' :key='album.name'>
        <ContextMenuTrigger as-child>
          <div
            @click='selectAlbum(album)'
            class='cursor-pointer group'
          >
            <div
              :class='viewLayout === "compact"
                ? "relative mb-2"
                : "relative mb-4"'
            >
              <AlbumStack
                @play='playAlbum'
                :album='album'
                :server-url='serverUrl'
                :show-play-button='false'
                :size='"responsive"'
                :token='token'
              />

              <div
                class='
                  absolute inset-0 bg-black/25 rounded-xl opacity-0
                  group-hover:opacity-100 transition-opacity flex items-center
                  justify-center z-10
                '
              >
                <Button
                  @click.stop='playAlbum(album)'
                  class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                    border-white/20
                  '
                  size='icon'
                >
                  <Play :class='viewLayout === "compact" ? "h-3.5 w-3.5" : "h-4 w-4"' />
                </Button>
              </div>
            </div>

            <div>
              <p
                :class='viewLayout === "compact"
                  ? "text-sm font-medium truncate"
                  : "font-semibold truncate"'
              >
                {{ album.name }}
              </p>
              <p
                :class='viewLayout === "compact"
                  ? "text-xs text-muted-foreground truncate"
                  : "text-sm text-muted-foreground truncate"'
              >
                {{ album.artist }}
              </p>
            </div>
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem @click='playAlbum(album)'>
            <Play class='size-4 mr-2' />
            Play Album
          </ContextMenuItem>
          <AddToPlaylistMenu
            :songs='album.songs ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)) : []'
            type='context'
          />
        </ContextMenuContent>
      </ContextMenu>
    </div>

    <div v-if='!libraryLoading && filteredAlbums.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground'>
        No albums found
      </p>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

/* Album stack effect */
</style>