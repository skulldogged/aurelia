<script setup lang="ts">
  import { Database, Disc, Home, Library, Link, Music, Palette, Search, Server, Users } from 'lucide-vue-next'
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
  import { useAuthStore, useLibraryStore } from '@/stores'

  // --- Props & Emits ---
  const props = defineProps<{
    open: boolean
  }>()

  const emit = defineEmits<{
    (e: 'update:open', value: boolean): void
  }>()

  // --- State & Stores ---
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

  // Use our own search term for now
  const searchTerm = ref('')

  // Initial logging
  console.log('GlobalSearch: Component mounted')
  console.log('GlobalSearch: Initial data - allSongs:', allSongs.value.length)
  console.log('GlobalSearch: Initial data - allArtists:', allArtistsWithSongs.value.length)
  console.log('GlobalSearch: Initial data - allAlbums:', allAlbums.value.length)

  // --- Search & Filtering ---
  const filteredSongs = computed(() => {
    if (!searchTerm.value) return []
    const lowerCaseQuery = searchTerm.value.toLowerCase()
    const results = allSongs.value
      .filter(song =>
        song.name.toLowerCase().includes(lowerCaseQuery) ||
        song.artists?.join(' ').toLowerCase().includes(lowerCaseQuery) ||
        song.album?.toLowerCase().includes(lowerCaseQuery),
      )
      .slice(0, 15)
    console.log('GlobalSearch: filteredSongs computed:', results.length, 'for query:', lowerCaseQuery)
    return results
  })

  const filteredArtists = computed(() => {
    if (!searchTerm.value) return []
    const lowerCaseQuery = searchTerm.value.toLowerCase()
    const results = allArtistsWithSongs.value
      .filter(artist =>
        artist.name.toLowerCase().includes(lowerCaseQuery),
      )
      .slice(0, 5)
    console.log('GlobalSearch: filteredArtists computed:', results.length, 'for query:', lowerCaseQuery)
    return results
  })

  const filteredAlbums = computed(() => {
    if (!searchTerm.value) return []
    const lowerCaseQuery = searchTerm.value.toLowerCase()
    const results = allAlbums.value
      .filter(album =>
        album.name.toLowerCase().includes(lowerCaseQuery) ||
        album.artist?.toLowerCase().includes(lowerCaseQuery),
      )
      .slice(0, 5)
    console.log('GlobalSearch: filteredAlbums computed:', results.length, 'for query:', lowerCaseQuery)
    return results
  })

  // Debug logging
  watch(searchTerm, newTerm => {
    console.log('GlobalSearch: searchTerm changed:', newTerm)
    console.log('GlobalSearch: allSongs count:', allSongs.value.length)
    console.log('GlobalSearch: allArtists count:', allArtistsWithSongs.value.length)
    console.log('GlobalSearch: allAlbums count:', allAlbums.value.length)
    console.log('GlobalSearch: filteredSongs count:', filteredSongs.value.length)
    console.log('GlobalSearch: filteredArtists count:', filteredArtists.value.length)
    console.log('GlobalSearch: filteredAlbums count:', filteredAlbums.value.length)
  })

  watch(() => props.open, isOpen => {
    console.log('GlobalSearch: dialog open state changed:', isOpen)
    if (!isOpen) {
      searchTerm.value = ''
    }
  })

  // --- Actions ---
  const closeDialog = (): void => {
    emit('update:open', false)
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
      router.push({ name: 'album-detail', params: { albumName: album.name } })
    closeDialog()
  }
</script>

<template>
  <CommandDialog @update:open="emit('update:open', $event)" :open='props.open'>
    <DialogTitle class='sr-only'>
      Global Search
    </DialogTitle>
    <DialogDescription class='sr-only'>
      Search for songs, artists, and albums across your entire library.
    </DialogDescription>
    <div class='flex items-center border-b border-border/50 px-3' cmdk-input-wrapper>
      <Search class='mr-2 h-5 w-5 shrink-0 opacity-50' />
      <input
        v-model='searchTerm'
        class='flex h-12 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground'
        placeholder='Search for songs, artists, or albums...'
        auto-focus
      >
    </div>
    <CommandList use-overlay-scrollbar>
      <CommandEmpty v-if='!searchTerm'>
        Start typing to search your library.
      </CommandEmpty>
      <CommandEmpty v-else>
        No results found for "{{ searchTerm }}".
      </CommandEmpty>

      <!-- Navigation Commands -->
      <CommandGroup v-if='!searchTerm' heading='Navigation'>
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
      <CommandGroup v-if='!searchTerm' heading='Settings'>
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
