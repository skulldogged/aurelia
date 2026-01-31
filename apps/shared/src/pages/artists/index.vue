<script setup lang="ts">
  import { breakpointsTailwind, refDebounced, useBreakpoints, useWindowSize } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { computed, inject, onMounted, onUnmounted, ref, Ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import ArtistsPageTopBar from '../../components/desktop/ArtistsPageTopBar.vue'
  import AlphabetNav from '../../components/shared/AlphabetNav.vue'
  import ArtistCard from '../../components/shared/ArtistCard.vue'
  import LibraryStats from '../../components/shared/LibraryStats.vue'
  import { Skeleton } from '../../components/ui/skeleton'
  import { useLayoutPreference } from '../../composables/useLayoutPreference'
  import { scrollElementKey } from '../../composables/useMainLayout'
  import { useOptimizedVirtualScroller } from '../../composables/useOptimizedVirtualScroller'
  import { useTopBar } from '../../composables/useTopBar'
  import { Artist, Song } from '../../index'
  import { useAuthStore } from '../../stores/auth'
  import { useLibraryStore } from '../../stores/library'

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

  // Letter filter
  const letterFilter = ref<null | string>(null)

  // Available letters for alphabet nav
  const availableLetters = computed(() => {
    const letters = new Set<string>()
    for (const artist of filteredArtists.value) {
      const firstChar = artist.name.charAt(0).toUpperCase()
      if (/[A-Z]/.test(firstChar)) {
        letters.add(firstChar)
      } else {
        letters.add('#')
      }
    }
    return letters
  })

  // Apply letter filter
  const displayedArtists = computed(() => {
    if (!letterFilter.value) return filteredArtists.value

    return filteredArtists.value.filter(artist => {
      const firstChar = artist.name.charAt(0).toUpperCase()
      if (letterFilter.value === '#') {
        return !/[A-Z]/.test(firstChar)
      }
      return firstChar === letterFilter.value
    })
  })

  const artistRows = computed(() => {
    const rows = []
    const items = displayedArtists.value
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

  watch([artistRows, viewLayout, cols, scrollElement, letterFilter], () => {
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
  <section class='px-4 md:px-6 lg:px-8 pt-4 md:pt-6 pb-8 max-w-7xl mx-auto'>
    <!-- Header section with stats -->
    <div v-if='!libraryLoading' class='mb-6 space-y-4'>
      <LibraryStats />

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
        <Skeleton class='w-full aspect-square rounded-full' />
        <div class='flex flex-col items-center gap-1'>
          <Skeleton :class='viewLayout === "compact" ? "h-3 w-3/4" : "h-4 w-3/4"' />
          <Skeleton :class='viewLayout === "compact" ? "h-2.5 w-1/2" : "h-3 w-1/2"' />
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
          <ArtistCard
            v-for='artist in artists'
            @click='selectArtist(artist)'
            @shuffle='playArtistShuffle'
            :key='artist.id'
            :artist='artist'
            :compact='viewLayout === "compact"'
            :is-scrolling='isScrolling'
            :server-url='serverUrl'
            :token='token'
            :width='itemWidth'
          />
        </div>
      </div>
    </div>

    <p
      v-if='!libraryLoading && displayedArtists.length === 0'
      class='text-center py-12 text-muted-foreground'
    >
      No artists found{{ letterFilter ? ` starting with "${letterFilter}"` : '' }}
    </p>
  </section>
</template>