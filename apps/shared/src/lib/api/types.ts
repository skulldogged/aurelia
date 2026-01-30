// Aurelia Shared API Types
// These types match the aurelia-core Rust models exactly

// Result type for API operations
export type Result<T, E = string> =
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E }

// Auth
export interface LoginResponse {
  token: string
  userId: string
}

export interface Credentials {
  serverUrl: string
  username: string
  token: string
  userId: string
}

export type AuthStatus = 'error' | 'loggedIn' | 'loggedOut' | 'pending'

export interface AuthError {
  code?: string
  isRetryable?: boolean
  message: string
  type: 'auth' | 'config' | 'network' | 'unknown'
}

// User data for items (favorites, play count, etc.)
export interface UserData {
  playbackPositionTicks: number
  playCount: number
  isFavorite: boolean
  played: boolean
  lastPlayedDate?: string
}

// Name-ID pair used for artists
export interface NameIdPair {
  name: string
  id: string
}

// Library - Song
export interface Song {
  id: string
  name: string
  itemType: string
  album?: string
  albumId?: string
  artists?: string[]
  artistIds?: string[]
  path?: string
  duration?: number
  albumArtUrl?: string
  year?: number
  playCount?: number
  isFavorite?: boolean
  discNumber?: number
  trackNumber?: number
  container?: string
  bitRate?: number
  sampleRate?: number
  codec?: string
  genres?: string[]
  premiereDate?: string
  datePlayed?: string
  dateCreated?: string
  dateModified?: string
  albumArtists?: NameIdPair[]
  lyrics?: string
  imageTags?: Record<string, string>
}

// Library - Artist
export interface Artist {
  name: string
  id: string
  imageTags?: Record<string, string>
  imageUrl?: string
  overview?: string
  providerIds?: Record<string, string>
  communityRating?: number
  songCount?: number
  dateModified?: string
  songs?: Song[]
}

// Library - Album
export interface Album {
  id?: string
  name: string
  artist: string
  artistId?: string
  albumArtUrl?: string
  songCount: number
  songs?: Song[]
  imageTags?: Record<string, string>
  providerIds?: Record<string, string>
  dateCreated?: string
  dateModified?: string
}

export interface LibraryData {
  albums: Album[]
  artists: Artist[]
  songs: Song[]
}

// Sync
export interface SyncStateInfo {
  lastSyncTime: string
  lastFullSyncTime?: string
  lastSyncVersion?: string
  songCount: number
  artistCount: number
  albumCount: number
}

export interface SyncProgress {
  stage: string
  current: number
  total: number
  isComplete: boolean
}

// Playlists
export interface Playlist {
  name: string
  serverId: string
  id: string
  canDelete?: boolean
  sortName?: string
  isFolder: boolean
  itemType: string
  userData?: UserData
  runTimeTicks?: number
  childCount?: number
  imageTags?: Record<string, string>
  backdropImageTags?: string[]
  imageBlurHashes?: Record<string, Record<string, string>>
  locationType: string
  mediaType?: string
  dateCreated?: string
  dateLastSaved?: string
  isFavorite?: boolean
  description?: string
  songs?: Song[]
}

export interface PlaylistCreateData {
  name: string
  ids?: string[]
  userId: string
  isPublic?: boolean
}

export interface PlaylistUpdateData {
  name?: string
  ids?: string[]
  userId?: string
  isPublic?: boolean
  songs?: Song[]
  isFavorite?: boolean
}

export interface PlaylistItem {
  id: string
  name: string
  itemType: string
}

// Home View
export interface HomeViewData {
  recentlyPlayed: Song[]
  recentlyAdded: Song[]
  randomAlbums: Album[]
  featuredAlbums: Album[]
}

// Audio
export interface AudioStreamParams {
  serverUrl: string
  token: string
  itemId: string
  container?: string
}

// Playback Progress (WebSocket)
export interface PlaybackProgress {
  itemId: string
  position: number
  duration: number
  isPlaying: boolean
}

// Third-party Integrations
export interface LastFmCredentials {
  apiKey: string
  apiSecret: string
  sessionKey: string
  username: string
}

export interface ListenBrainzCredentials {
  userToken: string
  username?: string
}

export interface ListenBrainzListen {
  artist: string
  track: string
  album?: string
}

// Image
export interface ImageParams {
  itemId: string
  imageType: string
  serverUrl: string
  token: string
  width?: number
  quality?: number
}

// EQ
export interface EQBand {
  band: number
  gain: number
}

export type EQPreset = 'bass' | 'classical' | 'custom' | 'flat' | 'pop' | 'rock' | 'vocal'

// Share URLs
export type ShareUrlType = 'jellyfin' | 'lastfm' | 'listenbrainz' | 'spotify'

// Now Playing payload for system integration
export interface NowPlayingPayload {
  title: string
  artist: string | null
  album: string | null
  durationSecs: number | null
  coverUrl: string | null
}

// Discord Rich Presence
export interface RpcButton {
  label: string
  url: string
}

export interface RpcActivity {
  buttons: RpcButton[] | null
  details: string | null
  end_timestamp: number | null
  large_image: string | null
  large_text: string | null
  small_image: string | null
  small_text: string | null
  start_timestamp: number | null
  state: string | null
}

// Main API Client Interface
export interface ApiClient {
  // Auth
  getSavedCredentials(): Promise<Result<Credentials | null>>
  authenticate(serverUrl: string, username: string, password: string): Promise<Result<Credentials>>
  logout(): Promise<Result<void>>
  saveCredentials(serverUrl: string, username: string, token: string, userId: string): Promise<Result<void>>

  // Library
  getLibrary(): Promise<Result<LibraryData>>
  syncLibrary(): Promise<Result<void>>
  getSyncState(): Promise<Result<SyncStateInfo>>
  getSong(songId: string): Promise<Result<Song>>
  getArtist(artistId: string): Promise<Result<Artist>>

  // Playlists
  getPlaylists(): Promise<Result<Playlist[]>>
  getPlaylistItems(playlistId: string): Promise<Result<Song[]>>
  createPlaylist(data: PlaylistCreateData): Promise<Result<Playlist>>
  updatePlaylist(playlistId: string, updates: PlaylistUpdateData): Promise<Result<Playlist>>
  deletePlaylist(playlistId: string): Promise<Result<void>>
  addPlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<void>>
  removePlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<void>>

  // Home
  getHomeViewData(): Promise<Result<HomeViewData>>
  getRecentlyPlayed(serverUrl: string, token: string, userId: string): Promise<Result<Song[]>>

  // Audio
  getAudioStreamUrl(params: AudioStreamParams): Promise<Result<string>>
  getSavedVolume(): Promise<Result<number | null>>
  saveVolume(volume: number): Promise<Result<void>>

  // Images
  getImage(params: ImageParams): Promise<Result<string | null>>
  clearCache(serverUrl: string, token: string): Promise<Result<void>>
  clearImageFromCache(itemId: string, imageType: string): Promise<Result<void>>
  getImageCacheStats(): Promise<Result<string>>

  // Lyrics
  getLyrics(id: string, artist: string, title: string, path?: string): Promise<Result<string>>

  // Favorites
  toggleFavoriteStatus(itemId: string, isFavorite: boolean): Promise<Result<boolean>>

  // Instant Mix
  getInstantMix(itemId: string): Promise<Result<Song[]>>

  // Related Artists
  getRelatedArtists(artistId: string): Promise<Result<Artist[]>>

  // Share URLs
  getSongShareUrls(songId: string): Promise<Result<Partial<Record<ShareUrlType, string>>>>

  // Last.fm
  lastfmSetCredentials(credentials: LastFmCredentials): Promise<Result<void>>
  lastfmClearCredentials(): Promise<Result<void>>
  lastfmIsAuthenticated(): Promise<Result<boolean>>
  lastfmAuthenticate(): Promise<Result<LastFmCredentials>>
  lastfmScrobble(artist: string, track: string, album?: string, timestamp?: number): Promise<Result<void>>
  lastfmUpdateNowPlaying(artist: string, track: string, album?: string): Promise<Result<void>>

  // ListenBrainz
  listenbrainzSetCredentials(credentials: ListenBrainzCredentials): Promise<Result<void>>
  listenbrainzClearCredentials(): Promise<Result<void>>
  listenbrainzIsAuthenticated(): Promise<Result<boolean>>
  listenbrainzValidateToken(userToken: string): Promise<Result<ListenBrainzCredentials>>
  listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number): Promise<Result<void>>
  listenbrainzPlayingNow(artist: string, track: string, album?: string): Promise<Result<void>>

  // Audio - Rust player (desktop only)
  audioInit?(): Promise<Result<void>>
  audioPlay?(streamUrl: string): Promise<Result<void>>
  audioPause?(): Promise<Result<void>>
  audioResume?(): Promise<Result<void>>
  audioStop?(): Promise<Result<void>>
  audioGetPosition?(): Promise<Result<number>>
  audioSeek?(position: number): Promise<Result<void>>
  audioGetVolume?(): Promise<Result<number>>
  audioSetVolume?(volume: number): Promise<Result<void>>
  audioSetEqBand?(band: number, gain: number): Promise<Result<void>>
  audioGetEqBand?(band: number): Promise<Result<number>>
  audioGetAllEqBands?(): Promise<Result<number[]>>
  audioSetEqEnabled?(enabled: boolean): Promise<Result<void>>
  audioIsEqEnabled?(): Promise<Result<boolean>>
  audioSetEqPreset?(preset: EQPreset): Promise<Result<void>>
  audioGetEqPreset?(): Promise<Result<EQPreset>>
  audioIsAnalyzerEnabled?(): Promise<Result<boolean>>
  audioSetAnalyzerEnabled?(enabled: boolean): Promise<Result<void>>
  audioPrepareNext?(streamUrl: string): Promise<Result<void>>
  audioAdvanceGapless?(): Promise<Result<void>>

  // Session/Playback reporting
  registerClientCapabilities?(serverUrl: string, token: string, deviceId: string): Promise<Result<void>>
  reportPlaybackStart?(itemId: string, position?: number): Promise<Result<void>>
  reportPlaybackProgress?(itemId: string, position: number, isPaused: boolean): Promise<Result<void>>
  reportPlaybackStop?(itemId: string, position: number): Promise<Result<void>>
  markItemPlayed?(itemId: string): Promise<Result<void>>

  // System Tray / Window Management (Desktop only)
  showMainWindow?(): Promise<Result<void>>
  hideMainWindow?(): Promise<Result<void>>
  quitApplication?(): Promise<Result<void>>
  setMinimizeToTray?(minimizeToTray: boolean): Promise<Result<void>>
  setCloseToTray?(closeToTray: boolean): Promise<Result<void>>

  // Discord Rich Presence (Desktop only)
  discordRpcIsRunning?(): Promise<Result<boolean>>
  discordRpcStart?(appId: string): Promise<Result<void>>
  discordRpcStop?(): Promise<Result<void>>
  discordRpcSetActivity?(activity: RpcActivity): Promise<Result<void>>
  discordRpcClearActivity?(): Promise<Result<void>>

  // Media Controls (Desktop only - system media integration)
  mediaClearNowPlaying?(): Promise<Result<void>>
  mediaSetPlaybackStatus?(isPlaying: boolean, positionSecs: number | null): Promise<Result<void>>
  mediaUpdateNowPlaying?(payload: NowPlayingPayload): Promise<Result<void>>
  mediaSetButtonEnabled?(button: string, enabled: boolean): Promise<Result<void>>
}

// WebSocket Events Interface
export interface WebSocketClient {
  onSyncState(callback: (state: SyncStateInfo) => void): () => void
  onPlaybackProgress(callback: (progress: PlaybackProgress) => void): () => void
  onLibraryUpdate(callback: () => void): () => void
  connect(): Promise<void>
  disconnect(): void
}

// Platform type for conditional behavior
export type Platform = 'desktop' | 'web'

// Platform detection
export const getPlatform = (): Platform => {
  // Check if running in Tauri
  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    return 'desktop'
  }
  return 'web'
}
