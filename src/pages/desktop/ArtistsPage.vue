<script setup lang="ts">
  import { breakpointsTailwind, refDebounced, useBreakpoints, useWindowSize } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { Shuffle } from 'lucide-vue-next'
  import { computed, inject, onMounted, onUnmounted, ref, Ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Artist, Song } from '@/bindings'
  import ArtistsPageTopBar from '@/components/desktop/ArtistsPageTopBar.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useLayoutPreference } from '@/composables/useLayoutPreference'
  import { scrollElementKey } from '@/composables/useMainLayout'
  import { useOptimizedVirtualScroller } from '@/composables/useOptimizedVirtualScroller'
  import { useTopBar } from '@/composables/useTopBar'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const router = useRouter()

  const artistMode = ref<'album' | 'all'>('album')
  const { layout: viewLayout } = useLayoutPreference('artists-layout', 'comfy')

  const emit = defineEmits<{
    'play-song':     [song: Song]
    'play-songs':    [songs: Song[]]
    'select-artist': [artist: Artist]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  // Use top bar for title display
  const { clearTopBarContent, setTopBarContent } = useTopBar()

  // Create computed properties from stores
  const allArtists = computed(() => libraryStore.allArtistsWithSongs as Artist[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const searchQuery = ref('')
  const debouncedSearchQuery = refDebounced(searchQuery, 300)

  // All artists from the library are now album artists only (from /Artists/AlbumArtists endpoint)
  // Both "album" and "all" modes show the same list since we only fetch album artists
  const artistsToDisplay = computed(() => allArtists.value)

  // Deduplicate artists by name (not ID) to handle Jellyfin duplicate artist entries
  // For duplicates, keep the entry with the most songs
  const artistsWithSongs = computed(() => {
    const uniqueArtistsByName = new Map<string, Artist>()

    for (const artist of artistsToDisplay.value) {
      const normalizedName = artist.name.toLowerCase()
      const existing = uniqueArtistsByName.get(normalizedName)

      // Keep the artist with more songs, or the first one if equal
      if (!existing || (artist.songs?.length || 0) > (existing.songs?.length || 0))
        uniqueArtistsByName.set(normalizedName, artist)
    }

    return Array.from(uniqueArtistsByName.values()).sort((a, b) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
    )
  })

  // Fuzzy search setup (Fuse.js)
  // Recreate the Fuse instance when artists change to avoid duplication issues
  const artistsFuse = computed(() => new Fuse(artistsWithSongs.value, {
    includeScore:       true,
    keys:               ['name'],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  const filteredArtists = computed(() =>
    debouncedSearchQuery.value && debouncedSearchQuery.value.length >= 2
      ? artistsFuse.value.search(debouncedSearchQuery.value).map(result => result.item)
      : artistsWithSongs.value,
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

  const artistRows = computed(() => {
    const rows = []
    const items = filteredArtists.value
    for (let i = 0; i < items.length; i += cols.value) {
      rows.push(items.slice(i, i + cols.value))
    }
    return rows
  })

  const scrollElement = inject(scrollElementKey) as Ref<HTMLElement | null>

  const estimateSize = computed(() => viewLayout.value === 'compact' ? 250 : 300)

  const {
    isScrolling,
    remeasure,
    rowVirtualizer,
    virtualItems,
  } = useOptimizedVirtualScroller({
    count: computed(() => artistRows.value.length),
    estimateSize,
    scrollElement,
    viewLayout,
  })

  watch([artistRows, viewLayout, cols, scrollElement], () => {
    remeasure()
  })

  const virtualRows = computed(() =>
    virtualItems.value.map(item => ({
      artists:    artistRows.value[item.index],
      virtualRow: item.virtualRow,
    })),
  )

  const playArtistShuffle = (artist: Artist): void => {
    const artistSongs = artist.songs

    if (artistSongs && artistSongs.length > 0)
      emit('play-songs', [...artistSongs].sort(() => 0.5 - Math.random()))
  }

  const selectArtist = (artist: Artist): void => {
    if (artist.id)
      router.push(`/artists/${artist.id}`)
  }

  // Set up top bar content when component mounts
  onMounted(() => {
    setTopBarContent({
      component: ArtistsPageTopBar,
      id:        'artists-page',
      props:     {
        artistMode:            artistMode.value,
        'onUpdate:artistMode': (value: 'album' | 'all') => {
          artistMode.value = value
        },
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
  <section class='px-4 md:px-6 lg:px-8 py-4 md:py-6 lg:py-8'>
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
        <Skeleton class='w-full aspect-square rounded-lg' />
        <div class='flex flex-col items-center gap-1'>
          <Skeleton :class='viewLayout === "compact" ? "h-4 w-3/4" : "h-6 w-3/4"' />
          <Skeleton :class='viewLayout === "compact" ? "h-3 w-1/2" : "h-4 w-1/2"' />
        </div>
      </div>
    </div>
    <div
      v-else
      :style="{
        height: `${rowVirtualizer.getTotalSize()}px`,
        width: '100%',
        position: 'relative'
      }"
    >
      <div
        v-for='{ artists, virtualRow } in virtualRows'
        :key='String(virtualRow.key)'
        :ref='el => rowVirtualizer.measureElement(el as Element)'
        :data-index='virtualRow.index'
        :style='{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          transform: `translateY(${virtualRow.start}px)`,
          willChange: "transform",
          contain: "content"
        }'
      >
        <div
          :class='[
            viewLayout === "compact" ? "gap-4 pb-4" : "gap-6 pb-6",
            "grid"
          ]'
          :style='{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }'
        >
          <div
            v-for='artist in artists'
            @click='selectArtist(artist)'
            :key='artist.id'
            :class="['cursor-pointer', !isScrolling && 'group']"
          >
            <div :class='viewLayout === "compact" ? "relative mb-2" : "relative mb-4"'>
              <ImageLoader
                :alt='`${artist.name} artist image`'
                :is-scrolling='isScrolling'
                :item-id='artist.id'
                :server-url='serverUrl'
                :token='token'
                :width='itemWidth'
                class='w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity'
              >
                <template #fallback>
                  <ImagePlaceholder
                    class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                    size='large'
                    type='artist'
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
                  @click.stop='playArtistShuffle(artist)'
                  :size='viewLayout === "compact" ? "sm" : "icon"'
                  class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white
                    border border-white/20
                  '
                >
                  <Shuffle :class='viewLayout === "compact" ? "h-3.5 w-3.5" : "h-4 w-4"' />
                </Button>
              </div>
            </div>

            <div class='text-center'>
              <p
                :class='viewLayout === "compact"
                  ? "text-sm font-medium truncate"
                  : "font-semibold truncate"'
              >
                {{ artist.name }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <p
      v-if='!libraryLoading && filteredArtists && filteredArtists.length === 0'
      class='text-center py-12 text-muted-foreground'
    >
      No artists found
    </p>
  </section>
</template>