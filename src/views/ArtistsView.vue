<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { Shuffle } from 'lucide-vue-next'
  import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Artist, Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import { Button } from '@/components/ui/button'
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
  import { usePagination } from '@/composables/useLayoutPreference'

  const router = useRouter()

  const showAllArtists = ref(false)

  const props = defineProps<{
    allArtists:     Artist[],
    libraryLoaded:  boolean,
    libraryLoading: boolean,
    serverUrl:      string,
    token:          string,
  }>()

  const emit = defineEmits<{
    'play-song':     [song: Song]
    'play-songs':    [songs: Song[]]
    'select-artist': [artist: Artist]
  }>()

  const searchQuery = ref('')
  const showSkeleton = ref(false) // Temporary dev toggle for adjusting skeleton sizes

  // Artists who appear as an "album artist" on at least one song
  const albumArtists = computed(() => props.allArtists.filter(artist =>
    artist.songs?.some(song =>
      song.albumArtists?.some(albumArtist => albumArtist.id === artist.id),
    ),
  ))

  const artistsToDisplay = computed(() =>
    showAllArtists.value ? props.allArtists : (albumArtists.value?.length ? albumArtists.value : props.allArtists),
  )

  // Use artists directly from props, sorted alphabetically (case-insensitive)
  const artistsWithSongs = computed(() =>
    [...artistsToDisplay.value].sort((a, b) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
    ),
  )

  // Fuzzy search setup (Fuse.js)
  const artistsFuse = ref(new Fuse(artistsWithSongs.value, {
    includeScore:       true,
    keys:               ['name'],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  watch(artistsWithSongs, newArtists => {
    artistsFuse.value.setCollection(newArtists)
  })

  const filteredArtists = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2) return artistsWithSongs.value
    return artistsFuse.value.search(searchQuery.value).map(result => result.item)
  })

  // Pagination
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
    pageSize,
    pageSizeOptions,
    setPageSize,
    total,
  } = usePagination(filteredArtists, 'artists-pagesize', 20)

  const toggleArtistMode = (): void => {
    showAllArtists.value = !showAllArtists.value
  }

  const playArtistShuffle = (artist: Artist): void => {
    const artistSongs = artist.songs
    if (artistSongs && artistSongs.length > 0) {
      // Shuffle the songs
      const shuffledSongs = [...artistSongs].sort(() => 0.5 - Math.random())
      emit('play-songs', shuffledSongs)
    }
  }

  const selectArtist = (artist: Artist): void => {
    if (artist.id)
      router.push(`/songs/artist/${artist.id}`)
  }
</script>

<template>
  <div class='p-8 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <div class='flex justify-between items-start mb-4'>
        <h1 class='text-4xl font-bold'>
          Artists
        </h1>
        <Button @click='toggleArtistMode'>
          {{ showAllArtists ? "Album Artists Only" : "All Artists" }}
        </Button>
      </div>
      <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center'>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search artists...'
          type='text'
        />
        <!-- Dev toggle for skeleton adjustment -->
        <Button
          @click='showSkeleton = !showSkeleton'
          :variant='showSkeleton ? "default" : "outline"'
          size='sm'
        >
          {{ showSkeleton ? 'Hide' : 'Show' }} Skeleton (dev)
        </Button>
      </div>
    </div>

    <div
      v-if='libraryLoading || showSkeleton'
      class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'
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
          <Skeleton class='h-6 w-3/4' />
          <!-- Song count skeleton -->
          <Skeleton class='h-4 w-1/2' />
        </div>
      </div>
    </div>
    <div v-else class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <div
        v-for='artist in pagedArtists'
        @click='selectArtist(artist)'
        :key='artist.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-4'>
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
              class='
                bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white
                border border-white/20
              '
              size='icon'
            >
              <Shuffle class='h-4 w-4' />
            </Button>
          </div>
        </div>

        <div class='text-center'>
          <p class='font-semibold truncate'>
            {{ artist.name }}
          </p>
          <p v-if='artist.songs' class='text-sm text-muted-foreground truncate'>
            {{ artist.songs.length }} songs
          </p>
        </div>
      </div>
    </div>

    <div
      v-if='!libraryLoading && !showSkeleton && filteredArtists && filteredArtists.length === 0'
      class='text-center py-12'
    >
      <p class='text-muted-foreground'>
        No artists found
      </p>
    </div>

    <!-- Pagination Controls -->
    <div v-if='pageCount > 1' class='flex items-center justify-between border-t border-border pt-6 mt-8'>
      <div class='flex items-center gap-2'>
        <span class='text-sm text-muted-foreground'>Artists per page:</span>
        <Select @update:model-value='(v) => setPageSize(Number(v))' :model-value='String(pageSize)'>
          <SelectTrigger class='w-20'>
            <SelectValue placeholder='Per page' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem v-for='option in pageSizeOptions' :key='option' :value='String(option)'>
                {{ option }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <div class='flex items-center gap-2'>
        <span class='text-sm text-muted-foreground'>
          Page {{ pageIndex + 1 }} of {{ pageCount }} ({{ total }} total)
        </span>
      </div>

      <div class='flex items-center gap-1'>
        <Button
          @click='goToFirstPage'
          :disabled='!canPreviousPage'
          size='sm'
          variant='outline'
        >
          <ChevronsLeft class='h-4 w-4' />
        </Button>
        <Button
          @click='goToPreviousPage'
          :disabled='!canPreviousPage'
          size='sm'
          variant='outline'
        >
          <ChevronLeft class='h-4 w-4' />
        </Button>
        <Button
          @click='goToNextPage'
          :disabled='!canNextPage'
          size='sm'
          variant='outline'
        >
          <ChevronRight class='h-4 w-4' />
        </Button>
        <Button
          @click='goToLastPage'
          :disabled='!canNextPage'
          size='sm'
          variant='outline'
        >
          <ChevronsRight class='h-4 w-4' />
        </Button>
      </div>
    </div>
  </div>
</template>
