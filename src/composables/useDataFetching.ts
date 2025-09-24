import { computed, type ComputedRef, readonly, ref, type Ref } from 'vue'

import type { Album, Artist, Credentials, Song } from '@/bindings'

import { commands } from '@/bindings'
import { withState } from '@/lib/result'

export interface DataState<T> {
  data:    readonly T[]
  error:   null | string
  loading: boolean
}

const createDataFetchingState = (): {
  albumsState:  Ref<DataState<Album>>
  artistsState: Ref<DataState<Artist>>
  songsState:   Ref<DataState<Song>>
} => ({
  albumsState: ref<DataState<Album>>({
    data:    [],
    error:   null,
    loading: false,
  }),
  artistsState: ref<DataState<Artist>>({
    data:    [],
    error:   null,
    loading: false,
  }),
  songsState: ref<DataState<Song>>({
    data:    [],
    error:   null,
    loading: false,
  }),
})

const fetchSongs = async (
  credentials: Credentials | null,
  state: ReturnType<typeof createDataFetchingState>,
  options?: {
    albumId?:  string
    artistId?: string
    limit?:    number
    offset?:   number
  },
): Promise<void> => {
  if (!credentials) {
    state.songsState.value.error = 'No credentials available'
    return
  }

  await withState(state.songsState.value, () =>
    commands.getSongs(
      credentials.serverUrl,
      credentials.token,
      options?.limit ?? null,
      options?.offset ?? null,
      options?.albumId ?? null,
      options?.artistId ?? null,
    ),
  )
}

const fetchArtists = async (
  credentials: Credentials | null,
  state: ReturnType<typeof createDataFetchingState>,
  options?: {
    albumArtistsOnly?: boolean
    includeSongs?:     boolean
    limit?:            number
    offset?:           number
  },
): Promise<void> => {
  if (!credentials) {
    state.artistsState.value.error = 'No credentials available'
    return
  }

  await withState(state.artistsState.value, () =>
    commands.getArtists(
      credentials.serverUrl,
      credentials.token,
      options?.includeSongs ?? false,
      options?.albumArtistsOnly ?? false,
      options?.limit ?? null,
      options?.offset ?? null,
    ),
  )
}

const fetchAlbums = async (
  credentials: Credentials | null,
  state: ReturnType<typeof createDataFetchingState>,
  options?: {
    includeSongs?: boolean
    limit?:        number
    offset?:       number
  },
): Promise<void> => {
  if (!credentials) {
    state.albumsState.value.error = 'No credentials available'
    return
  }

  await withState(state.albumsState.value, () =>
    commands.getAlbums(
      credentials.serverUrl,
      credentials.token,
      options?.includeSongs ?? false,
      options?.limit ?? null,
      options?.offset ?? null,
    ),
  )
}

export interface DataFetching {
  albums:       ComputedRef<readonly Album[]>
  albumsState:  Readonly<Ref<DataState<Album>>>
  artists:      ComputedRef<readonly Artist[]>
  artistsState: Readonly<Ref<DataState<Artist>>>
  fetchAlbums: (
    options?: {
      includeSongs?: boolean
      limit?:        number
      offset?:       number
    }
  ) => Promise<void>
  fetchArtists: (
    options?: {
      albumArtistsOnly?: boolean
      includeSongs?:     boolean
      limit?:            number
      offset?:           number
    }
  ) => Promise<void>
  fetchSongs:   (
    options?: {
      albumId?:  string
      artistId?: string
      limit?:    number
      offset?:   number
    },
  ) => Promise<void>
  hasError:   ComputedRef<boolean>
  isLoading:  ComputedRef<boolean>
  songs:      ComputedRef<readonly Song[]>
  songsState: Readonly<Ref<DataState<Song>>>
}

export const useDataFetching = (credentials: Credentials | null): DataFetching => {
  const state = createDataFetchingState()

  const songs = computed(() => state.songsState.value.data)
  const artists = computed(() => state.artistsState.value.data)
  const albums = computed(() => state.albumsState.value.data)
  const isLoading = computed(() =>
    state.songsState.value.loading || state.artistsState.value.loading || state.albumsState.value.loading,
  )
  const hasError = computed(() =>
    !!(state.songsState.value.error || state.artistsState.value.error || state.albumsState.value.error),
  )

  return {
    albums,
    albumsState:  readonly(state.albumsState) as Readonly<Ref<DataState<Album>>>,
    artists,
    artistsState: readonly(state.artistsState) as Readonly<Ref<DataState<Artist>>>,
    fetchAlbums:  options => fetchAlbums(credentials, state, options),
    fetchArtists: options => fetchArtists(credentials, state, options),
    fetchSongs:   options => fetchSongs(credentials, state, options),
    hasError,

    isLoading,
    songs,
    songsState: readonly(state.songsState) as Readonly<Ref<DataState<Song>>>,
  }
}
