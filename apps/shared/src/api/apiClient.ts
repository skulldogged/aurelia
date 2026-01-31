// Auto-generated TypeScript client for Aurelia API
// Generated from Api trait - DO NOT EDIT MANUALLY

import { invoke } from '@tauri-apps/api/core';

type Result<T, E = string> = 
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E };

const BASE_URL = (import.meta as any).env?.VITE_API_URL || '';
const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;

async function apiRequest<T>(
  method: string,
  endpoint: string,
  body?: unknown,
  query?: Record<string, string | number | undefined>
): Promise<Result<T>> {
  if (isTauri) {
    // Desktop: use Tauri IPC
    const command = `cmd_${method}`;
    try {
      const data = await invoke(command, body);
      return { status: 'ok', data: data as T };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  } else {
    // Web: use HTTP
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
}

export const apiClient = {
  // getSavedCredentials
  getSavedCredentials: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/auth/credentials`, undefined, undefined);
  },

  // authenticate
  authenticate: async (serverUrl: string, username: string, password: string): Promise<Result<any>> => {
    return apiRequest('POST', `/auth/login`, { serverUrl: serverUrl, username: username, password: password }, undefined);
  },

  // logout
  logout: async (): Promise<Result<any>> => {
    return apiRequest('POST', `/auth/logout`, undefined, undefined);
  },

  // getLibrary
  getLibrary: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/library`, undefined, undefined);
  },

  // syncLibrary
  syncLibrary: async (): Promise<Result<any>> => {
    return apiRequest('POST', `/library/sync`, undefined, undefined);
  },

  // getSyncState
  getSyncState: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/library/sync-state`, undefined, undefined);
  },

  // getSong
  getSong: async (songId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/songs/${songId}`, undefined, undefined);
  },

  // toggleFavoriteStatus
  toggleFavoriteStatus: async (itemId: string, isFavorite: boolean): Promise<Result<any>> => {
    return apiRequest('POST', `/songs/${itemId}/favorite`, { isFavorite: isFavorite }, undefined);
  },

  // getInstantMix
  getInstantMix: async (itemId: string, limit?: number): Promise<Result<any>> => {
    return apiRequest('GET', `/songs/${itemId}/instant-mix`, undefined, { limit: limit });
  },

  // getSongShareUrls
  getSongShareUrls: async (itemId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/songs/${itemId}/share-urls`, undefined, undefined);
  },

  // getArtist
  getArtist: async (artistId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/artists/${artistId}`, undefined, undefined);
  },

  // getRelatedArtists
  getRelatedArtists: async (artistId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/artists/${artistId}/related`, undefined, undefined);
  },

  // getAlbum
  getAlbum: async (albumId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/albums/${albumId}`, undefined, undefined);
  },

  // getAlbumShareUrls
  getAlbumShareUrls: async (albumId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/albums/${albumId}/share-urls`, undefined, undefined);
  },

  // getPlaylists
  getPlaylists: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/playlists`, undefined, undefined);
  },

  // getPlaylistItems
  getPlaylistItems: async (playlistId: string): Promise<Result<any>> => {
    return apiRequest('GET', `/playlists/${playlistId}/items`, undefined, undefined);
  },

  // createPlaylist
  createPlaylist: async (data: PlaylistCreateData): Promise<Result<any>> => {
    return apiRequest('POST', `/playlists`, data, undefined);
  },

  // updatePlaylist
  updatePlaylist: async (playlistId: string, updates: PlaylistUpdateData): Promise<Result<any>> => {
    return apiRequest('PATCH', `/playlists/${playlistId}`, updates, undefined);
  },

  // deletePlaylist
  deletePlaylist: async (playlistId: string): Promise<Result<any>> => {
    return apiRequest('DELETE', `/playlists/${playlistId}`, undefined, undefined);
  },

  // addPlaylistItems
  addPlaylistItems: async (playlistId: string, songIds: string[]): Promise<Result<any>> => {
    return apiRequest('POST', `/playlists/${playlistId}/items`, { songIds: songIds }, undefined);
  },

  // removePlaylistItems
  removePlaylistItems: async (playlistId: string, songIds: string[]): Promise<Result<any>> => {
    return apiRequest('DELETE', `/playlists/${playlistId}/items`, undefined, { songIds: songIds });
  },

  // getHomeViewData
  getHomeViewData: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/home`, undefined, undefined);
  },

  // getRecentlyPlayed
  getRecentlyPlayed: async (limit?: number): Promise<Result<any>> => {
    return apiRequest('GET', `/home/recently-played`, undefined, { limit: limit });
  },

  // getImage
  getImage: async (itemId: string, imageType: string, serverUrl: string, token: string, width?: number, quality?: number): Promise<Result<any>> => {
    return apiRequest('GET', `/images/${itemId}`, undefined, { imageType: imageType, serverUrl: serverUrl, token: token, width: width, quality: quality });
  },

  // clearImageCache
  clearImageCache: async (): Promise<Result<any>> => {
    return apiRequest('POST', `/cache/image-clear`, undefined, undefined);
  },

  // getImageCacheStats
  getImageCacheStats: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/cache/image-stats`, undefined, undefined);
  },

  // getAudioStreamUrl
  getAudioStreamUrl: async (itemId: string, serverUrl: string, token: string, container?: string): Promise<Result<any>> => {
    return apiRequest('GET', `/audio/${itemId}/stream-url`, undefined, { serverUrl: serverUrl, token: token, container: container });
  },

  // getLyrics
  getLyrics: async (id: string, artist: string, title: string, path?: string): Promise<Result<any>> => {
    return apiRequest('POST', `/lyrics`, { id: id, artist: artist, title: title, path: path }, undefined);
  },

  // clearCache
  clearCache: async (): Promise<Result<any>> => {
    return apiRequest('POST', `/cache/clear`, undefined, undefined);
  },

  // listenbrainzSetCredentials
  listenbrainzSetCredentials: async (credentials: ListenBrainzCredentials): Promise<Result<any>> => {
    return apiRequest('POST', `/listenbrainz/credentials`, { credentials: credentials }, undefined);
  },

  // listenbrainzClearCredentials
  listenbrainzClearCredentials: async (): Promise<Result<any>> => {
    return apiRequest('DELETE', `/listenbrainz/credentials`, undefined, undefined);
  },

  // listenbrainzIsAuthenticated
  listenbrainzIsAuthenticated: async (): Promise<Result<any>> => {
    return apiRequest('GET', `/listenbrainz/auth-status`, undefined, undefined);
  },

  // listenbrainzValidateToken
  listenbrainzValidateToken: async (userToken: string): Promise<Result<any>> => {
    return apiRequest('POST', `/listenbrainz/validate`, { userToken: userToken }, undefined);
  },

  // listenbrainzSubmitListen
  listenbrainzSubmitListen: async (listen: ListenBrainzListen, timestamp: number): Promise<Result<any>> => {
    return apiRequest('POST', `/listenbrainz/submit-listen`, { listen: listen, timestamp: timestamp }, undefined);
  },

  // listenbrainzPlayingNow
  listenbrainzPlayingNow: async (artist: string, track: string, album?: string): Promise<Result<any>> => {
    return apiRequest('POST', `/listenbrainz/playing-now`, { artist: artist, track: track, album: album }, undefined);
  },

  // registerClientCapabilities
  registerClientCapabilities: async (serverUrl: string, token: string, deviceId: string): Promise<Result<any>> => {
    return apiRequest('POST', `/sessions/capabilities`, { serverUrl: serverUrl, token: token, deviceId: deviceId }, undefined);
  },

  // reportPlayback
  reportPlayback: async (serverUrl: string, token: string, itemId: string, positionTicks?: number, eventName?: string, isPaused?: boolean): Promise<Result<any>> => {
    return apiRequest('POST', `/sessions/playing`, { serverUrl: serverUrl, token: token, itemId: itemId, positionTicks: positionTicks, eventName: eventName, isPaused: isPaused }, undefined);
  },

  // Desktop-only: audioPlay
  audioPlay: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_audioPlay`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

  // Desktop-only: audioPause
  audioPause: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_audioPause`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

  // Desktop-only: audioStop
  audioStop: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_audioStop`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

  // Desktop-only: audioGetVolume
  audioGetVolume: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_audioGetVolume`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

  // Desktop-only: audioSetVolume
  audioSetVolume: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_audioSetVolume`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

  // Desktop-only: discordRpcStart
  discordRpcStart: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_discordRpcStart`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

  // Desktop-only: discordRpcStop
  discordRpcStop: async (...args: any[]): Promise<Result<any>> => {
    if (!isTauri) {
      return { status: 'error', error: 'Desktop-only feature' };
    }
    const command = `cmd_discordRpcStop`;
    try {
      const data = await invoke(command, args[0] || {});
      return { status: 'ok', data };
    } catch (error) {
      return { status: 'error', error: String(error) };
    }
  },

};
