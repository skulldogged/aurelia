import { ref, computed, readonly } from 'vue'
import { commands } from '@/bindings'
import type { Song, Album, Artist, Credentials } from '@/bindings'

export interface DataState<T> {
  data:    T[]
  loading: boolean
  error:   string | null
}

export const useDataFetching = (credentials: Credentials | null) => {
  const songsState = ref<DataState<Song>>({
    data:    [],
    loading: false,
    error:   null,
  })

  const artistsState = ref<DataState<Artist>>({
    data:    [],
    loading: false,
    error:   null,
  })

  const albumsState = ref<DataState<Album>>({
    data:    [],
    loading: false,
    error:   null,
  })

  const songs = computed(() => songsState.value.data)
  const artists = computed(() => artistsState.value.data)
  const albums = computed(() => albumsState.value.data)
  const isLoading = computed(() =>
    songsState.value.loading || artistsState.value.loading || albumsState.value.loading,
  )
  const hasError = computed(() =>
    !!(songsState.value.error || artistsState.value.error || albumsState.value.error),
  )

  const fetchSongs = async (options?: {
    limit?:    number
    offset?:   number
    albumId?:  string
    artistId?: string
  }) => {
    if (!credentials) throw new Error('No credentials available')

    songsState.value.loading = true
    songsState.value.error = null

    try {
      const result = await commands.getSongs(
        credentials.serverUrl,
        credentials.token,
        options?.limit ?? null,
        options?.offset ?? null,
        options?.albumId ?? null,
        options?.artistId ?? null,
      )

      if (result.status === 'error') {
        throw new Error(`Failed to fetch songs: ${result.error}`)
      }

      songsState.value.data = result.data
    } catch (error) {
      songsState.value.error = error instanceof Error ? error.message : 'Failed to fetch songs'
      throw error
    } finally {
      songsState.value.loading = false
    }
  }

  const fetchArtists = async (options?: {
    includeSongs?:     boolean
    albumArtistsOnly?: boolean
    limit?:            number
    offset?:           number
  }) => {
    if (!credentials) throw new Error('No credentials available')

    artistsState.value.loading = true
    artistsState.value.error = null

    try {
      const result = await commands.getArtists(
        credentials.serverUrl,
        credentials.token,
        options?.includeSongs ?? false,
        options?.albumArtistsOnly ?? false,
        options?.limit ?? null,
        options?.offset ?? null,
      )

      if (result.status === 'error') {
        throw new Error(`Failed to fetch artists: ${result.error}`)
      }

      artistsState.value.data = result.data
    } catch (error) {
      artistsState.value.error = error instanceof Error ? error.message : 'Failed to fetch artists'
      throw error
    } finally {
      artistsState.value.loading = false
    }
  }

  const fetchAlbums = async (options?: {
    includeSongs?: boolean
    limit?:        number
    offset?:       number
  }) => {
    if (!credentials) throw new Error('No credentials available')

    albumsState.value.loading = true
    albumsState.value.error = null

    try {
      const result = await commands.getAlbums(
        credentials.serverUrl,
        credentials.token,
        options?.includeSongs ?? false,
        options?.limit ?? null,
        options?.offset ?? null,
      )

      if (result.status === 'error') {
        throw new Error(`Failed to fetch albums: ${result.error}`)
      }

      albumsState.value.data = result.data
    } catch (error) {
      albumsState.value.error = error instanceof Error ? error.message : 'Failed to fetch albums'
      throw error
    } finally {
      albumsState.value.loading = false
    }
  }

  return {
    songs,
    artists,
    albums,
    isLoading,
    hasError,
    songsState:   readonly(songsState),
    artistsState: readonly(artistsState),
    albumsState:  readonly(albumsState),

    fetchSongs,
    fetchArtists,
    fetchAlbums,
  }
}
