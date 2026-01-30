import { computed, ComputedRef, ref, shallowRef } from 'vue'

import type { Album, NameIdPair, Song } from '../lib/api/types'

import { useDebouncedComputed } from './useDebouncedComputed'
import { useSongInteractions } from './useSongInteractions'
import { logger } from '../lib/logger'
import { sortSongsByTrackOrder } from '../lib/transforms'
import { useAuthStore, useHomeStore } from '../stores'

export interface HomePageComposableReturn {
  featuredAlbum:            ComputedRef<Album | null>
  featuredAlbumArtistPairs: ComputedRef<NameIdPair[]>
  featuredAlbums:           ComputedRef<Album[]>
  hasMoreData:              ComputedRef<{
    featuredAlbums: boolean
    randomAlbums:   boolean
    recentlyAdded:  boolean
    recentlyPlayed: boolean
  }>
  isLoading:         ComputedRef<boolean>
  loadingStage:      ComputedRef<'extended' | 'full' | 'initial'>
  loadMoreData:      () => Promise<void>
  mostPlayed:        ComputedRef<Song[]>
  nextFeaturedAlbum: () => void
  playAlbumSongs:    (album: Album) => void
  playFeaturedAlbum: () => void
  playInstantMix:    (song: Song) => void
  playSongs:         (songs: Song[], startWith?: Song) => void
  prevFeaturedAlbum: () => void
  randomAlbums:      ComputedRef<Album[]>
  recentlyAdded:     ComputedRef<Album[]>
  recentlyPlayed:    ComputedRef<Song[]>
  serverUrl:         ComputedRef<string>
  token:             ComputedRef<string>
}

export const useHomePage = (emit: {
  (e: 'play-songs', songs: Song[]): void
  (e: 'select-album', album: Album): void
}): HomePageComposableReturn => {
  const authStore = useAuthStore()
  const homeStore = useHomeStore()

  const credentials = computed(() => ({
    serverUrl: authStore.serverUrl,
    token:     authStore.token,
    userId:    authStore.userId,
    username:  authStore.username,
  }))

  const serverUrl = computed(() => credentials.value.serverUrl)
  const token = computed(() => credentials.value.token)

  const { playInstantMix } = useSongInteractions(credentials)

  const isLoading = computed(() => homeStore.isLoading)
  const loadingStage = computed(() => homeStore.loadingStage)
  const hasMoreData = computed(() => homeStore.hasMoreData)
  const recentlyPlayed = computed(() => homeStore.recentlyPlayedSongs)
  const recentlyAdded = computed(() => homeStore.recentlyAddedAlbums)
  const randomAlbums = computed(() => homeStore.randomLibraryAlbums)
  const featuredAlbums = computed(() => homeStore.featuredLibraryAlbums)
  const currentFeaturedIndex = ref(0)

  // Debounced featured album to prevent excessive updates during rapid navigation
  const featuredAlbumDebounced = useDebouncedComputed(() =>
    featuredAlbums.value[currentFeaturedIndex.value] || null,
    // eslint-disable-next-line @stylistic/indent
    150, // 150ms delay for smooth transitions
  )

  // Ensure featuredAlbum matches the declared ComputedRef<Album | null> type:
  const featuredAlbum = computed<Album | null>(() => featuredAlbumDebounced.value)

  // Memoize artist pairs computation to avoid redundant processing
  const artistPairsCache = shallowRef<Map<string, NameIdPair[]>>(new Map())

  const featuredAlbumArtistPairs = computed<NameIdPair[]>(() => {
    const album = featuredAlbum.value
    if (!album) return []

    const cacheKey = album.id || album.name
    if (!cacheKey) return []

    // Check cache first
    if (artistPairsCache.value.has(cacheKey))
      return artistPairsCache.value.get(cacheKey)!

    const idToName = new Map<string, string>()
    const albumSongs = album.songs || []

    for (const song of albumSongs)
      if (song.albumArtists)
        for (const pair of song.albumArtists)
          if (pair.id && pair.name) idToName.set(pair.id, pair.name)

    // Fallbacks if albumArtists are not provided by backend
    if (idToName.size === 0) {
      const first = albumSongs[0]
      if (first?.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, idx) => {
          const name = first.artists![idx]
          if (id && name) idToName.set(id, name)
        })
      } else if (album.artist && album.artistId) {
        idToName.set(album.artistId, album.artist)
      }
    }

    const result = Array.from(idToName, ([id, name]) => ({ id, name }))

    // Cache the result with size limit
    const cache = artistPairsCache.value
    if (cache.size > 20) {
      const firstKey = cache.keys().next().value
      if (firstKey) cache.delete(firstKey)
    }
    cache.set(cacheKey, result)
    artistPairsCache.value = new Map(cache)

    return result
  })

  // Optimized most played computation with caching
  const mostPlayedCache = shallowRef<Song[]>([])

  const mostPlayed = computed(() => {
    const recentlyPlayedSongs = recentlyPlayed.value
    if (recentlyPlayedSongs.length === 0) return []

    // Simple cache invalidation - only recompute if recentlyPlayed changed
    if (mostPlayedCache.value.length > 0 &&
        recentlyPlayedSongs.length === mostPlayedCache.value.length) {
      return mostPlayedCache.value
    }

    const result = [...recentlyPlayedSongs]
      .sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
      .slice(0, 10)

    mostPlayedCache.value = result
    return result
  })

  const nextFeaturedAlbum = (): void => {
    if (featuredAlbums.value.length > 1)
      currentFeaturedIndex.value = (currentFeaturedIndex.value + 1) % featuredAlbums.value.length
  }

  const prevFeaturedAlbum = (): void => {
    if (featuredAlbums.value.length > 1)
      currentFeaturedIndex.value = currentFeaturedIndex.value === 0
        ? featuredAlbums.value.length - 1
        : currentFeaturedIndex.value - 1
  }

  const playSongs = (songs: Song[], startWith?: Song): void => {
    if (songs.length === 0) {
      logger.warn('No songs to play')
      return
    }

    const invalidSongs = songs.filter(song => !song || !song.id)

    if (invalidSongs.length > 0)
      logger.error('Found songs with invalid IDs:', invalidSongs)

    if (startWith) {
      const startIndex = songs.findIndex(song => song.id === startWith.id)
      if (startIndex === -1) {
        emit('play-songs', songs)
        return
      }
      const reorderedSongs = [...songs.slice(startIndex), ...songs.slice(0, startIndex)]
      emit('play-songs', reorderedSongs)
    } else {
      emit('play-songs', songs)
    }
  }

  const playFeaturedAlbum = (): void => {
    if (!featuredAlbum.value) {
      logger.warn('No featured album available')
      return
    }

    const albumSongs = featuredAlbum.value.songs || []
    if (albumSongs.length > 0)
      emit('play-songs', sortSongsByTrackOrder(albumSongs))
    else
      logger.warn('No songs found for featured album')
  }

  const playAlbumSongs = (album: Album): void => {
    if (album.songs && album.songs.length > 0)
      emit('play-songs', sortSongsByTrackOrder(album.songs))
    else
      logger.warn('No songs found for album', album.name)
  }

  return {
    featuredAlbum,
    featuredAlbumArtistPairs,
    featuredAlbums,
    hasMoreData,
    isLoading,
    loadingStage,
    loadMoreData: homeStore.loadMoreData,
    mostPlayed,
    nextFeaturedAlbum,
    playAlbumSongs,
    playFeaturedAlbum,
    playInstantMix,
    playSongs,
    prevFeaturedAlbum,
    randomAlbums,
    recentlyAdded,
    recentlyPlayed,
    serverUrl,
    token,
  }
}
