export interface MusicItem {
  id:            string
  name:          string
  item_type:     string
  album?:        string
  artists?:      string[]
  artistIds?:    string[]
  path?:         string
  duration?:     number
  albumArtUrl?:  string
  year?:         number
  playCount?:    number
  isFavorite?:   boolean
  artistArtUrl?: string
  trackNumber?:  number
  container?:    string
  bitRate?:      number
  sampleRate?:   number
  codec?:        string
  genres?:       string[]
  premiereDate?: string
  datePlayed?:   string
  albumArtists?: NameIdPair[]
}

export interface NameIdPair {
  Name: string
  Id:   string
}

export interface ArtistInfo {
  Name:             string
  Id:               string
  ImageTags?:       { Primary?: string }
  imageUrl?:        string
  overview?:        string
  providerIds?:     Record<string, string>
  communityRating?: number
}

export interface AlbumInfo {
  name:         string
  artist:       string
  artistId?:    string
  albumArtUrl?: string
  songCount:    number
}

export interface ArtistSummary {
  id:        string
  name:      string
  songCount: number
  imageUrl?: string
}

export interface AlbumWithSongs extends AlbumInfo {
  songs: MusicItem[]
}

export interface ArtistWithSongs {
  id:        string
  name:      string
  songCount: number
  imageUrl?: string
  songs:     MusicItem[]
}
