import { ref, computed, watch } from 'vue'
import Fuse from 'fuse.js'
import type { Song, Album, Artist } from '@/bindings'

export const useSearch = (
  songs: Song[],
  albums: Album[],
  artists: Artist[],
  searchQuery: string,
) => {
  // Fuzzy search instances
  const songFuse = ref<Fuse<Song>>()
  const albumFuse = ref<Fuse<Album>>()
  const artistFuse = ref<Fuse<Artist>>()

  // Initialize search engines
  const initializeSearch = () => {
    songFuse.value = new Fuse(songs, {
      keys: [
        { name: 'name', weight: 0.5 },
        { name: 'artists', weight: 0.3 },
        { name: 'album', weight: 0.2 },
      ],
      includeScore:       true,
      threshold:          0.2,
      minMatchCharLength: 2,
    })

    albumFuse.value = new Fuse(albums, {
      keys: [
        { name: 'name', weight: 0.6 },
        { name: 'artist', weight: 0.4 },
      ],
      includeScore:       true,
      threshold:          0.2,
      minMatchCharLength: 2,
    })

    artistFuse.value = new Fuse(artists, {
      keys: [
        { name: 'name', weight: 1.0 },
      ],
      includeScore:       true,
      threshold:          0.2,
      minMatchCharLength: 2,
    })
  }

  // Watch for data changes and reinitialize search
  watch([() => songs, () => albums, () => artists], () => {
    initializeSearch()
  }, { immediate: true })

  // Computed search results
  const filteredSongs = computed(() => {
    if (!searchQuery || searchQuery.length < 2 || !songFuse.value) {
      return songs
    }
    return songFuse.value.search(searchQuery).map(result => result.item)
  })

  const filteredAlbums = computed(() => {
    if (!searchQuery || searchQuery.length < 2 || !albumFuse.value) {
      return albums
    }
    return albumFuse.value.search(searchQuery).map(result => result.item)
  })

  const filteredArtists = computed(() => {
    if (!searchQuery || searchQuery.length < 2 || !artistFuse.value) {
      return artists
    }
    return artistFuse.value.search(searchQuery).map(result => result.item)
  })

  // Search all types and return combined results
  const searchAll = computed(() => {
    if (!searchQuery || searchQuery.length < 2) {
      return {
        songs:   songs.slice(0, 5),
        albums:  albums.slice(0, 5),
        artists: artists.slice(0, 5),
      }
    }

    return {
      songs:   filteredSongs.value.slice(0, 5),
      albums:  filteredAlbums.value.slice(0, 5),
      artists: filteredArtists.value.slice(0, 5),
    }
  })

  // Check if search is active
  const isSearching = computed(() => searchQuery && searchQuery.length >= 2)

  return {
    // Computed results
    filteredSongs,
    filteredAlbums,
    filteredArtists,
    searchAll,
    isSearching,

    // Actions
    initializeSearch,
  }
}
