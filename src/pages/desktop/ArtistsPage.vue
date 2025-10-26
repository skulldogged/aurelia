<script setup lang="ts">
  import { useMediaQuery } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { Shuffle } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Artist, Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
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
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const router = useRouter()

  const artistMode = ref<'album' | 'all'>('album')
  const { layout: viewLayout } = useLayoutPreference('artists-layout', 'comfy')

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

  const emit = defineEmits<{
    'play-song':     [song: Song]
    'play-songs':    [songs: Song[]]
    'select-artist': [artist: Artist]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  // Create computed properties from stores
  const allArtists = computed(() => libraryStore.allArtistsWithSongs as Artist[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const searchQuery = ref('')

  // Artists who appear as an "album artist" on at least one song
  const albumArtists = computed(() => allArtists.value.filter(artist =>
    artist.songs?.some(song =>
      song.albumArtists?.some(albumArtist => albumArtist.id === artist.id),
    ),
  ))

  const artistsToDisplay = computed(() => {
    const mode = artistMode.value
    return mode === 'all' ? allArtists.value : (albumArtists.value?.length ? albumArtists.value : allArtists.value)
  })

  // Deduplicate artists by name (not ID) to handle Jellyfin duplicate artist entries
  // For duplicates, keep the entry with the most songs
  const artistsWithSongs = computed(() => {
    const uniqueArtistsByName = new Map<string, Artist>()

    for (const artist of artistsToDisplay.value) {
      const normalizedName = artist.name.toLowerCase()
      const existing = uniqueArtistsByName.get(normalizedName)

      // Keep the artist with more songs, or the first one if equal
      if (!existing || (artist.songs?.length || 0) > (existing.songs?.length || 0)) {
        uniqueArtistsByName.set(normalizedName, artist)
      }
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
    searchQuery.value && searchQuery.value.length >= 2
      ? artistsFuse.value.search(searchQuery.value).map(result => result.item)
      : artistsWithSongs.value,
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
      ? 'artists-rows-compact'
      : 'artists-rows-comfy',
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
    pagedItems: pagedArtists,
    pageIndex,
  } = usePagination(filteredArtists, 'artists-pagesize', itemsPerPage, [])

  const setRowsPerPage = (rows: number): void => {
    rowsPerPage.value = rows
  }

  const playArtistShuffle = (artist: Artist): void => {
    const artistSongs = artist.songs

    if (artistSongs && artistSongs.length > 0)
      emit('play-songs', [...artistSongs].sort(() => 0.5 - Math.random()))
  }

  const selectArtist = (artist: Artist): void => {
    if (artist.id)
      router.push(`/artists/${artist.id}`)
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <div class='flex justify-between items-start mb-4'>
        <h1 class='text-4xl font-bold'>
          Artists
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
      <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between'>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search artists...'
          type='text'
        />

        <!-- Artist Mode Tabs -->
        <Tabs v-model='artistMode'>
          <TabsList>
            <TabsTrigger value='album'>
              Album Artists
            </TabsTrigger>
            <TabsTrigger value='all'>
              All Artists
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>
    </div>

    <div class='bg-sidebar rounded-lg p-6'>
      <div
        v-if='libraryLoading'
        :class='viewLayout === "compact"
          ? "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-4"
          : "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6"'
      >
        <!-- Skeleton loading grid -->
        <div
          v-for='n in 20'
          :key='`skeleton-${n}`'
          class='flex flex-col gap-4'
        >
          <!-- Artist image skeleton -->
          <Skeleton class='w-full aspect-square rounded-lg' />
          <!-- Text content skeleton -->
          <div class='flex flex-col items-center gap-1'>
            <!-- Artist name skeleton -->
            <Skeleton :class='viewLayout === "compact" ? "h-4 w-3/4" : "h-6 w-3/4"' />
            <!-- Song count skeleton -->
            <Skeleton :class='viewLayout === "compact" ? "h-3 w-1/2" : "h-4 w-1/2"' />
          </div>
        </div>
      </div>
      <div
        v-else
        :class='viewLayout === "compact"
          ? "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-4"
          : "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6"'
      >
        <div
          v-for='artist in pagedArtists'
          @click='selectArtist(artist)'
          :key='artist.id'
          class='cursor-pointer group'
        >
          <div :class='viewLayout === "compact" ? "relative mb-2" : "relative mb-4"'>
            <ImageLoader
              :alt='`${artist.name} artist image`'
              :item-id='artist.id'
              :server-url='serverUrl'
              :token='token'
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

            <!-- Play button overlay -->
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
            <p
              v-if='artist.songs'
              :class='viewLayout === "compact"
                ? "text-xs text-muted-foreground truncate"
                : "text-sm text-muted-foreground truncate"'
            >
              {{ artist.songs.length }} songs
            </p>
          </div>
        </div>
      </div>

      <div
        v-if='!libraryLoading && filteredArtists && filteredArtists.length === 0'
        class='text-center py-12'
      >
        <p class='text-muted-foreground'>
          No artists found
        </p>
      </div>

      <!-- Pagination Controls -->
      <div v-if='pageCount > 1' class='flex flex-col sm:flex-row items-center justify-between gap-4 mt-6'>
        <div class='flex items-center gap-2'>
          <span class='text-sm text-muted-foreground'>Rows per page:</span>
          <Select @update:model-value='(v) => setRowsPerPage(Number(v))' :model-value='String(rowsPerPage)'>
            <SelectTrigger class='w-20'>
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