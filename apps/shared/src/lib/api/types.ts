// Auto-generated TypeScript types for Aurelia API
// Generated from Api trait - DO NOT EDIT MANUALLY

import type { Album, Artist, Credentials, HomeViewData, LastFmCredentials, LibraryData, ListenBrainzCredentials, ListenBrainzListen, NowPlayingPayload, Playlist, PlaylistCreateData, PlaylistUpdateData, RpcActivity, Song, SyncStateInfo } from '../../generated'
import type { Result } from './result'

export interface ApiClient {
  addPlaylistItems(playlistId: string, songIds: string[]): Promise<Result<any>>
  audioAdvanceGapless(): Promise<Result<any>>
  audioGetAllEqBands(): Promise<Result<any>>
  audioGetEqBand(band: number): Promise<Result<any>>
  audioGetPosition(): Promise<Result<any>>
  audioGetVolume(): Promise<Result<any>>
  audioInit(): Promise<Result<any>>
  audioIsAnalyzerEnabled(): Promise<Result<any>>
  audioIsEqEnabled(): Promise<Result<any>>
  audioIsFinished(): Promise<Result<any>>
  audioIsPlaying(): Promise<Result<any>>
  audioPause(): Promise<Result<any>>
  audioPlay(url: string, token: string): Promise<Result<any>>
  audioPrepareNext(url: string, token: string): Promise<Result<any>>
  audioReinit(): Promise<Result<any>>
  audioResetEq(): Promise<Result<any>>
  audioResume(): Promise<Result<any>>
  audioSeek(positionSecs: number): Promise<Result<any>>
  audioSetAnalyzerEnabled(enabled: boolean): Promise<Result<any>>
  audioSetEqBand(band: number, gainDb: number): Promise<Result<any>>
  audioSetEqEnabled(enabled: boolean): Promise<Result<any>>
  audioSetVolume(volume: number): Promise<Result<any>>
  audioStop(): Promise<Result<any>>
  clearCache(): Promise<Result<any>>
  clearImageCache(): Promise<Result<any>>
  clearImageFromCache(itemId: string, imageType: string): Promise<Result<any>>
  clearSavedCredentials(): Promise<Result<any>>
  createPlaylist(data: PlaylistCreateData): Promise<Result<any>>
  deletePlaylist(playlistId: string): Promise<Result<any>>
  discordRpcClearActivity(): Promise<Result<any>>
  discordRpcIsRunning(): Promise<Result<any>>
  discordRpcSetActivity(activity: RpcActivity): Promise<Result<any>>
  discordRpcStart(appId: string): Promise<Result<any>>
  discordRpcStop(): Promise<Result<any>>
  getAlbum(albumId: string): Promise<Result<any>>
  getAlbumShareUrls(albumId: string): Promise<Result<any>>
  getArtist(artistId: string): Promise<Result<any>>
  getArtistShareUrls(artistId: string): Promise<Result<any>>
  getAudioStreamUrl(itemId: string, serverUrl: string, token: string, container?: string): Promise<Result<any>>
  getHomeViewData(): Promise<Result<any>>
  getImage(itemId: string, imageType: string, serverUrl: string, token: string, width?: number, quality?: number): Promise<Result<any>>
  getImageCacheStats(): Promise<Result<any>>
  getInstantMix(itemId: string): Promise<Result<any>>
  getLibrary(): Promise<Result<any>>
  getLyrics(id: string, artist: string, title: string, path?: string): Promise<Result<any>>
  getPlaylistItems(playlistId: string): Promise<Result<any>>
  getPlaylists(): Promise<Result<any>>
  getRecentlyPlayed(): Promise<Result<any>>
  getRelatedArtists(artistId: string): Promise<Result<any>>
  getSavedCredentials(): Promise<Result<any>>
  getSavedVolume(): Promise<Result<any>>
  getSong(songId: string): Promise<Result<any>>
  getSongShareUrls(itemId: string): Promise<Result<any>>
  getSyncState(): Promise<Result<any>>
  hideMainWindow(): Promise<Result<any>>
  lastfmAuthenticate(): Promise<Result<any>>
  lastfmClearCredentials(): Promise<Result<any>>
  lastfmIsAuthenticated(): Promise<Result<any>>
  lastfmScrobble(artist: string, track: string, album?: string, timestamp?: number): Promise<Result<any>>
  lastfmSetCredentials(credentials: LastFmCredentials): Promise<Result<any>>
  lastfmStartAuthServer(): Promise<Result<any>>
  lastfmUpdateNowPlaying(artist: string, track: string, album?: string): Promise<Result<any>>
  listenbrainzClearCredentials(): Promise<Result<any>>
  listenbrainzIsAuthenticated(): Promise<Result<any>>
  listenbrainzPlayingNow(artist: string, track: string, album?: string): Promise<Result<any>>
  listenbrainzSetCredentials(credentials: ListenBrainzCredentials): Promise<Result<any>>
  listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number): Promise<Result<any>>
  listenbrainzValidateToken(userToken: string): Promise<Result<any>>
  loginToJellyfin(serverUrl: string, username: string, password: string, deviceId: string): Promise<Result<any>>
  markItemPlayed(itemId: string): Promise<Result<any>>
  mediaClearNowPlaying(): Promise<Result<any>>
  mediaSetButtonEnabled(button: string, enabled: boolean): Promise<Result<any>>
  mediaSetPlaybackStatus(isPlaying: boolean, positionSecs?: number): Promise<Result<any>>
  mediaUpdateNowPlaying(payload: NowPlayingPayload): Promise<Result<any>>
  quitApplication(): Promise<Result<any>>
  registerClientCapabilities(serverUrl: string, token: string, deviceId: string): Promise<Result<any>>
  removePlaylistItems(playlistId: string, songIds: string[]): Promise<Result<any>>
  reportPlaybackProgress(itemId: string, positionTicks: number, isPaused: boolean): Promise<Result<any>>
  reportPlaybackStart(itemId: string, positionTicks?: number): Promise<Result<any>>
  reportPlaybackStop(itemId: string, positionTicks: number): Promise<Result<any>>
  saveCredentials(serverUrl: string, username: string, token: string, userId: string): Promise<Result<any>>
  saveVolume(volume: number): Promise<Result<any>>
  setCloseToTray(closeToTray: boolean): Promise<Result<any>>
  setMinimizeToTray(minimizeToTray: boolean): Promise<Result<any>>
  showMainWindow(): Promise<Result<any>>
  syncLibrary(): Promise<Result<any>>
  toggleFavoriteStatus(itemId: string, isFavorite: boolean): Promise<Result<any>>
  updatePlaylist(playlistId: string, updates: PlaylistUpdateData): Promise<Result<any>>; }

// Re-export model types from generated
export * from '../../generated'
