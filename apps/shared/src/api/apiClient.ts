// Auto-generated TypeScript client for Aurelia API
// Generated from Api trait - DO NOT EDIT MANUALLY

import { invoke } from '@tauri-apps/api/core';
import type { Credentials, Song, Album, Artist, Playlist, PlaylistCreateData, PlaylistUpdateData, LibraryData, HomeViewData, SyncStateInfo, ListenBrainzCredentials, ListenBrainzListen, AppError, RpcActivity, NowPlayingPayload, LastFmCredentials } from '../generated';

type Result<T, E = string> = 
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E };

const BASE_URL = (import.meta as any).env?.VITE_API_URL || '';
// Check for Tauri v2 internals using 'in' operator
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function tauriCommand<T>(
  command: string,
  payload?: Record<string, unknown>
): Promise<Result<T>> {
  try {
    const data = await invoke(command, payload);
    return { status: 'ok', data: data as T };
  } catch (error) {
    return { status: 'error', error: String(error) };
  }
}

async function webRequest<T>(
  method: string,
  endpoint: string,
  body?: unknown,
  query?: Record<string, string | number | undefined>
): Promise<Result<T>> {
  let url = `${BASE_URL}/api${endpoint}`;
  if (query) {
    const params = new URLSearchParams();
    for (const [key, value] of Object.entries(query)) {
      if (value !== undefined) {
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
  // loginToJellyfin
  loginToJellyfin: async (serverUrl: string, username: string, password: string, deviceId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('login_to_jellyfin', { serverUrl, username, password, deviceId });
    }
    return webRequest('POST', `/auth/login`, { serverUrl: serverUrl, username: username, password: password, deviceId: deviceId }, undefined);
  },

  // saveCredentials
  saveCredentials: async (serverUrl: string, username: string, token: string, userId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('save_credentials', { serverUrl, username, token, userId });
    }
    return webRequest('POST', `/auth/credentials`, { serverUrl: serverUrl, username: username, token: token, userId: userId }, undefined);
  },

  // getSavedCredentials
  getSavedCredentials: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_saved_credentials', undefined);
    }
    return webRequest('GET', `/auth/credentials`, undefined, undefined);
  },

  // clearSavedCredentials
  clearSavedCredentials: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('clear_saved_credentials', undefined);
    }
    return webRequest('POST', `/auth/credentials/clear`, undefined, undefined);
  },

  // saveVolume
  saveVolume: async (volume: number): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('save_volume', { volume });
    }
    return webRequest('POST', `/auth/volume`, { volume: volume }, undefined);
  },

  // getSavedVolume
  getSavedVolume: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_saved_volume', undefined);
    }
    return webRequest('GET', `/auth/volume`, undefined, undefined);
  },

  // getLibrary
  getLibrary: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_library', undefined);
    }
    return webRequest('GET', `/library`, undefined, undefined);
  },

  // syncLibrary
  syncLibrary: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('sync_library', undefined);
    }
    return webRequest('POST', `/library/sync`, undefined, undefined);
  },

  // getSyncState
  getSyncState: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_sync_state', undefined);
    }
    return webRequest('GET', `/library/sync-state`, undefined, undefined);
  },

  // getSong
  getSong: async (songId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_song', { songId });
    }
    return webRequest('GET', `/songs/${songId}`, undefined, undefined);
  },

  // toggleFavoriteStatus
  toggleFavoriteStatus: async (itemId: string, isFavorite: boolean): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('toggle_favorite_status', { itemId, isFavorite });
    }
    return webRequest('POST', `/songs/${itemId}/favorite`, { isFavorite: isFavorite }, undefined);
  },

  // getInstantMix
  getInstantMix: async (itemId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_instant_mix', { itemId });
    }
    return webRequest('GET', `/songs/${itemId}/instant-mix`, undefined, undefined);
  },

  // getSongShareUrls
  getSongShareUrls: async (itemId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_song_share_urls', { itemId });
    }
    return webRequest('GET', `/songs/${itemId}/share-urls`, undefined, undefined);
  },

  // getArtist
  getArtist: async (artistId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_artist', { artistId });
    }
    return webRequest('GET', `/artists/${artistId}`, undefined, undefined);
  },

  // getRelatedArtists
  getRelatedArtists: async (artistId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_related_artists', { artistId });
    }
    return webRequest('GET', `/artists/${artistId}/related`, undefined, undefined);
  },

  // getArtistShareUrls
  getArtistShareUrls: async (artistId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_artist_share_urls', { artistId });
    }
    return webRequest('GET', `/artists/${artistId}/share-urls`, undefined, undefined);
  },

  // getAlbum
  getAlbum: async (albumId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_album', { albumId });
    }
    return webRequest('GET', `/albums/${albumId}`, undefined, undefined);
  },

  // getAlbumShareUrls
  getAlbumShareUrls: async (albumId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_album_share_urls', { albumId });
    }
    return webRequest('GET', `/albums/${albumId}/share-urls`, undefined, undefined);
  },

  // getPlaylists
  getPlaylists: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_playlists', undefined);
    }
    return webRequest('GET', `/playlists`, undefined, undefined);
  },

  // getPlaylistItems
  getPlaylistItems: async (playlistId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_playlist_items', { playlistId });
    }
    return webRequest('GET', `/playlists/${playlistId}/items`, undefined, undefined);
  },

  // createPlaylist
  createPlaylist: async (data: PlaylistCreateData): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('create_playlist', { data });
    }
    return webRequest('POST', `/playlists`, data, undefined);
  },

  // updatePlaylist
  updatePlaylist: async (playlistId: string, updates: PlaylistUpdateData): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('update_playlist', { playlistId, updates });
    }
    return webRequest('PATCH', `/playlists/${playlistId}`, updates, undefined);
  },

  // deletePlaylist
  deletePlaylist: async (playlistId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('delete_playlist', { playlistId });
    }
    return webRequest('DELETE', `/playlists/${playlistId}`, undefined, undefined);
  },

  // addPlaylistItems
  addPlaylistItems: async (playlistId: string, songIds: string[]): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('add_playlist_items', { playlistId, songIds });
    }
    return webRequest('POST', `/playlists/${playlistId}/items`, { songIds: songIds }, undefined);
  },

  // removePlaylistItems
  removePlaylistItems: async (playlistId: string, songIds: string[]): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('remove_playlist_items', { playlistId, songIds });
    }
    return webRequest('DELETE', `/playlists/${playlistId}/items`, undefined, { songIds: songIds });
  },

  // getHomeViewData
  getHomeViewData: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_home_view_data', undefined);
    }
    return webRequest('GET', `/home`, undefined, undefined);
  },

  // getRecentlyPlayed
  getRecentlyPlayed: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_recently_played', undefined);
    }
    return webRequest('GET', `/home/recently-played`, undefined, undefined);
  },

  // getImage
  getImage: async (itemId: string, imageType: string, serverUrl: string, token: string, width?: number, quality?: number): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_image', { itemId, imageType, serverUrl, token, width, quality });
    }
    return webRequest('GET', `/images/${itemId}`, undefined, { imageType: imageType, serverUrl: serverUrl, token: token, width: width, quality: quality });
  },

  // clearImageCache
  clearImageCache: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('clear_image_cache', undefined);
    }
    return webRequest('POST', `/cache/image-clear`, undefined, undefined);
  },

  // getImageCacheStats
  getImageCacheStats: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_image_cache_stats', undefined);
    }
    return webRequest('GET', `/cache/image-stats`, undefined, undefined);
  },

  // clearImageFromCache
  clearImageFromCache: async (itemId: string, imageType: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('clear_image_from_cache', { itemId, imageType });
    }
    return webRequest('POST', `/cache/image-clear/${itemId}`, { imageType: imageType }, undefined);
  },

  // getAudioStreamUrl
  getAudioStreamUrl: async (itemId: string, serverUrl: string, token: string, container?: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_audio_stream_url', { itemId, serverUrl, token, container });
    }
    return webRequest('GET', `/audio/${itemId}/stream-url`, undefined, { serverUrl: serverUrl, token: token, container: container });
  },

  // getLyrics
  getLyrics: async (id: string, artist: string, title: string, path?: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('get_lyrics', { id, artist, title, path });
    }
    return webRequest('POST', `/lyrics`, { id: id, artist: artist, title: title, path: path }, undefined);
  },

  // clearCache
  clearCache: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('clear_cache', undefined);
    }
    return webRequest('POST', `/cache/clear`, undefined, undefined);
  },

  // registerClientCapabilities
  registerClientCapabilities: async (serverUrl: string, token: string, deviceId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('register_client_capabilities', { serverUrl, token, deviceId });
    }
    return webRequest('POST', `/sessions/capabilities`, { serverUrl: serverUrl, token: token, deviceId: deviceId }, undefined);
  },

  // reportPlaybackStart
  reportPlaybackStart: async (itemId: string, positionTicks?: number): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('report_playback_start', { itemId, positionTicks });
    }
    return webRequest('POST', `/sessions/playing/start`, { itemId: itemId, positionTicks: positionTicks }, undefined);
  },

  // reportPlaybackProgress
  reportPlaybackProgress: async (itemId: string, positionTicks: number, isPaused: boolean): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('report_playback_progress', { itemId, positionTicks, isPaused });
    }
    return webRequest('POST', `/sessions/playing/progress`, { itemId: itemId, positionTicks: positionTicks, isPaused: isPaused }, undefined);
  },

  // reportPlaybackStop
  reportPlaybackStop: async (itemId: string, positionTicks: number): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('report_playback_stop', { itemId, positionTicks });
    }
    return webRequest('POST', `/sessions/playing/stop`, { itemId: itemId, positionTicks: positionTicks }, undefined);
  },

  // markItemPlayed
  markItemPlayed: async (itemId: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('mark_item_played', { itemId });
    }
    return webRequest('POST', `/sessions/mark-played`, { itemId: itemId }, undefined);
  },

  // listenbrainzSetCredentials
  listenbrainzSetCredentials: async (credentials: ListenBrainzCredentials): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('listenbrainz_set_credentials', { credentials });
    }
    return webRequest('POST', `/listenbrainz/credentials`, { credentials: credentials }, undefined);
  },

  // listenbrainzClearCredentials
  listenbrainzClearCredentials: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('listenbrainz_clear_credentials', undefined);
    }
    return webRequest('DELETE', `/listenbrainz/credentials`, undefined, undefined);
  },

  // listenbrainzIsAuthenticated
  listenbrainzIsAuthenticated: async (): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('listenbrainz_is_authenticated', undefined);
    }
    return webRequest('GET', `/listenbrainz/auth-status`, undefined, undefined);
  },

  // listenbrainzValidateToken
  listenbrainzValidateToken: async (userToken: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('listenbrainz_validate_token', { userToken });
    }
    return webRequest('POST', `/listenbrainz/validate`, { userToken: userToken }, undefined);
  },

  // listenbrainzSubmitListen
  listenbrainzSubmitListen: async (listen: ListenBrainzListen, timestamp: number): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('listenbrainz_submit_listen', { listen, timestamp });
    }
    return webRequest('POST', `/listenbrainz/submit-listen`, { listen: listen, timestamp: timestamp }, undefined);
  },

  // listenbrainzPlayingNow
  listenbrainzPlayingNow: async (artist: string, track: string, album?: string): Promise<Result<any>> => {
    if (isTauri) {
      return tauriCommand('listenbrainz_playing_now', { artist, track, album });
    }
    return webRequest('POST', `/listenbrainz/playing-now`, { artist: artist, track: track, album: album }, undefined);
  },

  // Desktop-only: audioInit
  audioInit: async (): Promise<Result<any>> => {
    return tauriCommand('audio_init', undefined);
  },

  // Desktop-only: audioPlay
  audioPlay: async (url: string, token: string): Promise<Result<any>> => {
    return tauriCommand('audio_play', { url, token });
  },

  // Desktop-only: audioPause
  audioPause: async (): Promise<Result<any>> => {
    return tauriCommand('audio_pause', undefined);
  },

  // Desktop-only: audioResume
  audioResume: async (): Promise<Result<any>> => {
    return tauriCommand('audio_resume', undefined);
  },

  // Desktop-only: audioStop
  audioStop: async (): Promise<Result<any>> => {
    return tauriCommand('audio_stop', undefined);
  },

  // Desktop-only: audioGetVolume
  audioGetVolume: async (): Promise<Result<any>> => {
    return tauriCommand('audio_get_volume', undefined);
  },

  // Desktop-only: audioSetVolume
  audioSetVolume: async (volume: number): Promise<Result<any>> => {
    return tauriCommand('audio_set_volume', { volume });
  },

  // Desktop-only: audioSeek
  audioSeek: async (positionSecs: number): Promise<Result<any>> => {
    return tauriCommand('audio_seek', { positionSecs });
  },

  // Desktop-only: audioGetPosition
  audioGetPosition: async (): Promise<Result<any>> => {
    return tauriCommand('audio_get_position', undefined);
  },

  // Desktop-only: audioIsPlaying
  audioIsPlaying: async (): Promise<Result<any>> => {
    return tauriCommand('audio_is_playing', undefined);
  },

  // Desktop-only: discordRpcStart
  discordRpcStart: async (appId: string): Promise<Result<any>> => {
    return tauriCommand('discord_rpc_start', { appId });
  },

  // Desktop-only: discordRpcStop
  discordRpcStop: async (): Promise<Result<any>> => {
    return tauriCommand('discord_rpc_stop', undefined);
  },

  // Desktop-only: discordRpcIsRunning
  discordRpcIsRunning: async (): Promise<Result<any>> => {
    return tauriCommand('discord_rpc_is_running', undefined);
  },

  // Desktop-only: discordRpcSetActivity
  discordRpcSetActivity: async (activity: RpcActivity): Promise<Result<any>> => {
    return tauriCommand('discord_rpc_set_activity', { activity });
  },

  // Desktop-only: discordRpcClearActivity
  discordRpcClearActivity: async (): Promise<Result<any>> => {
    return tauriCommand('discord_rpc_clear_activity', undefined);
  },

  // Desktop-only: audioIsEqEnabled
  audioIsEqEnabled: async (): Promise<Result<any>> => {
    return tauriCommand('audio_is_eq_enabled', undefined);
  },

  // Desktop-only: audioSetEqEnabled
  audioSetEqEnabled: async (enabled: boolean): Promise<Result<any>> => {
    return tauriCommand('audio_set_eq_enabled', { enabled });
  },

  // Desktop-only: audioGetEqBand
  audioGetEqBand: async (band: number): Promise<Result<any>> => {
    return tauriCommand('audio_get_eq_band', { band });
  },

  // Desktop-only: audioSetEqBand
  audioSetEqBand: async (band: number, gainDb: number): Promise<Result<any>> => {
    return tauriCommand('audio_set_eq_band', { band, gainDb });
  },

  // Desktop-only: audioGetAllEqBands
  audioGetAllEqBands: async (): Promise<Result<any>> => {
    return tauriCommand('audio_get_all_eq_bands', undefined);
  },

  // Desktop-only: audioResetEq
  audioResetEq: async (): Promise<Result<any>> => {
    return tauriCommand('audio_reset_eq', undefined);
  },

  // Desktop-only: audioAdvanceGapless
  audioAdvanceGapless: async (): Promise<Result<any>> => {
    return tauriCommand('audio_advance_gapless', undefined);
  },

  // Desktop-only: audioPrepareNext
  audioPrepareNext: async (url: string, token: string): Promise<Result<any>> => {
    return tauriCommand('audio_prepare_next', { url, token });
  },

  // Desktop-only: audioIsFinished
  audioIsFinished: async (): Promise<Result<any>> => {
    return tauriCommand('audio_is_finished', undefined);
  },

  // Desktop-only: audioSetAnalyzerEnabled
  audioSetAnalyzerEnabled: async (enabled: boolean): Promise<Result<any>> => {
    return tauriCommand('audio_set_analyzer_enabled', { enabled });
  },

  // Desktop-only: audioIsAnalyzerEnabled
  audioIsAnalyzerEnabled: async (): Promise<Result<any>> => {
    return tauriCommand('audio_is_analyzer_enabled', undefined);
  },

  // Desktop-only: audioReinit
  audioReinit: async (): Promise<Result<any>> => {
    return tauriCommand('audio_reinit', undefined);
  },

  // Desktop-only: mediaUpdateNowPlaying
  mediaUpdateNowPlaying: async (payload: NowPlayingPayload): Promise<Result<any>> => {
    return tauriCommand('media_update_now_playing', { payload });
  },

  // Desktop-only: mediaClearNowPlaying
  mediaClearNowPlaying: async (): Promise<Result<any>> => {
    return tauriCommand('media_clear_now_playing', undefined);
  },

  // Desktop-only: mediaSetPlaybackStatus
  mediaSetPlaybackStatus: async (isPlaying: boolean, positionSecs?: number): Promise<Result<any>> => {
    return tauriCommand('media_set_playback_status', { isPlaying, positionSecs });
  },

  // Desktop-only: mediaSetButtonEnabled
  mediaSetButtonEnabled: async (button: string, enabled: boolean): Promise<Result<any>> => {
    return tauriCommand('media_set_button_enabled', { button, enabled });
  },

  // Desktop-only: lastfmSetCredentials
  lastfmSetCredentials: async (credentials: LastFmCredentials): Promise<Result<any>> => {
    return tauriCommand('lastfm_set_credentials', { credentials });
  },

  // Desktop-only: lastfmClearCredentials
  lastfmClearCredentials: async (): Promise<Result<any>> => {
    return tauriCommand('lastfm_clear_credentials', undefined);
  },

  // Desktop-only: lastfmIsAuthenticated
  lastfmIsAuthenticated: async (): Promise<Result<any>> => {
    return tauriCommand('lastfm_is_authenticated', undefined);
  },

  // Desktop-only: lastfmStartAuthServer
  lastfmStartAuthServer: async (): Promise<Result<any>> => {
    return tauriCommand('lastfm_start_auth_server', undefined);
  },

  // Desktop-only: lastfmAuthenticate
  lastfmAuthenticate: async (): Promise<Result<any>> => {
    return tauriCommand('lastfm_authenticate', undefined);
  },

  // Desktop-only: lastfmScrobble
  lastfmScrobble: async (artist: string, track: string, album?: string, timestamp?: number): Promise<Result<any>> => {
    return tauriCommand('lastfm_scrobble', { artist, track, album, timestamp });
  },

  // Desktop-only: lastfmUpdateNowPlaying
  lastfmUpdateNowPlaying: async (artist: string, track: string, album?: string): Promise<Result<any>> => {
    return tauriCommand('lastfm_update_now_playing', { artist, track, album });
  },

  // Desktop-only: showMainWindow
  showMainWindow: async (): Promise<Result<any>> => {
    return tauriCommand('show_main_window', undefined);
  },

  // Desktop-only: hideMainWindow
  hideMainWindow: async (): Promise<Result<any>> => {
    return tauriCommand('hide_main_window', undefined);
  },

  // Desktop-only: quitApplication
  quitApplication: async (): Promise<Result<any>> => {
    return tauriCommand('quit_application', undefined);
  },

  // Desktop-only: setMinimizeToTray
  setMinimizeToTray: async (minimizeToTray: boolean): Promise<Result<any>> => {
    return tauriCommand('set_minimize_to_tray', { minimizeToTray });
  },

  // Desktop-only: setCloseToTray
  setCloseToTray: async (closeToTray: boolean): Promise<Result<any>> => {
    return tauriCommand('set_close_to_tray', { closeToTray });
  },

};
