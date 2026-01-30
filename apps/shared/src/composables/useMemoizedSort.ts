import type { ComputedRef, Ref } from 'vue'

import { computed, shallowRef } from 'vue'

import type { Song } from '../lib/api/types'

interface SortOptions {
  songs:      Ref<Song[]>
  sortOption: Ref<string>
}

export const useMemoizedSort = ({ songs, sortOption }: SortOptions): {
  clearCache:  () => void
  sortedSongs: ComputedRef<Song[]>
} => {
  // Cache for sorted results
  const sortCache = shallowRef<Map<string, Song[]>>(new Map())

  // Create a unique cache key based on songs hash and sort option
  const getCacheKey = (songsList: Song[], option: string): string => {
    // Use length + first/last song IDs for quick hash
    const songIds = songsList.length > 0
      ? `${songsList.length}-${songsList[0].id}-${songsList[songsList.length - 1].id}`
      : 'empty'
    return `${option}-${songIds}`
  }

  // Memoized sorting function
  const sortSongs = (songsList: Song[], option: string): Song[] => {
    const cacheKey = getCacheKey(songsList, option)
    const cache = sortCache.value

    // Check cache first
    if (cache.has(cacheKey)) {
      return cache.get(cacheKey)!
    }

    // Perform sorting
    const sorted = [...songsList]
    switch (option) {
      case 'Album':
        sorted.sort((a, b) => (a.album || '').localeCompare(b.album || ''))
        break
      case 'Artist':
        sorted.sort((a, b) => (a.artists?.[0] || '').localeCompare(b.artists?.[0] || ''))
        break
      case 'Date Added':
        sorted.sort((a, b) => (b.dateCreated || '').localeCompare(a.dateCreated || ''))
        break
      case 'Play Count':
        sorted.sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
        break
      case 'Title':
        sorted.sort((a, b) => a.name.localeCompare(b.name))
        break
      default:
        sorted.sort((a, b) => a.name.localeCompare(b.name))
    }

    // Cache the result (limit cache size to prevent memory issues)
    if (cache.size > 50) {
      const firstKey = cache.keys().next().value
      if (firstKey)
        cache.delete(firstKey)
    }

    cache.set(cacheKey, sorted)

    return sorted
  }

  // Computed property that uses memoized sorting
  const sortedSongs = computed(() => sortSongs(songs.value, sortOption.value))

  // Function to clear cache when needed
  const clearCache = (): void => {
    sortCache.value.clear()
  }

  return {
    clearCache,
    sortedSongs,
  }
}