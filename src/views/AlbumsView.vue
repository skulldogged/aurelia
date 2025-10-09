<script setup lang="ts">
  import { useMediaQuery } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { Play } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, Song } from '@/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { Input } from '@/components/ui/input'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { Skeleton } from '@/components/ui/skeleton'
  import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
  import { useLayoutPreference, usePagination } from '@/composables/useLayoutPreference'
  import { useAuthStore, useLibraryStore } from '@/stores'

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

  const { layout: viewLayout } = useLayoutPreference('albums-layout', 'comfy')

  // Detect current breakpoint for column calculations
  const isXl = useMediaQuery('(min-width: 1280px)')
  const isLg = useMediaQuery('(min-width: 1024px)')
  const isMd = useMediaQuery('(min-width: 768px)')
  const isSm = useMediaQuery('(min-width: 640px)')

  // Calculate current column count based on viewport and layout mode
  const currentColumns = computed(() => {
    if (viewLayout.value === 'compact') {
      // Compact: 3 sm:4 md:5 lg:6 xl:7
      if (isXl.value) return 7
      if (isLg.value) return 6
      if (isMd.value) return 5
      if (isSm.value) return 4
      return 3
    } else {
      // Comfy: 2 sm:3 md:4 lg:5
      if (isLg.value) return 5
      if (isMd.value) return 4
      if (isSm.value) return 3
      return 2
    }
  })

  const allAlbums = computed(() => libraryStore.allAlbums as Album[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const albumsFuse = ref(new Fuse(allAlbums.value, {
    includeScore: true,
    keys:         [
      { name: 'name', weight: 0.6 },
      { name: 'artist', weight: 0.4 },
    ],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  watch(() => allAlbums.value, newAlbums => {
    albumsFuse.value.setCollection(newAlbums as Album[])
  })

  const filteredAlbums = computed(() =>
    searchQuery.value && searchQuery.value.length >= 2
      ? albumsFuse.value.search(searchQuery.value).map(result => result.item)
      : [...allAlbums.value].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())),
  )

  // User selects number of rows (1-5), which stays constant
  const rowsPerPage = ref(3) // Default: 3 rows

  // Items per page = rows × current columns (changes with viewport)
  const itemsPerPage = computed(() => rowsPerPage.value * currentColumns.value)

  // Static row options (always 1-5)
  const rowOptions = [1, 2, 3, 4, 5]

  // Dynamic storage key based on layout mode (stores row count, not item count)
  const pageSizeKey = computed(() =>
    viewLayout.value === 'compact'
      ? 'albums-rows-compact'
      : 'albums-rows-comfy',
  )

  // Load saved row count from localStorage
  watch(pageSizeKey, newKey => {
    const saved = localStorage.getItem(newKey)
    if (saved) {
      const parsed = parseInt(saved, 10)
      if (!isNaN(parsed) && parsed >= 1 && parsed <= 5) {
        rowsPerPage.value = parsed
      }
    }
  }, { immediate: true })

  // Save row count to localStorage when it changes
  watch(rowsPerPage, newRows => {
    localStorage.setItem(pageSizeKey.value, String(newRows))
  })

  // Pagination - pass itemsPerPage as the dynamic page size
  const {
    canNextPage,
    canPreviousPage,
    goToFirstPage,
    goToLastPage,
    goToNextPage,
    goToPreviousPage,
    pageCount,
    pagedItems: pagedAlbums,
    pageIndex,
  } = usePagination(filteredAlbums, 'albums-pagesize', itemsPerPage, [])

  const setRowsPerPage = (rows: number): void => {
    rowsPerPage.value = rows
  }

  const playAlbum = (album: Album): void => {
    if (album.songs && album.songs.length > 0) {
      const sortedSongs = [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
      emit('play-songs', sortedSongs)
    }
  }

  const selectAlbum = (album: Album): void => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <div class='flex justify-between items-start mb-4'>
        <h1 class='text-4xl font-bold'>
          Albums
        </h1>
        <Tabs v-model='viewLayout'>
          <TabsList>
            <TabsTrigger value='comfy'>
              Comfy
            </TabsTrigger>
            <TabsTrigger value='compact'>
              Compact
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>
      <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center'>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search albums...'
          type='text'
        />
      </div>
    </div>

    <div class='bg-sidebar rounded-lg p-6'>
      <div
        v-if='libraryLoading'
        :class='viewLayout === "compact"
          ? "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-4"
          : "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6"'
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
          ? "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-4"
          : "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6"'
      >
        <ContextMenu v-for='album in pagedAlbums' :key='album.name'>
          <ContextMenuTrigger as-child>
            <div
              @click='selectAlbum(album)'
              class='cursor-pointer group'
            >
              <div :class='viewLayout === "compact" ? "relative mb-2" : "relative mb-4"'>
                <ImageLoader
                  :alt='`${album.name} album art`'
                  :item-id='album.id || album.name'
                  :server-url='serverUrl'
                  :token='token'
                  class='
                    w-full aspect-square rounded-lg object-cover shadow-lg
                    group-hover:opacity-75 transition-opacity
                  '
                >
                  <template #fallback>
                    <ImagePlaceholder
                      class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                      size='large'
                      type='album'
                    />
                  </template>
                </ImageLoader>

                <div
                  class='
                    absolute inset-0 bg-black/50 rounded-lg opacity-0
                    group-hover:opacity-100 transition-opacity flex items-center
                    justify-center
                  '
                >
                  <Button
                    @click.stop='playAlbum(album)'
                    :size='viewLayout === "compact" ? "sm" : "icon"'
                    class='
                      bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                      border-white/20
                    '
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
                <p
                  v-if='album.songs'
                  :class='viewLayout === "compact"
                    ? "text-xs text-muted-foreground truncate"
                    : "text-sm text-muted-foreground truncate"'
                >
                  {{ album.songs.length }} songs
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

      <!-- Pagination Controls -->
      <div v-if='pageCount > 1' class='flex flex-col sm:flex-row items-center justify-between gap-4 mt-6'>
        <div class='flex items-center gap-2'>
          <span class='text-sm text-muted-foreground'>Rows per page:</span>
          <Select @update:model-value='(v) => setRowsPerPage(Number(v))' :model-value='String(rowsPerPage)'>
            <SelectTrigger class='w-[80px]'>
              <SelectValue placeholder='Rows' />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem v-for='row in rowOptions' :key='row' :value='String(row)'>
                  {{ row }}
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>

        <div class='flex items-center gap-2'>
          <span class='text-sm text-muted-foreground'>
            Page {{ pageIndex + 1 }} of {{ pageCount }}
          </span>

          <div class='flex items-center gap-1'>
            <Button
              @click='goToFirstPage'
              :disabled='!canPreviousPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              First
            </Button>
            <Button
              @click='goToPreviousPage'
              :disabled='!canPreviousPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              Previous
            </Button>
            <Button
              @click='goToNextPage'
              :disabled='!canNextPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              Next
            </Button>
            <Button
              @click='goToLastPage'
              :disabled='!canNextPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              Last
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
