<script setup lang="ts">
  import { useDebounce } from '@vueuse/core'
  import {
    Database, Disc, Home, Library, Link, Loader2, Music, Palette, Search, Server, Users, X,
  } from 'lucide-vue-next'
  import { ListboxFilter } from 'reka-ui'
  import { storeToRefs } from 'pinia'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import type { Album, Artist, Song } from '@/bindings'

  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandItem,
    CommandList,
  } from '@/components/ui/command'
  import { DialogDescription, DialogTitle } from '@/components/ui/dialog'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { logger } from '@/lib/logger'
  import { useAuthStore } from '@/stores'
  import { useLibraryStore } from '@/stores/library'

  const props = defineProps<{
    open: boolean
  }>()

  const emit = defineEmits<{
    (e: 'update:open', value: boolean): void
  }>()

  const router = useRouter()
  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  const { serverUrl, token, userId, username } = storeToRefs(authStore)
  const { allAlbums, allArtistsWithSongs, allSongs } = storeToRefs(libraryStore)

  const credentials = computed(() => ({
    serverUrl: serverUrl.value,
    token:     token.value,
    userId:    userId.value,
    username:  username.value,
  }))

  const { playSong } = useSongInteractions(computed(() => credentials.value))

  const searchTerm = ref('')

  const debouncedSearchTerm = useDebounce(searchTerm, 300)

  const isSearching = computed(() => searchTerm.value !== debouncedSearchTerm.value)

  const hasSearchResults = computed(() =>
    filteredSongs.value.length > 0 ||
    filteredArtists.value.length > 0 ||
    filteredAlbums.value.length > 0,
  )

  logger.info('Component mounted')
  logger.debug('Initial data - allSongs:', allSongs.value.length)
  logger.debug('Initial data - allArtists:', allArtistsWithSongs.value.length)
  logger.debug('Initial data - allAlbums:', allAlbums.value.length)

  const filteredSongs = computed(() => {
    if (!debouncedSearchTerm.value) return []
    const lowerCaseQuery = debouncedSearchTerm.value.toLowerCase()

    return allSongs.value
      .filter(song => song.name.toLowerCase().includes(lowerCaseQuery) ||
        song.artists?.join(' ').toLowerCase().includes(lowerCaseQuery) ||
        song.album?.toLowerCase().includes(lowerCaseQuery),
      )
      .slice(0, 15)
  })

  const filteredArtists = computed(() => {
    if (!debouncedSearchTerm.value) return []
    const lowerCaseQuery = debouncedSearchTerm.value.toLowerCase()

    return allArtistsWithSongs.value
      .filter(artist => artist.name.toLowerCase().includes(lowerCaseQuery),
      )
      .slice(0, 5)
  })

  const filteredAlbums = computed(() => {
    if (!debouncedSearchTerm.value) return []
    const lowerCaseQuery = debouncedSearchTerm.value.toLowerCase()

    return allAlbums.value
      .filter(album => album.name.toLowerCase().includes(lowerCaseQuery) ||
        album.artist?.toLowerCase().includes(lowerCaseQuery),
      )
      .slice(0, 5)
  })

  watch(searchTerm, (newTerm, oldTerm) => {
    if (newTerm !== oldTerm)
      logger.debug('searchTerm changed:', newTerm)
  })

  watch(debouncedSearchTerm, newTerm => {
    if (newTerm) {
      logger.debug('debounced search triggered:', newTerm)
      logger.debug('results - songs:', filteredSongs.value.length,
                   'artists:', filteredArtists.value.length,
                   'albums:', filteredAlbums.value.length)
    }
  })

  watch(() => props.open, async isOpen => {
    logger.debug('dialog open state changed:', isOpen)

    if (!isOpen)
      searchTerm.value = ''
  })

  const closeDialog = (): void => {
    emit('update:open', false)
  }

  const handleCloseClick = (): void => {
    if (!isSearching.value)
      closeDialog()
  }

  const handleSelectSong = (song: Song): void => {
    playSong(song)
    closeDialog()
  }

  const handleSelectArtist = (artist: Artist): void => {
    if (artist.id)
      router.push(`/artists/${artist.id}`)
    closeDialog()
  }

  const handleSelectAlbum = (album: Album): void => {
    if (album.id)
      router.push(`/albums/${album.id}`)
    closeDialog()
  }

  const handleKeydown = (event: KeyboardEvent): void => {
    if (event.key === 'Escape') {
      closeDialog()
    }
  }
</script>

<template>
  <CommandDialog @update:open="emit('update:open', $event)" :hide-close-button='true' :open='props.open'>
    <DialogTitle class='sr-only'>
      Global Search
    </DialogTitle>
    <DialogDescription class='sr-only'>
      Search for songs, artists, and albums across your entire library.
    </DialogDescription>

    <!-- Enhanced search input area -->
    <div class='relative flex items-center gap-3 border-b border-border/40 bg-muted/20 px-4' cmdk-input-wrapper>
      <Search class='size-5 shrink-0 text-muted-foreground/70' />
      <ListboxFilter
        v-model='searchTerm'
        @keydown='handleKeydown'
        class='flex h-14 w-full bg-transparent py-3 text-base outline-none placeholder:text-muted-foreground/60'
        placeholder='Search songs, artists, or albums...'
        auto-focus
      />
      <!-- Integrated close/loading button -->
      <button
        @click='handleCloseClick'
        :disabled='isSearching'
        class='flex size-7 items-center justify-center rounded-md text-muted-foreground/70
               transition-colors hover:bg-muted/50 hover:text-foreground
               disabled:pointer-events-none focus:outline-none'
      >
        <Loader2 v-if='isSearching' class='size-4 animate-spin' />
        <X v-else class='size-4' />
        <span class='sr-only'>{{ isSearching ? 'Searching...' : 'Close' }}</span>
      </button>
    </div>

    <CommandList use-overlay-scrollbar>
      <!-- Enhanced empty state -->
      <CommandEmpty v-if='!debouncedSearchTerm'>
        <div class='flex flex-col items-center gap-3 py-8 text-center'>
          <div class='flex size-12 items-center justify-center rounded-full bg-muted/30'>
            <Search class='size-5 text-muted-foreground/60' />
          </div>
          <div class='space-y-1'>
            <p class='text-sm font-medium text-foreground/80'>Search your library</p>
            <p class='text-xs text-muted-foreground'>Find songs, artists, and albums</p>
          </div>
        </div>
      </CommandEmpty>

      <!-- Enhanced no results message -->
      <div v-else-if='debouncedSearchTerm && !hasSearchResults' class='flex flex-col items-center gap-3 py-8 text-center'>
        <div class='flex size-12 items-center justify-center rounded-full bg-muted/30'>
          <Search class='size-5 text-muted-foreground/60' />
        </div>
        <div class='space-y-1'>
          <p class='text-sm font-medium text-foreground/80'>No results found</p>
          <p class='max-w-[200px] text-xs text-muted-foreground'>
            No matches for "<span class='text-foreground/70'>{{ debouncedSearchTerm }}</span>"
          </p>
        </div>
      </div>

      <!-- Navigation Commands -->
      <CommandGroup v-if='!debouncedSearchTerm' heading='Navigation'>
        <CommandItem @select='() => { router.push("/"); closeDialog() }' value='home'>
          <Home class='size-4' />
          <span>Home</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/songs"); closeDialog() }' value='songs'>
          <Music class='size-4' />
          <span>Songs</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/artists"); closeDialog() }' value='artists'>
          <Users class='size-4' />
          <span>Artists</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/albums"); closeDialog() }' value='albums'>
          <Disc class='size-4' />
          <span>Albums</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/playlists"); closeDialog() }' value='playlists'>
          <Library class='size-4' />
          <span>Playlists</span>
        </CommandItem>
      </CommandGroup>

      <!-- Settings Tabs -->
      <CommandGroup v-if='!debouncedSearchTerm' heading='Settings'>
        <CommandItem
          @select='() => { router.push("/settings?tab=appearance"); closeDialog() }'
          value='settings-appearance'
        >
          <Palette class='size-4' />
          <span>Appearance</span>
        </CommandItem>
        <CommandItem
          @select='() => { router.push("/settings?tab=integrations"); closeDialog() }'
          value='settings-integrations'
        >
          <Link class='size-4' />
          <span>Integrations</span>
        </CommandItem>
        <CommandItem
          @select='() => { router.push("/settings?tab=server"); closeDialog() }'
          value='settings-server'
        >
          <Server class='size-4' />
          <span>Server</span>
        </CommandItem>
        <CommandItem
          @select='() => { router.push("/settings?tab=library"); closeDialog() }'
          value='settings-library'
        >
          <Database class='size-4' />
          <span>Library</span>
        </CommandItem>
      </CommandGroup>

      <!-- Artists -->
      <CommandGroup v-if='filteredArtists.length > 0' heading='Artists'>
        <CommandItem
          v-for='artist in filteredArtists'
          @select='() => handleSelectArtist(artist as Artist)'
          :key='`artist-${artist.id}`'
          :value='`artist-${artist.name}`'
          class='gap-3'
        >
          <ImageLoader
            :alt='artist.name'
            :item-id='artist.id'
            :server-url='credentials.serverUrl'
            :token='credentials.token'
            class='size-10 rounded-full ring-1 ring-border/30'
          >
            <template #fallback>
              <ImagePlaceholder class='size-10' type='artist' />
            </template>
          </ImageLoader>
          <span class='truncate font-medium'>{{ artist.name }}</span>
        </CommandItem>
      </CommandGroup>

      <!-- Albums -->
      <CommandGroup v-if='filteredAlbums.length > 0' heading='Albums'>
        <CommandItem
          v-for='album in filteredAlbums'
          @select='() => handleSelectAlbum(album as Album)'
          :key='`album-${album.id}`'
          :value='`album-${album.name}`'
          class='gap-3'
        >
          <ImageLoader
            :alt='album.name'
            :item-id='album.id || undefined'
            :server-url='credentials.serverUrl'
            :token='credentials.token'
            class='size-10 rounded-md ring-1 ring-border/30'
          >
            <template #fallback>
              <ImagePlaceholder class='size-10' type='album' />
            </template>
          </ImageLoader>
          <div class='min-w-0 flex-1'>
            <p class='truncate font-medium'>
              {{ album.name }}
            </p>
            <p class='truncate text-xs text-muted-foreground'>
              {{ album.artist }}
            </p>
          </div>
        </CommandItem>
      </CommandGroup>

      <!-- Songs -->
      <CommandGroup v-if='filteredSongs.length > 0' heading='Songs'>
        <CommandItem
          v-for='song in filteredSongs'
          @select='() => handleSelectSong(song)'
          :key='`song-${song.id}`'
          :value='`song-${song.name}`'
          class='gap-3'
        >
          <ImageLoader
            :alt='song.album || "Unknown album"'
            :item-id='song.albumId || undefined'
            :server-url='credentials.serverUrl'
            :token='credentials.token'
            class='size-10 rounded-md ring-1 ring-border/30'
          >
            <template #fallback>
              <ImagePlaceholder class='size-10' type='album' />
            </template>
          </ImageLoader>
          <div class='min-w-0 flex-1'>
            <p class='truncate font-medium'>
              {{ song.name }}
            </p>
            <p class='truncate text-xs text-muted-foreground'>
              {{ song.artists?.join(', ') }}
            </p>
          </div>
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </CommandDialog>
</template>
