// Aurelia Shared API Types
// These types match the aurelia-core Rust models exactly

// Library - Album
export interface Album {
  albumArtUrl?:  string
  artist:        string
  artistId?:     string
  dateCreated?:  string
  dateModified?: string
  id?:           string
  imageTags?:    Record<string, string>
  name:          string
  providerIds?:  Record<string, string>
  songCount:     number
  songs?:        Song[]
}

// Main API Client Interface
export interface ApiClient {
  addPlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<void>>
  audioAdvanceGapless?(): Promise<Result<void>>
  audioGetAllEqBands?(): Promise<Result<number[]>>
  audioGetEqBand?(band: number): Promise<Result<number>>

  audioGetEqPreset?(): Promise<Result<EQPreset>>
  audioGetPosition?(): Promise<Result<number>>
  audioGetVolume?(): Promise<Result<number>>
  // Audio - Rust player (desktop only)
  audioInit?(): Promise<Result<void>>
  audioIsAnalyzerEnabled?(): Promise<Result<boolean>>

  audioIsEqEnabled?(): Promise<Result<boolean>>
  audioPause?(): Promise<Result<void>>
  audioPlay?(streamUrl: string): Promise<Result<void>>
  audioPrepareNext?(streamUrl: string): Promise<Result<void>>
  audioResume?(): Promise<Result<void>>
  audioSeek?(position: number): Promise<Result<void>>
  audioSetAnalyzerEnabled?(enabled: boolean): Promise<Result<void>>

  audioSetEqBand?(band: number, gain: number): Promise<Result<void>>
  audioSetEqEnabled?(enabled: boolean): Promise<Result<void>>

  audioSetEqPreset?(preset: EQPreset): Promise<Result<void>>
  audioSetVolume?(volume: number): Promise<Result<void>>
  audioStop?(): Promise<Result<void>>

  authenticate(serverUrl: string, username: string, password: string): Promise<Result<Credentials>>
  clearCache(serverUrl: string, token: string): Promise<Result<void>>
  clearImageFromCache(itemId: string, imageType: string): Promise<Result<void>>
  createPlaylist(data: PlaylistCreateData): Promise<Result<Playlist>>

  deletePlaylist(playlistId: string): Promise<Result<void>>

  discordRpcClearActivity?(): Promise<Result<void>>

  // Discord Rich Presence (Desktop only)
  discordRpcIsRunning?(): Promise<Result<boolean>>

  discordRpcSetActivity?(activity: RpcActivity): Promise<Result<void>>

  discordRpcStart?(appId: string): Promise<Result<void>>

  discordRpcStop?(): Promise<Result<void>>
  getArtist(artistId: string): Promise<Result<Artist>>
  // Audio
  getAudioStreamUrl(params: AudioStreamParams): Promise<Result<string>>
  // Home
  getHomeViewData(): Promise<Result<HomeViewData>>
  // Images
  getImage(params: ImageParams): Promise<Result<null | string>>
  getImageCacheStats(): Promise<Result<string>>

  // Instant Mix
  getInstantMix(itemId: string): Promise<Result<Song[]>>
  // Library
  getLibrary(): Promise<Result<LibraryData>>
  // Lyrics
  getLyrics(id: string, artist: string, title: string, path?: string): Promise<Result<string>>
  getPlaylistItems(playlistId: string): Promise<Result<Song[]>>
  // Playlists
  getPlaylists(): Promise<Result<Playlist[]>>
  getRecentlyPlayed(serverUrl: string, token: string, userId: string): Promise<Result<Song[]>>

  // Related Artists
  getRelatedArtists(artistId: string): Promise<Result<Artist[]>>
  // Auth
  getSavedCredentials(): Promise<Result<Credentials | null>>
  getSavedVolume(): Promise<Result<null | number>>
  getSong(songId: string): Promise<Result<Song>>
  // Share URLs
  getSongShareUrls(songId: string): Promise<Result<Partial<Record<ShareUrlType, string>>>>
  getSyncState(): Promise<Result<SyncStateInfo>>
  hideMainWindow?(): Promise<Result<void>>
  lastfmAuthenticate(): Promise<Result<LastFmCredentials>>
  lastfmClearCredentials(): Promise<Result<void>>
  lastfmIsAuthenticated(): Promise<Result<boolean>>
  lastfmScrobble(artist: string, track: string, album?: string, timestamp?: number): Promise<Result<void>>
  // Last.fm
  lastfmSetCredentials(credentials: LastFmCredentials): Promise<Result<void>>
  lastfmUpdateNowPlaying(artist: string, track: string, album?: string): Promise<Result<void>>
  listenbrainzClearCredentials(): Promise<Result<void>>
  listenbrainzIsAuthenticated(): Promise<Result<boolean>>
  listenbrainzPlayingNow(artist: string, track: string, album?: string): Promise<Result<void>>
  // ListenBrainz
  listenbrainzSetCredentials(credentials: ListenBrainzCredentials): Promise<Result<void>>
  listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number): Promise<Result<void>>
  listenbrainzValidateToken(userToken: string): Promise<Result<ListenBrainzCredentials>>
  logout(): Promise<Result<void>>

  markItemPlayed?(itemId: string): Promise<Result<void>>
  // Media Controls (Desktop only - system media integration)
  mediaClearNowPlaying?(): Promise<Result<void>>
  mediaSetButtonEnabled?(button: string, enabled: boolean): Promise<Result<void>>
  mediaSetPlaybackStatus?(isPlaying: boolean, positionSecs: null | number): Promise<Result<void>>
  mediaUpdateNowPlaying?(payload: NowPlayingPayload): Promise<Result<void>>

  quitApplication?(): Promise<Result<void>>
  // Session/Playback reporting
  registerClientCapabilities?(serverUrl: string, token: string, deviceId: string): Promise<Result<void>>
  removePlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<void>>
  reportPlaybackProgress?(itemId: string, position: number, isPaused: boolean): Promise<Result<void>>
  reportPlaybackStart?(itemId: string, position?: number): Promise<Result<void>>

  reportPlaybackStop?(itemId: string, position: number): Promise<Result<void>>
  saveCredentials(serverUrl: string, username: string, token: string, userId: string): Promise<Result<void>>
  saveVolume(volume: number): Promise<Result<void>>
  setCloseToTray?(closeToTray: boolean): Promise<Result<void>>
  setMinimizeToTray?(minimizeToTray: boolean): Promise<Result<void>>

  // System Tray / Window Management (Desktop only)
  showMainWindow?(): Promise<Result<void>>
  syncLibrary(): Promise<Result<void>>
  // Favorites
  toggleFavoriteStatus(itemId: string, isFavorite: boolean): Promise<Result<boolean>>
  updatePlaylist(playlistId: string, updates: PlaylistUpdateData): Promise<Result<Playlist>>
}

// Library - Artist
export interface Artist {
  communityRating?: number
  dateModified?:    string
  id:               string
  imageTags?:       Record<string, string>
  imageUrl?:        string
  name:             string
  overview?:        string
  providerIds?:     Record<string, string>
  songCount?:       number
  songs?:           Song[]
}

// Audio
export interface AudioStreamParams {
  container?: string
  itemId:     string
  serverUrl:  string
  token:      string
}

export interface AuthError {
  code?:        string
  isRetryable?: boolean
  message:      string
  type:         'auth' | 'config' | 'network' | 'unknown'
}

export type AuthStatus = 'error' | 'loggedIn' | 'loggedOut' | 'pending'

export interface Credentials {
  serverUrl: string
  token:     string
  userId:    string
  username:  string
}

// EQ
export interface EQBand {
  band: number
  gain: number
}

export type EQPreset = 'bass' | 'classical' | 'custom' | 'flat' | 'pop' | 'rock' | 'vocal'

// Home View
export interface HomeViewData {
  featuredAlbums: Album[]
  randomAlbums:   Album[]
  recentlyAdded:  Song[]
  recentlyPlayed: Song[]
}

// Image
export interface ImageParams {
  imageType: string
  itemId:    string
  quality?:  number
  serverUrl: string
  token:     string
  width?:    number
}

// Third-party Integrations
export interface LastFmCredentials {
  apiKey:     string
  apiSecret:  string
  sessionKey: string
  username:   string
}

export interface LibraryData {
  albums:  Album[]
  artists: Artist[]
  songs:   Song[]
}

export interface ListenBrainzCredentials {
  username?: string
  userToken: string
}

export interface ListenBrainzListen {
  album?: string
  artist: string
  track:  string
}

// Auth
export interface LoginResponse {
  token:  string
  userId: string
}

// Name-ID pair used for artists
export interface NameIdPair {
  id:   string
  name: string
}

// Now Playing payload for system integration
export interface NowPlayingPayload {
  album:        null | string
  artist:       null | string
  coverUrl:     null | string
  durationSecs: null | number
  title:        string
}

// Platform type for conditional behavior
export type Platform = 'desktop' | 'web'

// Playback Progress (WebSocket)
export interface PlaybackProgress {
  duration:  number
  isPlaying: boolean
  itemId:    string
  position:  number
}

// Playlists
export interface Playlist {
  backdropImageTags?: string[]
  canDelete?:         boolean
  childCount?:        number
  dateCreated?:       string
  dateLastSaved?:     string
  description?:       string
  id:                 string
  imageBlurHashes?:   Record<string, Record<string, string>>
  imageTags?:         Record<string, string>
  isFavorite?:        boolean
  isFolder:           boolean
  itemType:           string
  locationType:       string
  mediaType?:         string
  name:               string
  runTimeTicks?:      number
  serverId:           string
  songs?:             Song[]
  sortName?:          string
  userData?:          UserData
}

export interface PlaylistCreateData {
  ids?:      string[]
  isPublic?: boolean
  name:      string
  userId:    string
}

export interface PlaylistItem {
  id:       string
  itemType: string
  name:     string
}

export interface PlaylistUpdateData {
  ids?:        string[]
  isFavorite?: boolean
  isPublic?:   boolean
  name?:       string
  songs?:      Song[]
  userId?:     string
}

// Result type for API operations
export type Result<T, E = string> =
  | { data: T; status: 'ok'; }
  | { error: E; status: 'error'; }

export interface RpcActivity {
  buttons:         null | RpcButton[]
  details:         null | string
  end_timestamp:   null | number
  large_image:     null | string
  large_text:      null | string
  small_image:     null | string
  small_text:      null | string
  start_timestamp: null | number
  state:           null | string
}

// Discord Rich Presence
export interface RpcButton {
  label: string
  url:   string
}

// Share URLs
export type ShareUrlType = 'jellyfin' | 'lastfm' | 'listenbrainz' | 'spotify'

// Library - Song
export interface Song {
  album?:        string
  albumArtists?: NameIdPair[]
  albumArtUrl?:  string
  albumId?:      string
  artistIds?:    string[]
  artists?:      string[]
  bitRate?:      number
  codec?:        string
  container?:    string
  dateCreated?:  string
  dateModified?: string
  datePlayed?:   string
  discNumber?:   number
  duration?:     number
  genres?:       string[]
  id:            string
  imageTags?:    Record<string, string>
  isFavorite?:   boolean
  itemType:      string
  lyrics?:       string
  name:          string
  path?:         string
  playCount?:    number
  premiereDate?: string
  sampleRate?:   number
  trackNumber?:  number
  year?:         number
}

export interface SyncProgress {
  current:    number
  isComplete: boolean
  stage:      string
  total:      number
}

// Sync
export interface SyncStateInfo {
  albumCount:        number
  artistCount:       number
  lastFullSyncTime?: string
  lastSyncTime:      string
  lastSyncVersion?:  string
  songCount:         number
}

// User data for items (favorites, play count, etc.)
export interface UserData {
  isFavorite:            boolean
  lastPlayedDate?:       string
  playbackPositionTicks: number
  playCount:             number
  played:                boolean
}

// WebSocket Events Interface
export interface WebSocketClient {
  connect(): Promise<void>
  disconnect(): void
  onLibraryUpdate(callback: () => void): () => void
  onPlaybackProgress(callback: (progress: PlaybackProgress) => void): () => void
  onSyncState(callback: (state: SyncStateInfo) => void): () => void
}

// Platform detection
export const getPlatform = (): Platform => {
  // Check if running in Tauri
  if (typeof window !== 'undefined' && (window as Window & { __TAURI__?: unknown }).__TAURI__) {
    return 'desktop'
  }
  return 'web'
}
