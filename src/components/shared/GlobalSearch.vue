<script setup lang="ts">
  import { useDebounce } from '@vueuse/core'
  import {
    Database, Disc, Home, Library, Link, Loader2, Music, Palette, Search, Server, Users, X,
  } from 'lucide-vue-next'
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
  import { searchLogger } from '@/lib/logger'
  import { useAuthStore, useLibraryStore } from '@/stores'

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
  const searchInput = ref<HTMLInputElement>()

  const debouncedSearchTerm = useDebounce(searchTerm, 300)

  const isSearching = computed(() => searchTerm.value !== debouncedSearchTerm.value)

  const hasSearchResults = computed(() =>
    filteredSongs.value.length > 0 ||
    filteredArtists.value.length > 0 ||
    filteredAlbums.value.length > 0,
  )

  searchLogger.info('Component mounted')
  searchLogger.debug('Initial data - allSongs:', allSongs.value.length)
  searchLogger.debug('Initial data - allArtists:', allArtistsWithSongs.value.length)
  searchLogger.debug('Initial data - allAlbums:', allAlbums.value.length)

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
      searchLogger.debug('searchTerm changed:', newTerm)
  })

  watch(debouncedSearchTerm, newTerm => {
    if (newTerm) {
      searchLogger.debug('debounced search triggered:', newTerm)
      searchLogger.debug('results - songs:', filteredSongs.value.length,
                         'artists:', filteredArtists.value.length,
                         'albums:', filteredAlbums.value.length)
    }
  })

  watch(() => props.open, async isOpen => {
    searchLogger.debug('dialog open state changed:', isOpen)

    if (isOpen)
      setTimeout(((): void => searchInput.value?.focus()), 5)
    else
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
      router.push({ name: 'artist-detail', params: { artistId: artist.id } })
    closeDialog()
  }

  const handleSelectAlbum = (album: Album): void => {
    if (album.id)
      router.push({ name: 'album-detail', params: { albumId: album.id } })
    closeDialog()
  }
</script>

<template>
  <CommandDialog @update:open="emit('update:open', $event)" :hide-close-button='true' :open='props.open'>
    <!-- Custom close button - spinner when searching, X when not -->
    <button
      @click='handleCloseClick'
      :disabled='isSearching'
      class='ring-offset-background focus:ring-ring data-[state=open]:bg-accent
             data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs
             opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2
             focus:outline-hidden disabled:pointer-events-none disabled:opacity-50 z-10'
    >
      <Loader2 v-if='isSearching' class='size-4 animate-spin' />
      <X v-else class='size-4' />
      <span class='sr-only'>{{ isSearching ? 'Searching...' : 'Close' }}</span>
    </button>

    <DialogTitle class='sr-only'>
      Global Search
    </DialogTitle>
    <DialogDescription class='sr-only'>
      Search for songs, artists, and albums across your entire library.
    </DialogDescription>
    <div class='flex items-center border-b border-border/50 px-3' cmdk-input-wrapper>
      <Search class='mr-2 h-5 w-5 shrink-0 opacity-50' />
      <input
        ref='searchInput'
        v-model='searchTerm'
        class='flex h-12 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground'
        placeholder='Search for songs, artists, or albums...'
        auto-focus
      >
    </div>
    <CommandList use-overlay-scrollbar>
      <CommandEmpty v-if='!debouncedSearchTerm'>
        Start typing to search your library.
      </CommandEmpty>

      <!-- No results message -->
      <div v-else-if='debouncedSearchTerm && !hasSearchResults' class='py-6 text-center text-sm text-muted-foreground'>
        No results found for "{{ debouncedSearchTerm }}".
      </div>

      <!-- Navigation Commands -->
      <CommandGroup v-if='!debouncedSearchTerm' heading='Navigation'>
        <CommandItem @select='() => { router.push("/"); closeDialog() }' value='home'>
          <Home class='h-5 w-5' />
          <span>Home</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/songs"); closeDialog() }' value='songs'>
          <Music class='h-5 w-5' />
          <span>Songs</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/artists"); closeDialog() }' value='artists'>
          <Users class='h-5 w-5' />
          <span>Artists</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/albums"); closeDialog() }' value='albums'>
          <Disc class='h-5 w-5' />
          <span>Albums</span>
        </CommandItem>
        <CommandItem @select='() => { router.push("/playlists"); closeDialog() }' value='playlists'>
          <Library class='h-5 w-5' />
          <span>Playlists</span>
        </CommandItem>
      </CommandGroup>

      <!-- Settings Tabs -->
      <CommandGroup v-if='!debouncedSearchTerm' heading='Settings'>
        <CommandItem
          @select='() => { router.push("/settings?tab=appearance"); closeDialog() }'
          value='settings-appearance'
        >
          <Palette class='h-5 w-5' />
          <span>Appearance</span>
        </CommandItem>
        <CommandItem
          @select='() => { router.push("/settings?tab=integrations"); closeDialog() }'
          value='settings-integrations'
        >
          <Link class='h-5 w-5' />
          <span>Integrations</span>
        </CommandItem>
        <CommandItem
          @select='() => { router.push("/settings?tab=server"); closeDialog() }'
          value='settings-server'
        >
          <Server class='h-5 w-5' />
          <span>Server</span>
        </CommandItem>
        <CommandItem
          @select='() => { router.push("/settings?tab=library"); closeDialog() }'
          value='settings-library'
        >
          <Database class='h-5 w-5' />
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
        >
          <ImageLoader
            :alt='artist.name'
            :item-id='artist.id'
            :server-url='credentials.serverUrl'
            :token='credentials.token'
            class='size-12 rounded-full'
          >
            <template #fallback>
              <ImagePlaceholder class='size-12' type='artist' />
            </template>
          </ImageLoader>
          <span class='truncate'>{{ artist.name }}</span>
        </CommandItem>
      </CommandGroup>

      <!-- Albums -->
      <CommandGroup v-if='filteredAlbums.length > 0' heading='Albums'>
        <CommandItem
          v-for='album in filteredAlbums'
          @select='() => handleSelectAlbum(album as Album)'
          :key='`album-${album.id}`'
          :value='`album-${album.name}`'
        >
          <ImageLoader
            :alt='album.name'
            :item-id='album.id || undefined'
            :server-url='credentials.serverUrl'
            :token='credentials.token'
            class='size-12 rounded-sm'
          >
            <template #fallback>
              <ImagePlaceholder class='size-12' type='album' />
            </template>
          </ImageLoader>
          <div class='flex-1 overflow-hidden'>
            <p class='truncate'>
              {{ album.name }}
            </p>
            <p class='text-xs text-muted-foreground truncate'>
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
        >
          <ImageLoader
            :alt='song.album || "Unknown album"'
            :item-id='song.albumId || undefined'
            :server-url='credentials.serverUrl'
            :token='credentials.token'
            class='size-12 rounded-sm'
          >
            <template #fallback>
              <ImagePlaceholder class='size-12' type='album' />
            </template>
          </ImageLoader>
          <div class='flex-1 overflow-hidden'>
            <p class='truncate'>
              {{ song.name }}
            </p>
            <p class='text-xs text-muted-foreground truncate'>
              {{ song.artists?.join(', ') }}
            </p>
          </div>
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </CommandDialog>
</template>
