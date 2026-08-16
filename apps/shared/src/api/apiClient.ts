// Auto-generated TypeScript client for Aurelia API
// Generated from Api trait - DO NOT EDIT MANUALLY

import type { Credentials, AuthRequest, BackendProvider, ProviderCapabilities, Song, Album, Artist, Playlist, PlaylistCreateData, PlaylistUpdateData, LibraryData, HomeViewData, SyncStateInfo, ListenBrainzCredentials, ListenBrainzListen, AppError, RpcActivity, LastFmCredentials } from '../generated';

type Result<T, E = string> = 
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E };
type QueryValue = string | number | boolean | ReadonlyArray<string | number | boolean> | undefined;

const BASE_URL = (import.meta as any).env?.VITE_API_URL || '';

async function webRequest<T>(
  method: string,
  endpoint: string,
  body?: unknown,
  query?: Record<string, QueryValue>
): Promise<Result<T>> {
  let url = `${BASE_URL}/api${endpoint}`;
  if (query) {
    const params = new URLSearchParams();
    for (const [key, value] of Object.entries(query)) {
      if (value === undefined) {
        continue;
      }
      if (Array.isArray(value)) {
        for (const item of value) {
          params.append(key, String(item));
        }
      } else {
        params.append(key, String(value));
      }
    }
    const queryString = params.toString();
    if (queryString) {
      url += `?${queryString}`;
    }
  }
  
  const options: RequestInit = {
    method,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
  };

  if (body !== undefined) {
    options.body = JSON.stringify(body);
  }

  const response = await fetch(url, options);

  if (!response.ok) {
    const errorText = await response.text();
    return {
      status: 'error',
      error: `HTTP ${response.status}: ${errorText || response.statusText}`,
    };
  }

  return await response.json();
}

export const apiClient = {
  // detectProvider
  detectProvider: async (serverUrl: string): Promise<Result<any>> => {
    return webRequest('POST', `/auth/detect-provider`, { serverUrl: serverUrl }, undefined);
  },

  // getProviderCapabilities
  getProviderCapabilities: async (provider: BackendProvider, serverUrl: string): Promise<Result<any>> => {
    return webRequest('POST', `/auth/provider-capabilities`, { provider: provider, serverUrl: serverUrl }, undefined);
  },

  // authenticate
  authenticate: async (request: AuthRequest): Promise<Result<any>> => {
    return webRequest('POST', `/auth/authenticate`, request, undefined);
  },

  // saveCredentials
  saveCredentials: async (credentials: Credentials): Promise<Result<any>> => {
    return webRequest('POST', `/auth/credentials`, { credentials: credentials }, undefined);
  },

  // getSavedCredentials
  getSavedCredentials: async (): Promise<Result<any>> => {
    return webRequest('GET', `/auth/credentials`, undefined, undefined);
  },

  // clearSavedCredentials
  clearSavedCredentials: async (): Promise<Result<any>> => {
    return webRequest('POST', `/auth/credentials/clear`, undefined, undefined);
  },

  // saveVolume
  saveVolume: async (volume: number): Promise<Result<any>> => {
    return webRequest('POST', `/auth/volume`, { volume: volume }, undefined);
  },

  // getSavedVolume
  getSavedVolume: async (): Promise<Result<any>> => {
    return webRequest('GET', `/auth/volume`, undefined, undefined);
  },

  // getLibrary
  getLibrary: async (): Promise<Result<any>> => {
    return webRequest('GET', `/library`, undefined, undefined);
  },

  // syncLibrary
  syncLibrary: async (): Promise<Result<any>> => {
    return webRequest('POST', `/library/sync`, undefined, undefined);
  },

  // getSyncState
  getSyncState: async (): Promise<Result<any>> => {
    return webRequest('GET', `/library/sync-state`, undefined, undefined);
  },

  // getSong
  getSong: async (songId: string): Promise<Result<any>> => {
    return webRequest('GET', `/songs/${songId}`, undefined, undefined);
  },

  // toggleFavoriteStatus
  toggleFavoriteStatus: async (itemId: string, isFavorite: boolean): Promise<Result<any>> => {
    return webRequest('POST', `/songs/${itemId}/favorite`, { isFavorite: isFavorite }, undefined);
  },

  // getInstantMix
  getInstantMix: async (itemId: string): Promise<Result<any>> => {
    return webRequest('GET', `/songs/${itemId}/instant-mix`, undefined, undefined);
  },

  // getSongShareUrls
  getSongShareUrls: async (itemId: string): Promise<Result<any>> => {
    return webRequest('GET', `/songs/${itemId}/share-urls`, undefined, undefined);
  },

  // getArtist
  getArtist: async (artistId: string): Promise<Result<any>> => {
    return webRequest('GET', `/artists/${artistId}`, undefined, undefined);
  },

  // getRelatedArtists
  getRelatedArtists: async (artistId: string): Promise<Result<any>> => {
    return webRequest('GET', `/artists/${artistId}/related`, undefined, undefined);
  },

  // getArtistShareUrls
  getArtistShareUrls: async (artistId: string): Promise<Result<any>> => {
    return webRequest('GET', `/artists/${artistId}/share-urls`, undefined, undefined);
  },

  // getAlbum
  getAlbum: async (albumId: string): Promise<Result<any>> => {
    return webRequest('GET', `/albums/${albumId}`, undefined, undefined);
  },

  // getAlbumShareUrls
  getAlbumShareUrls: async (albumId: string): Promise<Result<any>> => {
    return webRequest('GET', `/albums/${albumId}/share-urls`, undefined, undefined);
  },

  // getPlaylists
  getPlaylists: async (): Promise<Result<any>> => {
    return webRequest('GET', `/playlists`, undefined, undefined);
  },

  // getPlaylistItems
  getPlaylistItems: async (playlistId: string): Promise<Result<any>> => {
    return webRequest('GET', `/playlists/${playlistId}/items`, undefined, undefined);
  },

  // createPlaylist
  createPlaylist: async (data: PlaylistCreateData): Promise<Result<any>> => {
    return webRequest('POST', `/playlists`, data, undefined);
  },

  // updatePlaylist
  updatePlaylist: async (playlistId: string, updates: PlaylistUpdateData): Promise<Result<any>> => {
    return webRequest('PATCH', `/playlists/${playlistId}`, updates, undefined);
  },

  // deletePlaylist
  deletePlaylist: async (playlistId: string): Promise<Result<any>> => {
    return webRequest('DELETE', `/playlists/${playlistId}`, undefined, undefined);
  },

  // addPlaylistItems
  addPlaylistItems: async (playlistId: string, songIds: string[]): Promise<Result<any>> => {
    return webRequest('POST', `/playlists/${playlistId}/items`, { songIds: songIds }, undefined);
  },

  // removePlaylistItems
  removePlaylistItems: async (playlistId: string, songIds: string[]): Promise<Result<any>> => {
    return webRequest('DELETE', `/playlists/${playlistId}/items`, undefined, { songIds: songIds });
  },

  // getHomeViewData
  getHomeViewData: async (): Promise<Result<any>> => {
    return webRequest('GET', `/home`, undefined, undefined);
  },

  // getRecentlyPlayed
  getRecentlyPlayed: async (): Promise<Result<any>> => {
    return webRequest('GET', `/home/recently-played`, undefined, undefined);
  },

  // getImage
  getImage: async (itemId: string, imageType: string, serverUrl: string, token: string, width?: number, quality?: number): Promise<Result<any>> => {
    return webRequest('GET', `/images/${itemId}`, undefined, { imageType: imageType, serverUrl: serverUrl, token: token, width: width, quality: quality });
  },

  // clearImageCache
  clearImageCache: async (): Promise<Result<any>> => {
    return webRequest('POST', `/cache/image-clear`, undefined, undefined);
  },

  // getImageCacheStats
  getImageCacheStats: async (): Promise<Result<any>> => {
    return webRequest('GET', `/cache/image-stats`, undefined, undefined);
  },

  // clearImageFromCache
  clearImageFromCache: async (itemId: string, imageType: string): Promise<Result<any>> => {
    return webRequest('POST', `/cache/image-clear/${itemId}`, { imageType: imageType }, undefined);
  },

  // getAudioStreamUrl
  getAudioStreamUrl: async (itemId: string, serverUrl: string, token: string, container?: string): Promise<Result<any>> => {
    return webRequest('GET', `/audio/${itemId}/stream-url`, undefined, { serverUrl: serverUrl, token: token, container: container });
  },

  // getLyrics
  getLyrics: async (id: string, artist: string, title: string, path?: string): Promise<Result<any>> => {
    return webRequest('POST', `/lyrics`, { id: id, artist: artist, title: title, path: path }, undefined);
  },

  // getParsedLyrics
  getParsedLyrics: async (id: string, artist: string, title: string, path?: string): Promise<Result<any>> => {
    return webRequest('POST', `/lyrics/parsed`, { id: id, artist: artist, title: title, path: path }, undefined);
  },

  // getSidecarLyrics
  getSidecarLyrics: async (itemId: string): Promise<Result<any>> => {
    return webRequest('GET', `/lyrics/sidecar/${itemId}`, undefined, undefined);
  },

  // getSetting
  getSetting: async (key: string): Promise<Result<any>> => {
    return webRequest('GET', `/settings/${key}`, undefined, undefined);
  },

  // saveSetting
  saveSetting: async (key: string, value: string): Promise<Result<any>> => {
    return webRequest('POST', `/settings/${key}`, { value: value }, undefined);
  },

  // deleteSetting
  deleteSetting: async (key: string): Promise<Result<any>> => {
    return webRequest('DELETE', `/settings/${key}`, undefined, undefined);
  },

  // clearCache
  clearCache: async (): Promise<Result<any>> => {
    return webRequest('POST', `/cache/clear`, undefined, undefined);
  },

  // registerClientCapabilities
  registerClientCapabilities: async (serverUrl: string, token: string, deviceId: string): Promise<Result<any>> => {
    return webRequest('POST', `/sessions/capabilities`, { serverUrl: serverUrl, token: token, deviceId: deviceId }, undefined);
  },

  // reportPlaybackStart
  reportPlaybackStart: async (itemId: string, positionTicks?: number): Promise<Result<any>> => {
    return webRequest('POST', `/sessions/playing/start`, { itemId: itemId, positionTicks: positionTicks }, undefined);
  },

  // reportPlaybackProgress
  reportPlaybackProgress: async (itemId: string, positionTicks: number, isPaused: boolean): Promise<Result<any>> => {
    return webRequest('POST', `/sessions/playing/progress`, { itemId: itemId, positionTicks: positionTicks, isPaused: isPaused }, undefined);
  },

  // reportPlaybackStop
  reportPlaybackStop: async (itemId: string, positionTicks: number): Promise<Result<any>> => {
    return webRequest('POST', `/sessions/playing/stop`, { itemId: itemId, positionTicks: positionTicks }, undefined);
  },

  // markItemPlayed
  markItemPlayed: async (itemId: string): Promise<Result<any>> => {
    return webRequest('POST', `/sessions/mark-played`, { itemId: itemId }, undefined);
  },

  // listenbrainzSetCredentials
  listenbrainzSetCredentials: async (credentials: ListenBrainzCredentials): Promise<Result<any>> => {
    return webRequest('POST', `/listenbrainz/credentials`, { credentials: credentials }, undefined);
  },

  // listenbrainzClearCredentials
  listenbrainzClearCredentials: async (): Promise<Result<any>> => {
    return webRequest('DELETE', `/listenbrainz/credentials`, undefined, undefined);
  },

  // listenbrainzIsAuthenticated
  listenbrainzIsAuthenticated: async (): Promise<Result<any>> => {
    return webRequest('GET', `/listenbrainz/auth-status`, undefined, undefined);
  },

  // listenbrainzValidateToken
  listenbrainzValidateToken: async (userToken: string): Promise<Result<any>> => {
    return webRequest('POST', `/listenbrainz/validate`, { userToken: userToken }, undefined);
  },

  // listenbrainzSubmitListen
  listenbrainzSubmitListen: async (listen: ListenBrainzListen, timestamp: number): Promise<Result<any>> => {
    return webRequest('POST', `/listenbrainz/submit-listen`, { listen: listen, timestamp: timestamp }, undefined);
  },

  // listenbrainzPlayingNow
  listenbrainzPlayingNow: async (artist: string, track: string, album?: string): Promise<Result<any>> => {
    return webRequest('POST', `/listenbrainz/playing-now`, { artist: artist, track: track, album: album }, undefined);
  },

  // discordRpcStart
  discordRpcStart: async (appId: string): Promise<Result<any>> => {
    return webRequest('POST', `/discord/start`, { appId: appId }, undefined);
  },

  // discordRpcStop
  discordRpcStop: async (): Promise<Result<any>> => {
    return webRequest('POST', `/discord/stop`, undefined, undefined);
  },

  // discordRpcIsRunning
  discordRpcIsRunning: async (): Promise<Result<any>> => {
    return webRequest('GET', `/discord/is-running`, undefined, undefined);
  },

  // discordRpcSetActivity
  discordRpcSetActivity: async (activity: RpcActivity): Promise<Result<any>> => {
    return webRequest('POST', `/discord/activity`, { activity: activity }, undefined);
  },

  // discordRpcClearActivity
  discordRpcClearActivity: async (): Promise<Result<any>> => {
    return webRequest('POST', `/discord/clear-activity`, undefined, undefined);
  },

  // lastfmSetCredentials
  lastfmSetCredentials: async (credentials: LastFmCredentials): Promise<Result<any>> => {
    return webRequest('POST', `/lastfm/credentials`, { credentials: credentials }, undefined);
  },

  // lastfmClearCredentials
  lastfmClearCredentials: async (): Promise<Result<any>> => {
    return webRequest('DELETE', `/lastfm/credentials`, undefined, undefined);
  },

  // lastfmIsAuthenticated
  lastfmIsAuthenticated: async (): Promise<Result<any>> => {
    return webRequest('GET', `/lastfm/auth-status`, undefined, undefined);
  },

  // lastfmAuthenticate
  lastfmAuthenticate: async (apiKey: string, apiSecret: string, token: string): Promise<Result<any>> => {
    return webRequest('POST', `/lastfm/authenticate`, { apiKey: apiKey, apiSecret: apiSecret, token: token }, undefined);
  },

  // lastfmScrobble
  lastfmScrobble: async (artist: string, track: string, album?: string, timestamp?: number): Promise<Result<any>> => {
    return webRequest('POST', `/lastfm/scrobble`, { artist: artist, track: track, album: album, timestamp: timestamp }, undefined);
  },

  // lastfmUpdateNowPlaying
  lastfmUpdateNowPlaying: async (artist: string, track: string, album?: string): Promise<Result<any>> => {
    return webRequest('POST', `/lastfm/playing-now`, { artist: artist, track: track, album: album }, undefined);
  },

  // audioInit
  audioInit: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/init`, undefined, undefined);
  },

  // audioPlay
  audioPlay: async (url: string, token: string): Promise<Result<any>> => {
    return webRequest('POST', `/audio/play`, { url: url, token: token }, undefined);
  },

  // audioPause
  audioPause: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/pause`, undefined, undefined);
  },

  // audioResume
  audioResume: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/resume`, undefined, undefined);
  },

  // audioStop
  audioStop: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/stop`, undefined, undefined);
  },

  // audioGetVolume
  audioGetVolume: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/volume`, undefined, undefined);
  },

  // audioSetVolume
  audioSetVolume: async (volume: number): Promise<Result<any>> => {
    return webRequest('POST', `/audio/volume`, { volume: volume }, undefined);
  },

  // audioSeek
  audioSeek: async (positionSecs: number): Promise<Result<any>> => {
    return webRequest('POST', `/audio/seek`, { positionSecs: positionSecs }, undefined);
  },

  // audioGetPosition
  audioGetPosition: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/position`, undefined, undefined);
  },

  // audioIsPlaying
  audioIsPlaying: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/is-playing`, undefined, undefined);
  },

  // audioIsEqEnabled
  audioIsEqEnabled: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/eq/enabled`, undefined, undefined);
  },

  // audioSetEqEnabled
  audioSetEqEnabled: async (enabled: boolean): Promise<Result<any>> => {
    return webRequest('POST', `/audio/eq/enabled`, { enabled: enabled }, undefined);
  },

  // audioGetEqBand
  audioGetEqBand: async (band: number): Promise<Result<any>> => {
    return webRequest('GET', `/audio/eq/band`, undefined, { band: band });
  },

  // audioSetEqBand
  audioSetEqBand: async (band: number, gainDb: number): Promise<Result<any>> => {
    return webRequest('POST', `/audio/eq/band`, { band: band, gainDb: gainDb }, undefined);
  },

  // audioGetAllEqBands
  audioGetAllEqBands: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/eq/all-bands`, undefined, undefined);
  },

  // audioResetEq
  audioResetEq: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/eq/reset`, undefined, undefined);
  },

  // audioAdvanceGapless
  audioAdvanceGapless: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/advance-gapless`, undefined, undefined);
  },

  // audioPrepareNext
  audioPrepareNext: async (url: string, token: string): Promise<Result<any>> => {
    return webRequest('POST', `/audio/prepare-next`, { url: url, token: token }, undefined);
  },

  // audioIsFinished
  audioIsFinished: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/is-finished`, undefined, undefined);
  },

  // audioSetAnalyzerEnabled
  audioSetAnalyzerEnabled: async (enabled: boolean): Promise<Result<any>> => {
    return webRequest('POST', `/audio/analyzer`, { enabled: enabled }, undefined);
  },

  // audioIsAnalyzerEnabled
  audioIsAnalyzerEnabled: async (): Promise<Result<any>> => {
    return webRequest('GET', `/audio/analyzer`, undefined, undefined);
  },

  // audioReinit
  audioReinit: async (): Promise<Result<any>> => {
    return webRequest('POST', `/audio/reinit`, undefined, undefined);
  },

};
