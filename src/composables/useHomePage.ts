import { computed, ComputedRef, ref } from 'vue'
import { useRouter } from 'vue-router'

import type { Album, NameIdPair, Song } from '@/bindings'

import { useSongInteractions } from '@/composables/useSongInteractions'
import { logger } from '@/lib/logger'
import { sortSongsByTrackOrder } from '@/lib/transforms'
import { useAuthStore, useHomeStore } from '@/stores'

export interface HomePageComposableReturn {
  featuredAlbum:            ComputedRef<Album | null>
  featuredAlbumArtistPairs: ComputedRef<NameIdPair[]>
  featuredAlbums:           ComputedRef<Album[]>
  isLoading:                ComputedRef<boolean>
  mostPlayed:               ComputedRef<Song[]>
  nextFeaturedAlbum:        () => void
  playAlbumSongs:           (album: Album) => void
  playFeaturedAlbum:        () => void
  playInstantMix:           (song: Song) => void
  playSongs:                (songs: Song[], startWith?: Song) => void
  prevFeaturedAlbum:        () => void
  randomAlbums:             ComputedRef<Album[]>
  recentlyAdded:            ComputedRef<Album[]>
  recentlyPlayed:           ComputedRef<Song[]>
  serverUrl:                ComputedRef<string>
  token:                    ComputedRef<string>
}

export const useHomePage = (emit: {
  (e: 'play-songs', songs: Song[]): void
  (e: 'select-album', album: Album): void
}): HomePageComposableReturn => {
  const router = useRouter()
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
  const recentlyPlayed = computed(() => homeStore.recentlyPlayedSongs)
  const recentlyAdded = computed(() => homeStore.recentlyAddedAlbums)
  const randomAlbums = computed(() => homeStore.randomLibraryAlbums)
  const featuredAlbums = computed(() => homeStore.featuredLibraryAlbums)
  const currentFeaturedIndex = ref(0)

  const featuredAlbum = computed(() =>
    featuredAlbums.value[currentFeaturedIndex.value] || null,
  )

  const featuredAlbumArtistPairs = computed<NameIdPair[]>(() => {
    const album = featuredAlbum.value
    if (!album) return []

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

    return Array.from(idToName, ([id, name]) => ({ id, name }))
  })

  const mostPlayed = computed(() =>
    recentlyPlayed.value.length > 0
      ? [...recentlyPlayed.value]
        .sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
        .slice(0, 10)
      : [],
  )

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
    if (albumSongs.length > 0) {
      emit('play-songs', sortSongsByTrackOrder(albumSongs))
      if (featuredAlbum.value.id) {
        router.push(`/albums/${featuredAlbum.value.id}`)
      }
    } else {
      logger.warn('No songs found for featured album')
    }
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
    isLoading,
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
