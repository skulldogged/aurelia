/**
 * Pure Data Transformation Functions
 *
 * These functions provide pure, composable transformations for common data operations
 * used throughout the application.
 */

import type { Album, Artist, Song } from '@/bindings'

/**
 * Song transformations
 */
export const getSongDuration = (song: Song): number =>
  Number(song.duration) || 0

export const formatSongDuration = (song: Song): string => {
  const seconds = getSongDuration(song)
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

export const getSongArtists = (song: Song): string[] =>
  song.artists || []

export const getSongGenres = (song: Song): string[] =>
  song.genres || []

export const isSongFavorite = (song: Song): boolean =>
  song.isFavorite || false

export const hasSongLyrics = (song: Song): boolean =>
  Boolean(song.lyrics)

/**
 * Album transformations
 */
export const getAlbumArtist = (album: Album): string =>
  album.artist || 'Unknown Artist'

export const getAlbumTrackCount = (album: Album): number =>
  Number(album.songCount) || 0

/**
 * Artist transformations
 */
export const getArtistName = (artist: Artist): string =>
  artist.name || 'Unknown Artist'

/**
 * Collection filters and sorters
 */
export const filterSongsByArtist = (artistId: string, songs: Song[]): Song[] =>
  songs.filter(song => song.artistIds?.includes(artistId) || false)

export const filterSongsByAlbum = (albumId: string, songs: Song[]): Song[] =>
  songs.filter(song => song.albumId === albumId)

export const filterSongsByGenre = (genreName: string, songs: Song[]): Song[] =>
  songs.filter(song => song.genres?.includes(genreName) || false)

export const filterFavoriteSongs = (songs: Song[]): Song[] =>
  songs.filter(isSongFavorite)

export const filterSongsWithLyrics = (songs: Song[]): Song[] =>
  songs.filter(hasSongLyrics)

export const sortSongsByName = (songs: Song[]): Song[] =>
  [...songs].sort((a, b) => (a.name || '').localeCompare(b.name || ''))

export const sortSongsByArtist = (songs: Song[]): Song[] =>
  [...songs].sort((a, b) => {
    const artistA = getSongArtists(a)[0] || ''
    const artistB = getSongArtists(b)[0] || ''
    return artistA.localeCompare(artistB)
  })

export const sortSongsByDuration = (songs: Song[]): Song[] =>
  [...songs].sort((a, b) => getSongDuration(a) - getSongDuration(b))

export const sortAlbumsByName = (albums: Album[]): Album[] =>
  [...albums].sort((a, b) => (a.name || '').localeCompare(b.name || ''))

export const sortAlbumsByArtist = (albums: Album[]): Album[] =>
  [...albums].sort((a, b) => getAlbumArtist(a).localeCompare(getAlbumArtist(b)))

export const sortArtistsByName = (artists: Artist[]): Artist[] =>
  [...artists].sort((a, b) => getArtistName(a).localeCompare(getArtistName(b)))

/**
 * Search and matching functions
 */
export const songMatchesQuery = (query: string, song: Song): boolean => {
  const lowerQuery = query.toLowerCase()
  return (
    song.name?.toLowerCase().includes(lowerQuery) ||
    getSongArtists(song).some(artist => artist.toLowerCase().includes(lowerQuery)) ||
    song.album?.toLowerCase().includes(lowerQuery) ||
    getSongGenres(song).some(genre => genre.toLowerCase().includes(lowerQuery))
  )
}

export const albumMatchesQuery = (query: string, album: Album): boolean => {
  const lowerQuery = query.toLowerCase()
  return (
    album.name?.toLowerCase().includes(lowerQuery) ||
    getAlbumArtist(album).toLowerCase().includes(lowerQuery)
  )
}

export const artistMatchesQuery = (query: string, artist: Artist): boolean => {
  const lowerQuery = query.toLowerCase()
  return getArtistName(artist).toLowerCase().includes(lowerQuery)
}

/**
 * Aggregation functions
 */
export const getTotalDuration = (songs: Song[]): number =>
  songs.reduce((total, song) => total + getSongDuration(song), 0)

export const getGenreCounts = (songs: Song[]): Record<string, number> =>
  songs.reduce((counts, song) => {
    getSongGenres(song).forEach(genre => {
      counts[genre] = (counts[genre] || 0) + 1
    })
    return counts
  }, {} as Record<string, number>)

export const getArtistSongCounts = (songs: Song[]): Record<string, number> =>
  songs.reduce((counts, song) => {
    getSongArtists(song).forEach(artist => {
      counts[artist] = (counts[artist] || 0) + 1
    })
    return counts
  }, {} as Record<string, number>)

/**
 * Data enrichment functions
 */
export const getFormattedSongList = (songs: Song[]): Array<Song & {
  artistNames:       string[]
  formattedDuration: string
  genreNames:        string[]
  isFavorite:        boolean
}> => sortSongsByName(songs).map(song => ({
  ...song,
  artistNames:       getSongArtists(song),
  formattedDuration: formatSongDuration(song),
  genreNames:        getSongGenres(song),
  isFavorite:        isSongFavorite(song),
}))

export const getAlbumSummary = (album: Album): Album & {
  artist:     string
  trackCount: number
} => ({
  ...album,
  artist:     getAlbumArtist(album),
  trackCount: getAlbumTrackCount(album),
})

export const getArtistSummary = (artist: Artist): Artist & {
  displayName: string
} => ({
  ...artist,
  displayName: getArtistName(artist),
})