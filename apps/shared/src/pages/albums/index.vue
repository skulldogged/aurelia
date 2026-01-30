<script setup lang="ts">
  import { breakpointsTailwind, refDebounced, useBreakpoints, useWindowSize } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { computed, inject, onMounted, onUnmounted, ref, Ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { getApiClient, Album, Song } from '../../index'
  import AlbumsPageTopBar from '../../components/desktop/AlbumsPageTopBar.vue'
  import AddToPlaylistMenu from '../../components/shared/AddToPlaylistMenu.vue'
  import AlbumCard from '../../components/shared/AlbumCard.vue'
  import AlphabetNav from '../../components/shared/AlphabetNav.vue'
  import LibraryStats from '../../components/shared/LibraryStats.vue'
  import RecentlyAddedRow from '../../components/shared/RecentlyAddedRow.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '../../components/ui/context-menu'
  import { Skeleton } from '../../components/ui/skeleton'
  import { useLayoutPreference } from '../../composables/useLayoutPreference'
  import { scrollElementKey } from '../../composables/useMainLayout'
  import { useOptimizedVirtualScroller } from '../../composables/useOptimizedVirtualScroller'
  import { useTopBar } from '../../composables/useTopBar'
  import { useAuthStore } from '../../stores'
  import { useLibraryStore } from '../../stores/library'

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

  const breakpoints = useBreakpoints(breakpointsTailwind)
  const { width: windowWidth } = useWindowSize()

  const cols = computed(() => {
    const isCompact = viewLayout.value === 'compact'
    if (breakpoints.xl.value) return isCompact ? 8 : 7
    if (breakpoints.lg.value) return isCompact ? 7 : 6
    if (breakpoints.md.value) return isCompact ? 6 : 5
    if (breakpoints.sm.value) return isCompact ? 5 : 4
    return isCompact ? 4 : 3
  })

  const itemWidth = computed(() => {
    // Calculate approximate item width for optimal image loading
    const padding = breakpoints.lg.value ? 64 : breakpoints.md.value ? 48 : 32
    const gap = viewLayout.value === 'compact' ? 16 : 24
    const availableWidth = windowWidth.value - padding
    const totalGapWidth = (cols.value - 1) * gap
    return Math.round((availableWidth - totalGapWidth) / cols.value)
  })

  // Letter filter
  const letterFilter = ref<string | null>(null)

  // Available letters for alphabet nav (based on all albums, not filtered)
  const availableLetters = computed(() => {
    const letters = new Set<string>()
    const baseAlbums = debouncedSearchQuery.value && debouncedSearchQuery.value.length >= 2
      ? albumsFuse.value.search(debouncedSearchQuery.value).map(result => result.item)
      : allAlbums.value
    for (const album of baseAlbums) {
      const firstChar = album.name.charAt(0).toUpperCase()
      if (/[A-Z]/.test(firstChar)) {
        letters.add(firstChar)
      } else {
        letters.add('#')
      }
    }
    return letters
  })

  // Apply letter filter on top of search filter
  const displayedAlbums = computed(() => {
    if (!letterFilter.value) return filteredAlbums.value

    return filteredAlbums.value.filter((album) => {
      const firstChar = album.name.charAt(0).toUpperCase()
      if (letterFilter.value === '#') {
        return !/[A-Z]/.test(firstChar)
      }
      return firstChar === letterFilter.value
    })
  })

  const albumRows = computed(() => {
    const rows = []
    const items = displayedAlbums.value
    for (let i = 0; i < items.length; i += cols.value) {
      rows.push(items.slice(i, i + cols.value))
    }
    return rows
  })

  const scrollElement = inject(scrollElementKey) as Ref<HTMLElement | null>

  const estimateSize = computed(() => viewLayout.value === 'compact' ? 250 : 300)

  const {
    remeasure,
    rowVirtualizer,
    virtualItems,
  } = useOptimizedVirtualScroller({
    count: computed(() => albumRows.value.length),
    estimateSize,
    scrollElement,
    viewLayout,
  })

  watch([albumRows, viewLayout, cols, scrollElement, letterFilter], () => {
    remeasure()
  })

  const virtualRows = computed(() =>
    virtualItems.value.map(item => ({
      albums:     albumRows.value[item.index],
      virtualRow: item.virtualRow,
    })),
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

  const handleSelectAlbum = (album: Album): void => {
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
  <section class='px-4 md:px-6 lg:px-8 pt-4 md:pt-6 pb-8 max-w-7xl mx-auto'>
    <!-- Header section with stats -->
    <div v-if='!libraryLoading' class='mb-6 space-y-4'>
      <LibraryStats />

      <!-- Recently Added -->
      <RecentlyAddedRow
        @play-songs='emit("play-songs", $event)'
        @select-album='handleSelectAlbum'
      />

      <!-- Alphabet Navigation -->
      <div class='pt-2 border-t border-border/50'>
        <AlphabetNav
          @select='letterFilter = $event'
          :active-letter='letterFilter'
          :available-letters='availableLetters'
        />
      </div>
    </div>

    <!-- Loading state -->
    <div
      v-if='libraryLoading'
      :class='viewLayout === "compact"
        ? "grid grid-cols-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-7 xl:grid-cols-8 gap-4"
        : "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-6"'
    >
      <div
        v-for='n in 20'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-3'
      >
        <Skeleton class='w-full aspect-square rounded-lg' name='album-art' />
        <div class='flex flex-col gap-1'>
          <Skeleton :class='viewLayout === "compact" ? "h-3 w-3/4" : "h-4 w-3/4"' name='album-title' />
          <Skeleton :class='viewLayout === "compact" ? "h-2.5 w-1/2" : "h-3 w-1/2"' name='artist' />
        </div>
      </div>
    </div>

    <!-- Virtual scrolling grid -->
    <div
      v-else
      :style="{
        height: `${rowVirtualizer.getTotalSize()}px`,
        width: '100%',
        position: 'relative'
      }"
    >
      <div
        v-for='{ albums, virtualRow } in virtualRows'
        :key='String(virtualRow.key)'
        :ref='el => rowVirtualizer.measureElement(el as Element)'
        :data-index='virtualRow.index'
        :style='{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          transform: `translateY(${virtualRow.start}px)`,
          willChange: "transform"
        }'
      >
        <div
          :class='[
            viewLayout === "compact" ? "gap-4 pb-4 pt-1" : "gap-5 pb-5 pt-1",
            "grid"
          ]'
          :style='{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }'
        >
          <ContextMenu v-for='album in albums' :key='album.name'>
            <ContextMenuTrigger as-child>
              <AlbumCard
                @click='selectAlbum(album)'
                @play='playAlbum'
                :album='album'
                :compact='viewLayout === "compact"'
                :server-url='serverUrl'
                :token='token'
                :width='itemWidth'
              />
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click='playAlbum(album)'>
                Play Album
              </ContextMenuItem>
              <AddToPlaylistMenu
                :songs='album.songs ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)) : []'
                type='context'
              />
            </ContextMenuContent>
          </ContextMenu>
        </div>
      </div>
    </div>

    <p v-if='!libraryLoading && displayedAlbums.length === 0' class='text-center py-12 text-muted-foreground'>
      No albums found{{ letterFilter ? ` starting with "${letterFilter}"` : '' }}
    </p>
  </section>
</template>

<style scoped>
@reference "tailwindcss";
</style>