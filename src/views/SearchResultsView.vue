<script setup lang="ts">
  import { computed, ref, onMounted, onUnmounted } from 'vue'
  import Fuse, { FuseResult } from 'fuse.js'
  import { AlbumWithSongs, ArtistInfo, MusicItem } from '../types'
  import { useRouter } from 'vue-router'
  import { ScrollArea } from '@/components/ui/scroll-area'

  const props = defineProps<{
    query:     string
    songs:     MusicItem[]
    albums:    AlbumWithSongs[]
    artists:   ArtistInfo[]
    isVisible: boolean
  }>()

  const emit = defineEmits<{
    'close':          []
    'select-album':   [album: AlbumWithSongs]
    'select-artist':  [artist: ArtistInfo]
    'play-song':      [song: MusicItem]
    'result-clicked': []
  }>()

  const router = useRouter()
  const searchResultsRef = ref<HTMLElement | null>(null)

  const handleClickOutside = (event: Event) => {
    const target = event.target as HTMLElement
    if (searchResultsRef.value && !searchResultsRef.value.contains(target)) {
      // Check if we're not clicking on the search input
      const searchInput = document.querySelector('input[placeholder="Search music..."]')
      if (searchInput && !searchInput.contains(target)) {
        emit('close')
      }
    }
  }

  onMounted(() => {
    document.addEventListener('click', handleClickOutside)
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
  })

  const fuseOptions = {
    includeScore:       true,
    threshold:          0.2,
    minMatchCharLength: 2,
  }

  const combinedData = computed(() => {
    const songs = props.songs.map(item => ({ type: 'song' as const, item }))
    const albums = props.albums.map(item => ({ type: 'album' as const, item }))
    const artists = props.artists.map(item => ({ type: 'artist' as const, item }))
    return [...songs, ...albums, ...artists]
  })

  const fuse = computed(() => new Fuse(combinedData.value, {
    ...fuseOptions,
    keys: [
      { name: 'item.name', weight: 0.6 },
      { name: 'item.Name', weight: 0.7 },
      { name: 'item.artists', weight: 0.3 },
      { name: 'item.artist', weight: 0.4 },
      { name: 'item.album', weight: 0.2 },
    ],
  }))

  const searchResults = computed(() => {
    if (!props.query || props.query.length < 2) return []
    return fuse.value.search(props.query)
  })

  type SearchResultItem =
    | { type: 'song', item: MusicItem }
    | { type: 'album', item: AlbumWithSongs }
    | { type: 'artist', item: ArtistInfo }

  const categorizedResults = computed(() => {
    const results: {
      songs:   FuseResult<SearchResultItem>[]
      albums:  FuseResult<SearchResultItem>[]
      artists: FuseResult<SearchResultItem>[]
    } = {
      songs:   [],
      albums:  [],
      artists: [],
    }

    for (const result of searchResults.value) {
      if (result.item.type === 'song' && results.songs.length < 5)
        results.songs.push(result)
      else if (result.item.type === 'album' && results.albums.length < 5)
        results.albums.push(result)
      else if (result.item.type === 'artist')
        results.artists.push(result)
    }
    return results
  })

  const filteredSongs = computed(() => categorizedResults.value.songs.map(r => r.item.item as MusicItem))
  const filteredAlbums = computed(() => categorizedResults.value.albums.map(r => r.item.item as AlbumWithSongs))
  const filteredArtists = computed(() => categorizedResults.value.artists.map(r => r.item.item as ArtistInfo))

  const resultOrder = computed(() => {
    const topScores: { [key: string]: number | undefined } = {
      songs:   categorizedResults.value.songs[0]?.score,
      albums:  categorizedResults.value.albums[0]?.score,
      artists: categorizedResults.value.artists[0]?.score,
    }

    const sections: Array<'songs' | 'albums' | 'artists'> = ['songs', 'albums', 'artists']

    sections.sort((a, b) => {
      const scoreA = topScores[a] ?? 1
      const scoreB = topScores[b] ?? 1
      return scoreA - scoreB
    })

    return sections
  })

  const hasResults = computed(() => {
    return searchResults.value.length > 0
  })

  const selectSong = (song: MusicItem) => {
    emit('play-song', song)
    emit('close')
    emit('result-clicked')
  }

  const selectAlbum = (album: AlbumWithSongs) => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
    emit('close')
    emit('result-clicked')
  }

  const selectArtist = (artist: ArtistInfo) => {
    router.push(`/songs/artist/${artist.Id}`)
    emit('close')
    emit('result-clicked')
  }
</script>

<template>
  <div
    v-if='isVisible && query'
    ref='searchResultsRef'
    class='absolute top-14 left-1/2 -translate-x-1/2 w-96 bg-background border border-border rounded-md shadow-lg z-50'
  >
    <ScrollArea class='h-[400px]'>
      <div v-if='hasResults' class='p-2'>
        <template v-for='(section, index) in resultOrder' :key='section'>
          <div v-if="section === 'songs' && filteredSongs.length > 0" :class="{ 'mt-2': index > 0 }">
            <h3 class='text-sm font-semibold text-muted-foreground px-2 py-1.5'>
              Songs
            </h3>
            <ul>
              <li
                v-for='song in filteredSongs'
                @click='selectSong(song)'
                :key='song.id'
                class='flex items-center p-2 rounded-md hover:bg-accent cursor-pointer'
              >
                <img :src='song.albumArtUrl' class='w-10 h-10 rounded-md mr-3'>
                <div>
                  <p class='font-semibold'>
                    {{ song.name }}
                  </p>
                  <p class='text-sm text-muted-foreground'>
                    {{ song.artists?.join(', ') }}
                  </p>
                </div>
              </li>
            </ul>
          </div>
          <div v-if="section === 'albums' && filteredAlbums.length > 0" :class="{ 'mt-2': index > 0 }">
            <h3 class='text-sm font-semibold text-muted-foreground px-2 py-1.5'>
              Albums
            </h3>
            <ul>
              <li
                v-for='album in filteredAlbums'
                @click='selectAlbum(album)'
                :key='album.name'
                class='flex items-center p-2 rounded-md hover:bg-accent cursor-pointer'
              >
                <img :src='album.albumArtUrl' class='w-10 h-10 rounded-md mr-3'>
                <div>
                  <p class='font-semibold'>
                    {{ album.name }}
                  </p>
                  <p class='text-sm text-muted-foreground'>
                    {{ album.artist }}
                  </p>
                </div>
              </li>
            </ul>
          </div>
          <div v-if="section === 'artists' && filteredArtists.length > 0" :class="{ 'mt-2': index > 0 }">
            <h3 class='text-sm font-semibold text-muted-foreground px-2 py-1.5'>
              Artists
            </h3>
            <ul>
              <li
                v-for='artist in filteredArtists'
                @click='selectArtist(artist)'
                :key='artist.Id'
                class='flex items-center p-2 rounded-md hover:bg-accent cursor-pointer'
              >
                <div class='w-10 h-10 rounded-full bg-muted flex items-center justify-center mr-3'>
                  <span class='text-lg font-bold'>{{ artist.Name.charAt(0) }}</span>
                </div>
                <p class='font-semibold'>
                  {{ artist.Name }}
                </p>
              </li>
            </ul>
          </div>
        </template>
      </div>
      <div v-else class='p-4 text-center text-muted-foreground'>
        No results found for "{{ query }}"
      </div>
    </ScrollArea>
  </div>
</template>
