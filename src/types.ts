export interface MusicItem {
  id: string
  name: string
  item_type: string
  album?: string
  artists?: string[]
  path?: string
  duration?: number
  albumArtUrl?: string
  year?: number
  playCount?: number
  isFavorite?: boolean
  artistArtUrl?: string
  trackNumber?: number
  genres?: string[]
  premiereDate?: string
  datePlayed?: string
}

export interface ArtistInfo {
  Name: string
  Id: string
  ImageTags?: { Primary?: string }
  imageUrl?: string
  overview?: string
  providerIds?: Record<string, string>
  communityRating?: number
}
