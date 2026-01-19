import type { ComputedRef, Ref } from 'vue'

import Fuse from 'fuse.js'
import { computed, ref, shallowRef, watch } from 'vue'

import type { Song } from '@/lib/api/bindings'

interface SearchOptions {
  debounceMs?: number
  searchQuery: Ref<string>
  songs:       Ref<Song[]>
}

const DEFAULT_FUSE_OPTIONS = {
  includeScore: true,
  keys:         [
    { name: 'name', weight: 0.5 },
    { name: 'artists', weight: 0.3 },
    { name: 'album', weight: 0.2 },
  ],
  minMatchCharLength: 2,
  threshold:          0.2,
}

export const useMemoizedSearch = ({ searchQuery, songs }: SearchOptions): {
  clearSearchIndex: () => void
  searchResults:    ComputedRef<Song[]>
} => {
  const songFuse = shallowRef<Fuse<Song>>()
  const lastSongsHash = ref<string>('')

  // Create a simple hash for the songs array to detect changes
  const getSongsHash = (songsList: Song[]): string => {
    if (songsList.length === 0) return 'empty'
    return `${songsList.length}-${songsList[0].id}-${songsList[songsList.length - 1]?.id}`
  }

  // Only rebuild Fuse index when songs actually change
  watch(songs, newSongs => {
    const currentHash = getSongsHash(newSongs)

    if (currentHash !== lastSongsHash.value && newSongs.length > 0) {
      songFuse.value = new Fuse(newSongs, DEFAULT_FUSE_OPTIONS)
      lastSongsHash.value = currentHash
    } else if (newSongs.length === 0) {
      songFuse.value = undefined
      lastSongsHash.value = 'empty'
    }
  }, { immediate: true })

  // Memoized search results
  const searchResults = computed(() => {
    const query = searchQuery.value

    // Return all songs if no query or too short
    if (!query || query.length < 2 || !songFuse.value) {
      return songs.value
    }

    try {
      return songFuse.value.search(query).map(result => result.item)
    } catch (error) {
      console.warn('Search error:', error)
      return songs.value
    }
  })

  // Function to manually clear the search index
  const clearSearchIndex = (): void => {
    songFuse.value = undefined
    lastSongsHash.value = ''
  }

  return {
    clearSearchIndex,
    searchResults,
  }
}