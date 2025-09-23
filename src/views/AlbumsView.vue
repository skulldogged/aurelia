<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'
  import { Song, Album } from '@/bindings'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { Play } from 'lucide-vue-next'
  import Fuse from 'fuse.js'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { usePagination } from '@/composables/useLayoutPreference'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-vue-next'

  const router = useRouter()

  const props = defineProps<{
    serverUrl:      string,
    token:          string,
    libraryLoaded:  boolean,
    libraryLoading: boolean,
    allAlbums:      Album[],
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()

  const searchQuery = ref('')
  const showSkeleton = ref(false) // Temporary dev toggle for adjusting skeleton sizes

  const albumsFuse = ref(new Fuse(props.allAlbums, {
    keys: [
      { name: 'name', weight: 0.6 },
      { name: 'artist', weight: 0.4 },
    ],
    includeScore:       true,
    threshold:          0.2,
    minMatchCharLength: 2,
  }))

  watch(() => props.allAlbums, newAlbums => {
    albumsFuse.value.setCollection(newAlbums)
  })

  const filteredAlbums = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2)
      return [...props.allAlbums].sort((a, b) =>
        a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
      )

    return albumsFuse.value.search(searchQuery.value).map(result => result.item)
  })

  // Pagination
  const {
    pagedItems: pagedAlbums,
    pageIndex,
    pageSize,
    total,
    pageCount,
    canPreviousPage,
    canNextPage,
    goToPreviousPage,
    goToNextPage,
    goToFirstPage,
    goToLastPage,
    setPageSize,
    pageSizeOptions,
  } = usePagination(filteredAlbums, 'albums-pagesize', 20)

  const playAlbum = (album: Album) => {
    if (album.songs && album.songs.length > 0) {
      const sortedSongs = [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
      emit('play-songs', sortedSongs)
    }
  }

  const selectAlbum = (album: Album) => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
  }
</script>

<template>
  <div class='p-8 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <h1 class='text-4xl font-bold mb-4'>
        Albums
      </h1>
      <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center'>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search albums...'
          type='text'
        />
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
      <div
        v-for='n in 20'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-4'
      >
        <Skeleton class='w-full aspect-square rounded-lg' name='album-art' />
        <div class='flex flex-col gap-1'>
          <Skeleton class='h-6 w-3/4' name='album-title' />
          <Skeleton class='h-4 w-20' name='artist' />
          <Skeleton class='h-4 w-16' name='song-count' />
        </div>
      </div>
    </div>
    <div v-else class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <div
        v-for='album in pagedAlbums'
        @click='selectAlbum(album)'
        :key='album.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-4'>
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
              class='
                bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                border-white/20
              '
              size='icon'
            >
              <Play class='h-4 w-4' />
            </Button>
          </div>
        </div>

        <div>
          <p class='font-semibold truncate'>
            {{ album.name }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            {{ album.artist }}
          </p>
          <p v-if='album.songs' class='text-sm text-muted-foreground truncate'>
            {{ album.songs.length }} songs
          </p>
        </div>
      </div>
    </div>

    <div v-if='!libraryLoading && !showSkeleton && filteredAlbums.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground'>
        No albums found
      </p>
    </div>

    <!-- Pagination Controls -->
    <div v-if='pageCount > 1' class='flex items-center justify-between border-t border-border pt-6 mt-8'>
      <div class='flex items-center gap-2'>
        <span class='text-sm text-muted-foreground'>Albums per page:</span>
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
