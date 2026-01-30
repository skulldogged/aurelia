// Tauri API Client Implementation
// Wraps specta-generated bindings to implement the shared ApiClient interface

import type {
  ApiClient,
  AudioStreamParams,
  Credentials,
  EQBand,
  EQPreset,
  ImageParams,
  LastFmCredentials,
  ListenBrainzCredentials,
  ListenBrainzListen,
  PlaylistCreateData,
  PlaylistUpdateData,
  Result,
  ShareUrlType,
} from './types'

// Import Tauri commands (will be provided by the desktop app)
// This import path is set up by the desktop app that uses this client
declare const TAURI_COMMANDS: {
  getSavedCredentials(): Promise<Result<Credentials | null, string>>
  authenticate(serverUrl: string, username: string, password: string): Promise<Result<Credentials, string>>
  logout(): Promise<Result<void, string>>
  getLibrary(): Promise<Result<any, string>>
  syncLibrary(): Promise<Result<void, string>>
  getSyncState(): Promise<Result<any, string>>
  getSong(songId: string): Promise<Result<any, string>>
  getPlaylists(): Promise<Result<any[], string>>
  getPlaylistItems(playlistId: string): Promise<Result<any[], string>>
  createPlaylist(data: PlaylistCreateData): Promise<Result<any, string>>
  updatePlaylist(playlistId: string, updates: PlaylistUpdateData): Promise<Result<any, string>>
  deletePlaylist(playlistId: string): Promise<Result<void, string>>
  addPlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<void, string>>
  removePlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<void, string>>
  getHomeViewData(): Promise<Result<any, string>>
  getRecentlyPlayed(serverUrl: string, token: string, userId: string): Promise<Result<any[], string>>
  getAudioStreamUrl(serverUrl: string, token: string, itemId: string, container: null | string): Promise<Result<string, string>>
  getSavedVolume(): Promise<Result<number | null, string>>
  saveVolume(volume: number): Promise<Result<void, string>>
  getImage(itemId: string, imageType: string, serverUrl: string, token: string, width: null | number, quality: null | number): Promise<Result<string | null, string>>
  clearCache(serverUrl: string, token: string): Promise<Result<void, string>>
  clearImageFromCache(itemId: string, imageType: string): Promise<Result<void, string>>
  getImageCacheStats(): Promise<Result<string, string>>
  getLyrics(id: string, artist: string, title: string, path: null | string): Promise<Result<string, string>>
  toggleFavoriteStatus(itemId: string, isFavorite: boolean): Promise<Result<boolean, string>>
  getInstantMix(itemId: string): Promise<Result<any[], string>>
  getRelatedArtists(artistId: string): Promise<Result<any[], string>>
  getSongShareUrls(songId: string): Promise<Result<Partial<Record<ShareUrlType, string>>, string>>
  lastfmSetCredentials(credentials: LastFmCredentials): Promise<Result<void, string>>
  lastfmClearCredentials(): Promise<Result<void, string>>
  lastfmIsAuthenticated(): Promise<Result<boolean, string>>
  lastfmAuthenticate(): Promise<Result<LastFmCredentials, string>>
  lastfmScrobble(artist: string, track: string, album: null | string, timestamp: number): Promise<Result<void, string>>
  lastfmUpdateNowPlaying(artist: string, track: string, album: null | string): Promise<Result<void, string>>
  listenbrainzSetCredentials(credentials: ListenBrainzCredentials): Promise<Result<void, string>>
  listenbrainzClearCredentials(): Promise<Result<void, string>>
  listenbrainzIsAuthenticated(): Promise<Result<boolean, string>>
  listenbrainzValidateToken(userToken: string): Promise<Result<ListenBrainzCredentials, string>>
  listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number): Promise<Result<void, string>>
  listenbrainzPlayingNow(artist: string, track: string, album: null | string): Promise<Result<void, string>>
  saveCredentials(serverUrl: string, username: string, token: string, userId: string): Promise<Result<void, string>>
  // Audio (Rust player)
  audioInit(): Promise<Result<void, string>>
  audioPlay(streamUrl: string): Promise<Result<void, string>>
  audioPause(): Promise<Result<void, string>>
  audioResume(): Promise<Result<void, string>>
  audioStop(): Promise<Result<void, string>>
  audioGetPosition(): Promise<Result<number, string>>
  audioSeek(position: number): Promise<Result<void, string>>
  audioGetVolume(): Promise<Result<number, string>>
  audioSetVolume(volume: number): Promise<Result<void, string>>
  audioSetEqBand(band: number, gain: number): Promise<Result<void, string>>
  audioGetEqBand(band: number): Promise<Result<number, string>>
  audioGetAllEqBands(): Promise<Result<number[], string>>
  audioSetEqEnabled(enabled: boolean): Promise<Result<void, string>>
  audioIsEqEnabled(): Promise<Result<boolean, string>>
  audioSetEqPreset(preset: EQPreset): Promise<Result<void, string>>
  audioGetEqPreset(): Promise<Result<EQPreset, string>>
  audioIsAnalyzerEnabled(): Promise<Result<boolean, string>>
  audioSetAnalyzerEnabled(enabled: boolean): Promise<Result<void, string>>
  audioPrepareNext(streamUrl: string): Promise<Result<void, string>>
  audioAdvanceGapless(): Promise<Result<void, string>>
  // Session
  reportPlaybackStart(itemId: string, position?: number): Promise<Result<void, string>>
  reportPlaybackProgress(itemId: string, position: number, isPaused: boolean): Promise<Result<void, string>>
  reportPlaybackStop(itemId: string, position: number): Promise<Result<void, string>>
  markItemPlayed(itemId: string): Promise<Result<void, string>>
  // System Tray / Window Management
  showMainWindow(): Promise<Result<void, string>>
  hideMainWindow(): Promise<Result<void, string>>
  quitApplication(): Promise<Result<void, string>>
  setMinimizeToTray(minimizeToTray: boolean): Promise<Result<void, string>>
  setCloseToTray(closeToTray: boolean): Promise<Result<void, string>>
}

// Tauri API Client factory function
export function createTauriClient(commands: typeof TAURI_COMMANDS): ApiClient {
  return {
    // Auth
    getSavedCredentials: () => commands.getSavedCredentials(),
    authenticate: (serverUrl, username, password) =>
      commands.authenticate(serverUrl, username, password),
    logout: () => commands.logout(),
    saveCredentials: (serverUrl, username, token, userId) =>
      commands.saveCredentials(serverUrl, username, token, userId),

    // Library
    getLibrary: () => commands.getLibrary(),
    syncLibrary: () => commands.syncLibrary(),
    getSyncState: () => commands.getSyncState(),
    getSong: (songId) => commands.getSong(songId),

    // Playlists
    getPlaylists: () => commands.getPlaylists(),
    getPlaylistItems: (playlistId) => commands.getPlaylistItems(playlistId),
    createPlaylist: (data) => commands.createPlaylist(data),
    updatePlaylist: (playlistId, updates) => commands.updatePlaylist(playlistId, updates),
    deletePlaylist: (playlistId) => commands.deletePlaylist(playlistId),
    addPlaylistItems: (playlistId, itemIds) => commands.addPlaylistItems(playlistId, itemIds),
    removePlaylistItems: (playlistId, itemIds) => commands.removePlaylistItems(playlistId, itemIds),

    // Home
    getHomeViewData: () => commands.getHomeViewData(),
    getRecentlyPlayed: (serverUrl, token, userId) =>
      commands.getRecentlyPlayed(serverUrl, token, userId),

    // Audio
    getAudioStreamUrl: (params) =>
      commands.getAudioStreamUrl(params.serverUrl, params.token, params.itemId, params.container ?? null),
    getSavedVolume: () => commands.getSavedVolume(),
    saveVolume: (volume) => commands.saveVolume(volume),

    // Images
    getImage: (params) =>
      commands.getImage(
        params.itemId,
        params.imageType,
        params.serverUrl,
        params.token,
        params.width ?? null,
        params.quality ?? null,
      ),
    clearCache: (serverUrl, token) => commands.clearCache(serverUrl, token),
    clearImageFromCache: (itemId, imageType) => commands.clearImageFromCache(itemId, imageType),
    getImageCacheStats: () => commands.getImageCacheStats(),

    // Lyrics
    getLyrics: (id, artist, title, path) => commands.getLyrics(id, artist, title, path ?? null),

    // Favorites
    toggleFavoriteStatus: (itemId, isFavorite) =>
      commands.toggleFavoriteStatus(itemId, isFavorite),

    // Instant Mix
    getInstantMix: (itemId) => commands.getInstantMix(itemId),

    // Related Artists
    getRelatedArtists: (artistId) => commands.getRelatedArtists(artistId),

    // Share URLs
    getSongShareUrls: (songId) => commands.getSongShareUrls(songId),

    // Last.fm
    lastfmSetCredentials: (credentials) => commands.lastfmSetCredentials(credentials),
    lastfmClearCredentials: () => commands.lastfmClearCredentials(),
    lastfmIsAuthenticated: () => commands.lastfmIsAuthenticated(),
    lastfmAuthenticate: () => commands.lastfmAuthenticate(),
    lastfmScrobble: (artist, track, album, timestamp) =>
      commands.lastfmScrobble(artist, track, album ?? null, timestamp ?? Date.now()),
    lastfmUpdateNowPlaying: (artist, track, album) =>
      commands.lastfmUpdateNowPlaying(artist, track, album ?? null),

    // ListenBrainz
    listenbrainzSetCredentials: (credentials) => commands.listenbrainzSetCredentials(credentials),
    listenbrainzClearCredentials: () => commands.listenbrainzClearCredentials(),
    listenbrainzIsAuthenticated: () => commands.listenbrainzIsAuthenticated(),
    listenbrainzValidateToken: (userToken) => commands.listenbrainzValidateToken(userToken),
    listenbrainzSubmitListen: (listen, timestamp) =>
      commands.listenbrainzSubmitListen(listen, timestamp),
    listenbrainzPlayingNow: (artist, track, album) =>
      commands.listenbrainzPlayingNow(artist, track, album ?? null),

    // Audio (Rust player - desktop only)
    audioInit: () => commands.audioInit(),
    audioPlay: (streamUrl) => commands.audioPlay(streamUrl),
    audioPause: () => commands.audioPause(),
    audioResume: () => commands.audioResume(),
    audioStop: () => commands.audioStop(),
    audioGetPosition: () => commands.audioGetPosition(),
    audioSeek: (position) => commands.audioSeek(position),
    audioGetVolume: () => commands.audioGetVolume(),
    audioSetVolume: (volume) => commands.audioSetVolume(volume),
    audioSetEqBand: (band, gain) => commands.audioSetEqBand(band, gain),
    audioGetEqBand: (band) => commands.audioGetEqBand(band),
    audioGetAllEqBands: () => commands.audioGetAllEqBands(),
    audioSetEqEnabled: (enabled) => commands.audioSetEqEnabled(enabled),
    audioIsEqEnabled: () => commands.audioIsEqEnabled(),
    audioSetEqPreset: (preset) => commands.audioSetEqPreset(preset),
    audioGetEqPreset: () => commands.audioGetEqPreset(),
    audioIsAnalyzerEnabled: () => commands.audioIsAnalyzerEnabled(),
    audioSetAnalyzerEnabled: (enabled) => commands.audioSetAnalyzerEnabled(enabled),
    audioPrepareNext: (streamUrl) => commands.audioPrepareNext(streamUrl),
    audioAdvanceGapless: () => commands.audioAdvanceGapless(),

    // Session
    reportPlaybackStart: (itemId, position) =>
      commands.reportPlaybackStart(itemId, position),
    reportPlaybackProgress: (itemId, position, isPaused) =>
      commands.reportPlaybackProgress(itemId, position, isPaused),
    reportPlaybackStop: (itemId, position) =>
      commands.reportPlaybackStop(itemId, position),
    markItemPlayed: (itemId) => commands.markItemPlayed(itemId),

    // System Tray / Window Management
    showMainWindow: () => commands.showMainWindow(),
    hideMainWindow: () => commands.hideMainWindow(),
    quitApplication: () => commands.quitApplication(),
    setMinimizeToTray: (minimizeToTray) => commands.setMinimizeToTray(minimizeToTray),
    setCloseToTray: (closeToTray) => commands.setCloseToTray(closeToTray),
  }
}

export type { ApiClient, Result }
export * from './types'
