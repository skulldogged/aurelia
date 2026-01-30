// Tauri API Client Implementation
// Wraps specta-generated bindings to implement the shared ApiClient interface

import type {
  ApiClient,
  Artist,
  AudioStreamParams,
  Credentials,
  EQPreset,
  HomeViewData,
  ImageParams,
  LastFmCredentials,
  LibraryData,
  ListenBrainzCredentials,
  ListenBrainzListen,
  NowPlayingPayload,
  Playlist,
  PlaylistCreateData,
  PlaylistUpdateData,
  Result,
  RpcActivity,
  ShareUrlType,
  Song,
  SyncStateInfo,
} from './types'

// The commands object from bindings.ts - passed in at runtime
// This interface matches the actual tauri-specta generated bindings
interface TauriCommands {
  // Auth
  getSavedCredentials(): Promise<Result<Credentials | null, string>>
  loginToJellyfin(serverUrl: string, username: string, password: string): Promise<Result<{ token: string; userId: string }, string>>
  saveCredentials(serverUrl: string, username: string, token: string, userId: string): Promise<Result<null, string>>
  clearSavedCredentials(): Promise<Result<null, string>>

  // Library
  getLibrary(): Promise<Result<LibraryData, string>>
  syncLibrary(serverUrl: string, token: string): Promise<Result<null, string>>
  getSyncState(): Promise<Result<SyncStateInfo, string>>
  getSong(songId: string): Promise<Result<Song, string>>
  getArtist(artistId: string, includeSongs: boolean | null): Promise<Result<Artist, string>>
  getAlbum(albumId: string, includeSongs: boolean | null): Promise<Result<unknown, string>>

  // Playlists
  getPlaylists(): Promise<Result<Playlist[], string>>
  getPlaylistItems(playlistId: string): Promise<Result<Song[], string>>
  createPlaylist(data: PlaylistCreateData): Promise<Result<Playlist, string>>
  updatePlaylist(playlistId: string, updates: PlaylistUpdateData): Promise<Result<Playlist, string>>
  deletePlaylist(playlistId: string): Promise<Result<null, string>>
  addPlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<null, string>>
  removePlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<null, string>>

  // Home
  getHomeViewData(): Promise<Result<HomeViewData, string>>
  getRecentlyPlayed(serverUrl: string, token: string, userId: string): Promise<Result<Song[], string>>

  // Audio
  getAudioStreamUrl(serverUrl: string, token: string, itemId: string, container: string | null): Promise<Result<string, string>>
  getSavedVolume(): Promise<Result<number | null, string>>
  saveVolume(volume: number): Promise<Result<null, string>>

  // Images
  getImage(itemId: string, imageType: string, serverUrl: string, token: string, width: number | null, quality: number | null): Promise<Result<string | null, string>>
  clearCache(serverUrl: string, token: string): Promise<Result<null, string>>
  clearImageFromCache(itemId: string, imageType: string): Promise<Result<null, string>>
  getImageCacheStats(): Promise<Result<string, string>>

  // Lyrics
  getLyrics(id: string, artist: string, title: string, path: string | null): Promise<Result<string, string>>

  // Favorites
  toggleFavoriteStatus(serverUrl: string, token: string, userId: string, itemId: string, isFavorite: boolean): Promise<Result<boolean, string>>

  // Instant Mix
  getInstantMix(itemId: string): Promise<Result<Song[], string>>

  // Related Artists
  getRelatedArtists(artistId: string): Promise<Result<Artist[], string>>

  // Share URLs
  getSongShareUrls(songId: string): Promise<Result<Partial<Record<string, string>>, string>>

  // Last.fm
  lastfmSetCredentials(credentials: LastFmCredentials): Promise<Result<null, string>>
  lastfmClearCredentials(): Promise<Result<null, string>>
  lastfmIsAuthenticated(): Promise<Result<boolean, string>>
  lastfmAuthenticate(): Promise<Result<LastFmCredentials, string>>
  lastfmScrobble(artist: string, track: string, album: string | null, timestamp: number): Promise<Result<null, string>>
  lastfmUpdateNowPlaying(artist: string, track: string, album: string | null): Promise<Result<null, string>>

  // ListenBrainz
  listenbrainzSetCredentials(credentials: ListenBrainzCredentials): Promise<Result<null, string>>
  listenbrainzClearCredentials(): Promise<Result<null, string>>
  listenbrainzIsAuthenticated(): Promise<Result<boolean, string>>
  listenbrainzValidateToken(userToken: string): Promise<Result<ListenBrainzCredentials, string>>
  listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number): Promise<Result<null, string>>
  listenbrainzPlayingNow(artist: string, track: string, album: string | null): Promise<Result<null, string>>

  // Audio (Rust player)
  audioInit(): Promise<Result<null, string>>
  audioPlay(url: string, token: string): Promise<Result<null, string>>
  audioPause(): Promise<Result<null, string>>
  audioResume(): Promise<Result<null, string>>
  audioStop(): Promise<Result<null, string>>
  audioGetPosition(): Promise<Result<number, string>>
  audioSeek(positionSecs: number): Promise<Result<null, string>>
  audioGetVolume(): Promise<Result<number, string>>
  audioSetVolume(volume: number): Promise<Result<null, string>>
  audioSetEqBand(band: number, gainDb: number): Promise<Result<null, string>>
  audioGetEqBand(band: number): Promise<Result<number, string>>
  audioGetAllEqBands(): Promise<Result<number[], string>>
  audioSetEqEnabled(enabled: boolean): Promise<Result<null, string>>
  audioIsEqEnabled(): Promise<Result<boolean, string>>
  audioSetEqPreset?(preset: EQPreset): Promise<Result<null, string>>
  audioGetEqPreset?(): Promise<Result<EQPreset, string>>
  audioIsAnalyzerEnabled(): Promise<Result<boolean, string>>
  audioSetAnalyzerEnabled(enabled: boolean): Promise<Result<null, string>>
  audioPrepareNext(url: string, token: string): Promise<Result<null, string>>
  audioAdvanceGapless(): Promise<Result<null, string>>

  // Session
  registerClientCapabilities(serverUrl: string, token: string, deviceId: string): Promise<Result<null, string>>
  reportPlaybackStart(serverUrl: string, token: string, itemId: string, positionTicks: number | null): Promise<Result<null, string>>
  reportPlaybackProgress(serverUrl: string, token: string, itemId: string, positionTicks: number | null, eventName: string | null, isPaused: boolean | null): Promise<Result<null, string>>
  reportPlaybackStop(serverUrl: string, token: string, itemId: string, positionTicks: number | null): Promise<Result<null, string>>
  markItemPlayed(serverUrl: string, token: string, userId: string, itemId: string): Promise<Result<null, string>>

  // System Tray / Window Management
  showMainWindow(): Promise<void>
  hideMainWindow(): Promise<void>
  quitApplication(): Promise<void>
  setMinimizeToTray(minimizeToTray: boolean): Promise<void>
  setCloseToTray(closeToTray: boolean): Promise<void>

  // Discord Rich Presence
  discordRpcIsRunning(): Promise<Result<boolean, string>>
  discordRpcStart(appId: string): Promise<Result<null, string>>
  discordRpcStop(): Promise<Result<null, string>>
  discordRpcSetActivity(activity: RpcActivity): Promise<Result<null, string>>
  discordRpcClearActivity(): Promise<Result<null, string>>

  // Media Controls
  mediaClearNowPlaying(): Promise<Result<null, string>>
  mediaSetPlaybackStatus(isPlaying: boolean, positionSecs: number | null): Promise<Result<null, string>>
  mediaUpdateNowPlaying(payload: NowPlayingPayload): Promise<Result<null, string>>
  mediaSetButtonEnabled(button: string, enabled: boolean): Promise<Result<null, string>>
}

// Helper to convert null results to void
const nullToVoid = <E>(result: Result<null, E>): Result<void, E> => {
  if (result.status === 'ok') {
    return { status: 'ok', data: undefined }
  }
  return result
}

// Helper to wrap void-returning functions as Result
const wrapVoid = async (fn: () => Promise<void>): Promise<Result<void, string>> => {
  try {
    await fn()
    return { status: 'ok', data: undefined }
  } catch (e) {
    return { status: 'error', error: e instanceof Error ? e.message : String(e) }
  }
}

// Tauri API Client factory function
export function createTauriClient(commands: TauriCommands): ApiClient {
  // Store for credentials - populated after login
  let cachedCredentials: Credentials | null = null

  const getCredentials = (): Credentials | null => cachedCredentials

  const requireCredentials = (): Credentials => {
    if (!cachedCredentials) {
      throw new Error('Not authenticated')
    }
    return cachedCredentials
  }

  return {
    // Auth
    getSavedCredentials: async () => {
      const result = await commands.getSavedCredentials()
      if (result.status === 'ok' && result.data) {
        cachedCredentials = result.data
      }
      return result
    },

    authenticate: async (serverUrl, username, password) => {
      const result = await commands.loginToJellyfin(serverUrl, username, password)
      if (result.status === 'ok') {
        cachedCredentials = {
          serverUrl,
          username,
          token: result.data.token,
          userId: result.data.userId,
        }
        return { status: 'ok', data: cachedCredentials }
      }
      return result
    },

    logout: async () => {
      const result = await commands.clearSavedCredentials()
      cachedCredentials = null
      return nullToVoid(result)
    },

    saveCredentials: async (serverUrl, username, token, userId) => {
      const result = await commands.saveCredentials(serverUrl, username, token, userId)
      if (result.status === 'ok') {
        cachedCredentials = { serverUrl, username, token, userId }
      }
      return nullToVoid(result)
    },

    // Library
    getLibrary: () => commands.getLibrary(),
    syncLibrary: async () => {
      const creds = requireCredentials()
      return nullToVoid(await commands.syncLibrary(creds.serverUrl, creds.token))
    },
    getSyncState: () => commands.getSyncState(),
    getSong: (songId) => commands.getSong(songId),
    getArtist: (artistId) => commands.getArtist(artistId, true),

    // Playlists
    getPlaylists: () => commands.getPlaylists(),
    getPlaylistItems: (playlistId) => commands.getPlaylistItems(playlistId),
    createPlaylist: (data) => commands.createPlaylist(data),
    updatePlaylist: (playlistId, updates) => commands.updatePlaylist(playlistId, updates),
    deletePlaylist: async (playlistId) => nullToVoid(await commands.deletePlaylist(playlistId)),
    addPlaylistItems: async (playlistId, itemIds) => nullToVoid(await commands.addPlaylistItems(playlistId, itemIds)),
    removePlaylistItems: async (playlistId, itemIds) => nullToVoid(await commands.removePlaylistItems(playlistId, itemIds)),

    // Home
    getHomeViewData: () => commands.getHomeViewData(),
    getRecentlyPlayed: (serverUrl, token, userId) => commands.getRecentlyPlayed(serverUrl, token, userId),

    // Audio
    getAudioStreamUrl: (params) =>
      commands.getAudioStreamUrl(params.serverUrl, params.token, params.itemId, params.container ?? null),
    getSavedVolume: () => commands.getSavedVolume(),
    saveVolume: async (volume) => nullToVoid(await commands.saveVolume(volume)),

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
    clearCache: async (serverUrl, token) => nullToVoid(await commands.clearCache(serverUrl, token)),
    clearImageFromCache: async (itemId, imageType) => nullToVoid(await commands.clearImageFromCache(itemId, imageType)),
    getImageCacheStats: () => commands.getImageCacheStats(),

    // Lyrics
    getLyrics: (id, artist, title, path) => commands.getLyrics(id, artist, title, path ?? null),

    // Favorites
    toggleFavoriteStatus: async (itemId, isFavorite) => {
      const creds = requireCredentials()
      return commands.toggleFavoriteStatus(creds.serverUrl, creds.token, creds.userId, itemId, isFavorite)
    },

    // Instant Mix
    getInstantMix: (itemId) => commands.getInstantMix(itemId),

    // Related Artists
    getRelatedArtists: (artistId) => commands.getRelatedArtists(artistId),

    // Share URLs
    getSongShareUrls: (songId) => commands.getSongShareUrls(songId),

    // Last.fm
    lastfmSetCredentials: async (credentials) => nullToVoid(await commands.lastfmSetCredentials(credentials)),
    lastfmClearCredentials: async () => nullToVoid(await commands.lastfmClearCredentials()),
    lastfmIsAuthenticated: () => commands.lastfmIsAuthenticated(),
    lastfmAuthenticate: () => commands.lastfmAuthenticate(),
    lastfmScrobble: async (artist, track, album, timestamp) =>
      nullToVoid(await commands.lastfmScrobble(artist, track, album ?? null, timestamp ?? Date.now())),
    lastfmUpdateNowPlaying: async (artist, track, album) =>
      nullToVoid(await commands.lastfmUpdateNowPlaying(artist, track, album ?? null)),

    // ListenBrainz
    listenbrainzSetCredentials: async (credentials) => nullToVoid(await commands.listenbrainzSetCredentials(credentials)),
    listenbrainzClearCredentials: async () => nullToVoid(await commands.listenbrainzClearCredentials()),
    listenbrainzIsAuthenticated: () => commands.listenbrainzIsAuthenticated(),
    listenbrainzValidateToken: (userToken) => commands.listenbrainzValidateToken(userToken),
    listenbrainzSubmitListen: async (listen, timestamp) =>
      nullToVoid(await commands.listenbrainzSubmitListen(listen, timestamp)),
    listenbrainzPlayingNow: async (artist, track, album) =>
      nullToVoid(await commands.listenbrainzPlayingNow(artist, track, album ?? null)),

    // Audio (Rust player - desktop only)
    audioInit: async () => nullToVoid(await commands.audioInit()),
    audioPlay: async (streamUrl) => {
      const creds = getCredentials()
      return nullToVoid(await commands.audioPlay(streamUrl, creds?.token ?? ''))
    },
    audioPause: async () => nullToVoid(await commands.audioPause()),
    audioResume: async () => nullToVoid(await commands.audioResume()),
    audioStop: async () => nullToVoid(await commands.audioStop()),
    audioGetPosition: () => commands.audioGetPosition(),
    audioSeek: async (position) => nullToVoid(await commands.audioSeek(position)),
    audioGetVolume: () => commands.audioGetVolume(),
    audioSetVolume: async (volume) => nullToVoid(await commands.audioSetVolume(volume)),
    audioSetEqBand: async (band, gain) => nullToVoid(await commands.audioSetEqBand(band, gain)),
    audioGetEqBand: (band) => commands.audioGetEqBand(band),
    audioGetAllEqBands: () => commands.audioGetAllEqBands(),
    audioSetEqEnabled: async (enabled) => nullToVoid(await commands.audioSetEqEnabled(enabled)),
    audioIsEqEnabled: () => commands.audioIsEqEnabled(),
    audioSetEqPreset: commands.audioSetEqPreset
      ? async (preset) => nullToVoid(await commands.audioSetEqPreset!(preset))
      : undefined,
    audioGetEqPreset: commands.audioGetEqPreset
      ? () => commands.audioGetEqPreset!()
      : undefined,
    audioIsAnalyzerEnabled: () => commands.audioIsAnalyzerEnabled(),
    audioSetAnalyzerEnabled: async (enabled) => nullToVoid(await commands.audioSetAnalyzerEnabled(enabled)),
    audioPrepareNext: async (streamUrl) => {
      const creds = getCredentials()
      return nullToVoid(await commands.audioPrepareNext(streamUrl, creds?.token ?? ''))
    },
    audioAdvanceGapless: async () => nullToVoid(await commands.audioAdvanceGapless()),

    // Session
    registerClientCapabilities: async (serverUrl, token, deviceId) =>
      nullToVoid(await commands.registerClientCapabilities(serverUrl, token, deviceId)),
    reportPlaybackStart: async (itemId, position) => {
      const creds = requireCredentials()
      const positionTicks = position ? Math.round(position * 10_000_000) : null
      return nullToVoid(await commands.reportPlaybackStart(creds.serverUrl, creds.token, itemId, positionTicks))
    },
    reportPlaybackProgress: async (itemId, position, isPaused) => {
      const creds = requireCredentials()
      const positionTicks = Math.round(position * 10_000_000)
      return nullToVoid(await commands.reportPlaybackProgress(creds.serverUrl, creds.token, itemId, positionTicks, null, isPaused))
    },
    reportPlaybackStop: async (itemId, position) => {
      const creds = requireCredentials()
      const positionTicks = Math.round(position * 10_000_000)
      return nullToVoid(await commands.reportPlaybackStop(creds.serverUrl, creds.token, itemId, positionTicks))
    },
    markItemPlayed: async (itemId) => {
      const creds = requireCredentials()
      return nullToVoid(await commands.markItemPlayed(creds.serverUrl, creds.token, creds.userId, itemId))
    },

    // System Tray / Window Management
    showMainWindow: () => wrapVoid(() => commands.showMainWindow()),
    hideMainWindow: () => wrapVoid(() => commands.hideMainWindow()),
    quitApplication: () => wrapVoid(() => commands.quitApplication()),
    setMinimizeToTray: (minimizeToTray) => wrapVoid(() => commands.setMinimizeToTray(minimizeToTray)),
    setCloseToTray: (closeToTray) => wrapVoid(() => commands.setCloseToTray(closeToTray)),

    // Discord Rich Presence
    discordRpcIsRunning: () => commands.discordRpcIsRunning(),
    discordRpcStart: async (appId) => nullToVoid(await commands.discordRpcStart(appId)),
    discordRpcStop: async () => nullToVoid(await commands.discordRpcStop()),
    discordRpcSetActivity: async (activity) => nullToVoid(await commands.discordRpcSetActivity(activity)),
    discordRpcClearActivity: async () => nullToVoid(await commands.discordRpcClearActivity()),

    // Media Controls
    mediaClearNowPlaying: async () => nullToVoid(await commands.mediaClearNowPlaying()),
    mediaSetPlaybackStatus: async (isPlaying, positionSecs) =>
      nullToVoid(await commands.mediaSetPlaybackStatus(isPlaying, positionSecs)),
    mediaUpdateNowPlaying: async (payload) => nullToVoid(await commands.mediaUpdateNowPlaying(payload)),
    mediaSetButtonEnabled: async (button, enabled) =>
      nullToVoid(await commands.mediaSetButtonEnabled(button, enabled)),
  }
}

export type { ApiClient, Result }
export * from './types'
