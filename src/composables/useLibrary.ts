import { ref, computed, readonly } from 'vue'
import type { Song, Album, Artist, Credentials } from '@/bindings'
import { commands } from '@/bindings'

export const useLibrary = () => {
  const allSongs = ref<Song[]>([])
  const allArtists = ref<Artist[]>([])
  const allArtistsWithSongs = ref<Artist[]>([])
  const albumArtistsWithSongs = ref<Artist[]>([])
  const libraryLoading = ref(false)
  const libraryError = ref<string | null>(null)

  // Optimized computed albums from songs
  const allAlbums = computed(() => {
    // Use a Map for O(1) lookups and efficient updates
    const albumsMap = new Map<string, Album>()

    // Process songs in batches for better performance with large libraries
    const songs = allSongs.value
    const length = songs.length

    for (let i = 0; i < length; i++) {
      const song = songs[i]

      if (song.album && song.albumId) {
        const albumId = song.albumId

        if (!albumsMap.has(albumId)) {
          albumsMap.set(albumId, {
            id:          albumId,
            name:        song.album,
            artist:      song.albumArtists?.[0]?.name || song.artists?.[0] || 'Unknown Artist',
            artistId:    song.albumArtists?.[0]?.id || song.artistIds?.[0] || null,
            albumArtUrl: song.albumArtUrl,
            songCount:   0,
            songs:       [],
          })
        }

        const album = albumsMap.get(albumId)!
        album.songs!.push(song)
        album.songCount = album.songs!.length
      }
    }

    return Array.from(albumsMap.values())
  })

  const loadLibrary = async (credentials: Credentials) => {
    libraryLoading.value = true
    libraryError.value = null

    try {
      const songsResult = await commands.getSongs(
        credentials.serverUrl,
        credentials.token,
        null, null, null, null,
      )

      if (songsResult.status === 'error') {
        throw new Error(`Failed to load songs: ${songsResult.error}`)
      }

      allSongs.value = songsResult.data

      const [artistsWithSongsResult, albumArtistsResult] = await Promise.all([
        commands.getArtists(credentials.serverUrl, credentials.token, true, false, null, null),
        commands.getArtists(credentials.serverUrl, credentials.token, true, true, null, null),
      ])

      if (artistsWithSongsResult.status === 'error') {
        throw new Error(`Failed to load artists: ${artistsWithSongsResult.error}`)
      }
      if (albumArtistsResult.status === 'error') {
        throw new Error(`Failed to load album artists: ${albumArtistsResult.error}`)
      }

      allArtistsWithSongs.value = artistsWithSongsResult.data
      albumArtistsWithSongs.value = albumArtistsResult.data

      libraryError.value = null
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load music library'
      libraryError.value = errorMessage
      throw new Error(errorMessage)
    } finally {
      libraryLoading.value = false
    }
  }

  const syncLibrary = async (credentials: Credentials) => {
    try {
      const syncResult = await commands.syncLibrary(credentials.serverUrl, credentials.token)
      if (syncResult.status === 'error')
        throw new Error(`Failed to sync library: ${syncResult.error}`)

      await loadLibrary(credentials)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to sync music library'
      libraryError.value = errorMessage
      throw new Error(errorMessage)
    }
  }

  const clearCache = async (credentials: Credentials) => {
    try {
      const clearResult = await commands.clearCache(credentials.serverUrl, credentials.token)
      if (clearResult.status === 'error')
        throw new Error(`Failed to clear cache: ${clearResult.error}`)

      await loadLibrary(credentials)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to clear music cache'
      libraryError.value = errorMessage
      throw new Error(errorMessage)
    }
  }

  return {
    allSongs,
    allArtists,
    allArtistsWithSongs,
    albumArtistsWithSongs,
    allAlbums,
    libraryLoading: readonly(libraryLoading),
    libraryError:   readonly(libraryError),

    loadLibrary,
    syncLibrary,
    clearCache,
  }
}
