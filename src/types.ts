// Re-export generated types from tauri-specta bindings
export type {
  MusicItem,
  NameIdPair,
  ArtistInfo,
  AlbumWithSongs,
  ArtistWithSongs,
  Credentials,
  LoginResponse,
} from './bindings'

// Additional types that extend the generated ones
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
