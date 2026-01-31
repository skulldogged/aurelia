// Tauri API Client Implementation
// Wraps specta-generated bindings to implement the shared ApiClient interface

import type {
  ApiClient,
  Artist,
  Credentials,
  EQPreset,
  HomeViewData,
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
  Song,
  SyncStateInfo,
} from './types'

// The commands object from bindings.ts - passed in at runtime
// This interface matches the actual tauri-specta generated bindings
interface TauriCommands {
  addPlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<null, string>>
  audioAdvanceGapless(): Promise<Result<null, string>>
  audioGetAllEqBands(): Promise<Result<number[], string>>
  audioGetEqBand(band: number): Promise<Result<number, string>>

  audioGetEqPreset?(): Promise<Result<EQPreset, string>>
  audioGetPosition(): Promise<Result<number, string>>
  audioGetVolume(): Promise<Result<number, string>>
  // Audio (Rust player)
  audioInit(): Promise<Result<null, string>>
  audioIsAnalyzerEnabled(): Promise<Result<boolean, string>>
  audioIsEqEnabled(): Promise<Result<boolean, string>>

  audioPause(): Promise<Result<null, string>>
  audioPlay(url: string, token: string): Promise<Result<null, string>>
  audioPrepareNext(url: string, token: string): Promise<Result<null, string>>
  audioResume(): Promise<Result<null, string>>
  audioSeek(positionSecs: number): Promise<Result<null, string>>
  audioSetAnalyzerEnabled(enabled: boolean): Promise<Result<null, string>>
  audioSetEqBand(band: number, gainDb: number): Promise<Result<null, string>>

  audioSetEqEnabled(enabled: boolean): Promise<Result<null, string>>
  audioSetEqPreset?(preset: EQPreset): Promise<Result<null, string>>

  audioSetVolume(volume: number): Promise<Result<null, string>>
  audioStop(): Promise<Result<null, string>>
  clearCache(serverUrl: string, token: string): Promise<Result<null, string>>

  clearImageFromCache(itemId: string, imageType: string): Promise<Result<null, string>>
  clearSavedCredentials(): Promise<Result<null, string>>
  createPlaylist(data: PlaylistCreateData): Promise<Result<Playlist, string>>
  deletePlaylist(playlistId: string): Promise<Result<null, string>>

  discordRpcClearActivity(): Promise<Result<null, string>>

  // Discord Rich Presence
  discordRpcIsRunning(): Promise<Result<boolean, string>>

  discordRpcSetActivity(activity: RpcActivity): Promise<Result<null, string>>

  discordRpcStart(appId: string): Promise<Result<null, string>>

  discordRpcStop(): Promise<Result<null, string>>

  getAlbum(albumId: string, includeSongs: boolean | null): Promise<Result<unknown, string>>
  getArtist(artistId: string, includeSongs: boolean | null): Promise<Result<Artist, string>>
  // Audio
  getAudioStreamUrl(
    serverUrl: string,
    token: string,
    itemId: string,
    container: null | string,
  ): Promise<Result<string, string>>
  // Home
  getHomeViewData(): Promise<Result<HomeViewData, string>>
  // Images
  getImage(
    itemId: string,
    imageType: string,
    serverUrl: string,
    token: string,
    width: null | number,
    quality: null | number,
  ): Promise<Result<null | string, string>>
  getImageCacheStats(): Promise<Result<string, string>>

  // Instant Mix
  getInstantMix(itemId: string): Promise<Result<Song[], string>>
  // Library
  getLibrary(): Promise<Result<LibraryData, string>>
  // Lyrics
  getLyrics(id: string, artist: string, title: string, path: null | string): Promise<Result<string, string>>
  getPlaylistItems(playlistId: string): Promise<Result<Song[], string>>
  // Playlists
  getPlaylists(): Promise<Result<Playlist[], string>>
  getRecentlyPlayed(serverUrl: string, token: string, userId: string): Promise<Result<Song[], string>>

  // Related Artists
  getRelatedArtists(artistId: string): Promise<Result<Artist[], string>>
  // Auth
  getSavedCredentials(): Promise<Result<Credentials | null, string>>
  getSavedVolume(): Promise<Result<null | number, string>>
  getSong(songId: string): Promise<Result<Song, string>>
  // Share URLs
  getSongShareUrls(songId: string): Promise<Result<Partial<Record<string, string>>, string>>
  getSyncState(): Promise<Result<SyncStateInfo, string>>
  hideMainWindow(): Promise<void>
  lastfmAuthenticate(): Promise<Result<LastFmCredentials, string>>
  lastfmClearCredentials(): Promise<Result<null, string>>
  lastfmIsAuthenticated(): Promise<Result<boolean, string>>
  lastfmScrobble(artist: string, track: string, album: null | string, timestamp: number): Promise<Result<null, string>>
  // Last.fm
  lastfmSetCredentials(credentials: LastFmCredentials): Promise<Result<null, string>>
  lastfmUpdateNowPlaying(artist: string, track: string, album: null | string): Promise<Result<null, string>>
  listenbrainzClearCredentials(): Promise<Result<null, string>>
  listenbrainzIsAuthenticated(): Promise<Result<boolean, string>>
  listenbrainzPlayingNow(artist: string, track: string, album: null | string): Promise<Result<null, string>>
  // ListenBrainz
  listenbrainzSetCredentials(credentials: ListenBrainzCredentials): Promise<Result<null, string>>
  listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number): Promise<Result<null, string>>
  listenbrainzValidateToken(userToken: string): Promise<Result<ListenBrainzCredentials, string>>
  loginToJellyfin(
    serverUrl: string,
    username: string,
    password: string,
  ): Promise<Result<{ token: string; userId: string }, string>>

  markItemPlayed(serverUrl: string, token: string, userId: string, itemId: string): Promise<Result<null, string>>
  // Media Controls
  mediaClearNowPlaying(): Promise<Result<null, string>>
  mediaSetButtonEnabled(button: string, enabled: boolean): Promise<Result<null, string>>
  mediaSetPlaybackStatus(isPlaying: boolean, positionSecs: null | number): Promise<Result<null, string>>
  mediaUpdateNowPlaying(payload: NowPlayingPayload): Promise<Result<null, string>>

  quitApplication(): Promise<void>
  // Session
  registerClientCapabilities(serverUrl: string, token: string, deviceId: string): Promise<Result<null, string>>
  removePlaylistItems(playlistId: string, itemIds: string[]): Promise<Result<null, string>>
  reportPlaybackProgress(
    serverUrl: string,
    token: string,
    itemId: string,
    positionTicks: null | number,
    eventName: null | string,
    isPaused: boolean | null,
  ): Promise<Result<null, string>>
  reportPlaybackStart(
    serverUrl: string,
    token: string,
    itemId: string,
    positionTicks: null | number,
  ): Promise<Result<null, string>>

  reportPlaybackStop(
    serverUrl: string,
    token: string,
    itemId: string,
    positionTicks: null | number,
  ): Promise<Result<null, string>>
  saveCredentials(serverUrl: string, username: string, token: string, userId: string): Promise<Result<null, string>>
  saveVolume(volume: number): Promise<Result<null, string>>
  setCloseToTray(closeToTray: boolean): Promise<void>
  setMinimizeToTray(minimizeToTray: boolean): Promise<void>

  // System Tray / Window Management
  showMainWindow(): Promise<void>
  syncLibrary(serverUrl: string, token: string): Promise<Result<null, string>>
  // Favorites
  toggleFavoriteStatus(
    serverUrl: string,
    token: string,
    userId: string,
    itemId: string,
    isFavorite: boolean,
  ): Promise<Result<boolean, string>>
  updatePlaylist(playlistId: string, updates: PlaylistUpdateData): Promise<Result<Playlist, string>>
}

// Helper to convert null results to void
const nullToVoid = <E>(result: Result<null, E>): Result<void, E> => {
  if (result.status === 'ok') {
    return { data: undefined, status: 'ok' }
  }
  return result
}

// Helper to wrap void-returning functions as Result
const wrapVoid = async (fn: () => Promise<void>): Promise<Result<void, string>> => {
  try {
    await fn()
    return { data: undefined, status: 'ok' }
  } catch (e) {
    return { error: e instanceof Error ? e.message : String(e), status: 'error' }
  }
}

// Tauri API Client factory function
export const createTauriClient = (commands: TauriCommands): ApiClient => {
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
    addPlaylistItems: async (playlistId, itemIds) => nullToVoid(await commands.addPlaylistItems(playlistId, itemIds)),

    audioAdvanceGapless: async () => nullToVoid(await commands.audioAdvanceGapless()),

    audioGetAllEqBands: () => commands.audioGetAllEqBands(),

    audioGetEqBand: band => commands.audioGetEqBand(band),

    audioGetEqPreset: commands.audioGetEqPreset
      ? () => commands.audioGetEqPreset!()
      : undefined,
    audioGetPosition:       () => commands.audioGetPosition(),
    audioGetVolume:         () => commands.audioGetVolume(),
    // Audio (Rust player - desktop only)
    audioInit:              async () => nullToVoid(await commands.audioInit()),
    audioIsAnalyzerEnabled: () => commands.audioIsAnalyzerEnabled(),

    audioIsEqEnabled: () => commands.audioIsEqEnabled(),
    audioPause:       async () => nullToVoid(await commands.audioPause()),
    audioPlay:        async streamUrl => {
      const creds = getCredentials()
      return nullToVoid(await commands.audioPlay(streamUrl, creds?.token ?? ''))
    },
    audioPrepareNext: async streamUrl => {
      const creds = getCredentials()
      return nullToVoid(await commands.audioPrepareNext(streamUrl, creds?.token ?? ''))
    },
    audioResume:             async () => nullToVoid(await commands.audioResume()),
    audioSeek:               async position => nullToVoid(await commands.audioSeek(position)),
    audioSetAnalyzerEnabled: async enabled => nullToVoid(await commands.audioSetAnalyzerEnabled(enabled)),

    audioSetEqBand:    async (band, gain) => nullToVoid(await commands.audioSetEqBand(band, gain)),
    audioSetEqEnabled: async enabled => nullToVoid(await commands.audioSetEqEnabled(enabled)),

    audioSetEqPreset: commands.audioSetEqPreset
      ? async preset => nullToVoid(await commands.audioSetEqPreset!(preset))
      : undefined,
    audioSetVolume: async volume => nullToVoid(await commands.audioSetVolume(volume)),
    audioStop:      async () => nullToVoid(await commands.audioStop()),

    authenticate: async (serverUrl, username, password) => {
      const result = await commands.loginToJellyfin(serverUrl, username, password)
      if (result.status === 'ok') {
        cachedCredentials = {
          serverUrl,
          token:  result.data.token,
          userId: result.data.userId,
          username,
        }
        return { data: cachedCredentials, status: 'ok' }
      }
      return result
    },
    clearCache:          async (serverUrl, token) => nullToVoid(await commands.clearCache(serverUrl, token)),
    clearImageFromCache: async (itemId, imageType) => nullToVoid(await commands.clearImageFromCache(itemId, imageType)),
    createPlaylist:      data => commands.createPlaylist(data),

    deletePlaylist: async playlistId => nullToVoid(await commands.deletePlaylist(playlistId)),

    discordRpcClearActivity: async () => nullToVoid(await commands.discordRpcClearActivity()),

    // Discord Rich Presence
    discordRpcIsRunning: () => commands.discordRpcIsRunning(),

    discordRpcSetActivity: async activity => nullToVoid(await commands.discordRpcSetActivity(activity)),

    discordRpcStart: async appId => nullToVoid(await commands.discordRpcStart(appId)),

    discordRpcStop:    async () => nullToVoid(await commands.discordRpcStop()),
    getArtist:         artistId => commands.getArtist(artistId, true),
    // Audio
    getAudioStreamUrl: params =>
      commands.getAudioStreamUrl(
        params.serverUrl,
        params.token,
        params.itemId,
        params.container ?? null,
      ),
    // Home
    getHomeViewData: () => commands.getHomeViewData(),
    // Images
    getImage:        params =>
      commands.getImage(
        params.itemId,
        params.imageType,
        params.serverUrl,
        params.token,
        params.width ?? null,
        params.quality ?? null,
      ),
    getImageCacheStats: () => commands.getImageCacheStats(),

    // Instant Mix
    getInstantMix:     itemId => commands.getInstantMix(itemId),
    // Library
    getLibrary:        () => commands.getLibrary(),
    // Lyrics
    getLyrics:         (id, artist, title, path) => commands.getLyrics(id, artist, title, path ?? null),
    getPlaylistItems:  playlistId => commands.getPlaylistItems(playlistId),
    // Playlists
    getPlaylists:      () => commands.getPlaylists(),
    getRecentlyPlayed: (serverUrl, token, userId) => commands.getRecentlyPlayed(serverUrl, token, userId),

    // Related Artists
    getRelatedArtists:   artistId => commands.getRelatedArtists(artistId),
    // Auth
    getSavedCredentials: async () => {
      const result = await commands.getSavedCredentials()
      if (result.status === 'ok' && result.data) {
        cachedCredentials = result.data
      }
      return result
    },
    getSavedVolume:         () => commands.getSavedVolume(),
    getSong:                songId => commands.getSong(songId),
    // Share URLs
    getSongShareUrls:       songId => commands.getSongShareUrls(songId),
    getSyncState:           () => commands.getSyncState(),
    hideMainWindow:         () => wrapVoid(() => commands.hideMainWindow()),
    lastfmAuthenticate:     () => commands.lastfmAuthenticate(),
    lastfmClearCredentials: async () => nullToVoid(await commands.lastfmClearCredentials()),
    lastfmIsAuthenticated:  () => commands.lastfmIsAuthenticated(),
    lastfmScrobble:         async (artist, track, album, timestamp) =>
      nullToVoid(await commands.lastfmScrobble(artist, track, album ?? null, timestamp ?? Date.now())),
    // Last.fm
    lastfmSetCredentials:   async credentials => nullToVoid(await commands.lastfmSetCredentials(credentials)),
    lastfmUpdateNowPlaying: async (artist, track, album) =>
      nullToVoid(await commands.lastfmUpdateNowPlaying(artist, track, album ?? null)),
    listenbrainzClearCredentials: async () => nullToVoid(await commands.listenbrainzClearCredentials()),
    listenbrainzIsAuthenticated:  () => commands.listenbrainzIsAuthenticated(),
    listenbrainzPlayingNow:       async (artist, track, album) =>
      nullToVoid(await commands.listenbrainzPlayingNow(artist, track, album ?? null)),
    // ListenBrainz
    listenbrainzSetCredentials: async credentials => nullToVoid(await commands.listenbrainzSetCredentials(credentials)),
    listenbrainzSubmitListen:   async (listen, timestamp) =>
      nullToVoid(await commands.listenbrainzSubmitListen(listen, timestamp)),
    listenbrainzValidateToken: userToken => commands.listenbrainzValidateToken(userToken),
    logout:                    async () => {
      const result = await commands.clearSavedCredentials()
      cachedCredentials = null
      return nullToVoid(result)
    },

    markItemPlayed: async itemId => {
      const creds = requireCredentials()
      return nullToVoid(await commands.markItemPlayed(creds.serverUrl, creds.token, creds.userId, itemId))
    },
    // Media Controls
    mediaClearNowPlaying:  async () => nullToVoid(await commands.mediaClearNowPlaying()),
    mediaSetButtonEnabled: async (button, enabled) =>
      nullToVoid(await commands.mediaSetButtonEnabled(button, enabled)),
    mediaSetPlaybackStatus: async (isPlaying, positionSecs) =>
      nullToVoid(await commands.mediaSetPlaybackStatus(isPlaying, positionSecs)),
    mediaUpdateNowPlaying: async payload => nullToVoid(await commands.mediaUpdateNowPlaying(payload)),

    quitApplication:            () => wrapVoid(() => commands.quitApplication()),
    // Session
    registerClientCapabilities: async (serverUrl, token, deviceId) =>
      nullToVoid(await commands.registerClientCapabilities(serverUrl, token, deviceId)),
    removePlaylistItems: async (playlistId, itemIds) =>
      nullToVoid(await commands.removePlaylistItems(playlistId, itemIds)),
    reportPlaybackProgress: async (itemId, position, isPaused) => {
      const creds = requireCredentials()
      const positionTicks = Math.round(position * 10_000_000)
      return nullToVoid(
        await commands.reportPlaybackProgress(
          creds.serverUrl,
          creds.token,
          itemId,
          positionTicks,
          null,
          isPaused,
        ),
      )
    },
    reportPlaybackStart: async (itemId, position) => {
      const creds = requireCredentials()
      const positionTicks = position ? Math.round(position * 10_000_000) : null
      return nullToVoid(await commands.reportPlaybackStart(creds.serverUrl, creds.token, itemId, positionTicks))
    },

    reportPlaybackStop: async (itemId, position) => {
      const creds = requireCredentials()
      const positionTicks = Math.round(position * 10_000_000)
      return nullToVoid(await commands.reportPlaybackStop(creds.serverUrl, creds.token, itemId, positionTicks))
    },
    saveCredentials: async (serverUrl, username, token, userId) => {
      const result = await commands.saveCredentials(serverUrl, username, token, userId)
      if (result.status === 'ok') {
        cachedCredentials = { serverUrl, token, userId, username }
      }
      return nullToVoid(result)
    },
    saveVolume:        async volume => nullToVoid(await commands.saveVolume(volume)),
    setCloseToTray:    closeToTray => wrapVoid(() => commands.setCloseToTray(closeToTray)),
    setMinimizeToTray: minimizeToTray => wrapVoid(() => commands.setMinimizeToTray(minimizeToTray)),

    // System Tray / Window Management
    showMainWindow: () => wrapVoid(() => commands.showMainWindow()),
    syncLibrary:    async () => {
      const creds = requireCredentials()
      return nullToVoid(await commands.syncLibrary(creds.serverUrl, creds.token))
    },
    // Favorites
    toggleFavoriteStatus: async (itemId, isFavorite) => {
      const creds = requireCredentials()
      return commands.toggleFavoriteStatus(creds.serverUrl, creds.token, creds.userId, itemId, isFavorite)
    },
    updatePlaylist: (playlistId, updates) => commands.updatePlaylist(playlistId, updates),
  }
}

export type { ApiClient, Result }
export * from './types'
