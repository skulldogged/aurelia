// HTTP API Client Implementation for Web Version
// Communicates with the Axum backend server

import type {
  ApiClient,
  AudioStreamParams,
  Credentials,
  EQBand,
  EQPreset,
  ImageParams,
  LastFmCredentials,
  LibraryData,
  ListenBrainzCredentials,
  ListenBrainzListen,
  PlaybackProgress,
  Playlist,
  PlaylistCreateData,
  PlaylistUpdateData,
  Result,
  ShareUrlType,
  SyncStateInfo,
} from './types'

// Configuration
const API_BASE_URL = (import.meta as any).env?.VITE_API_URL || ''

// Helper to make HTTP requests
async function httpRequest<T>(method: string, path: string, body?: unknown): Promise<Result<T>> {
  try {
    const response = await fetch(`${API_BASE_URL}${path}`, {
      method,
      headers: {
        'Content-Type': 'application/json',
      },
      body: body ? JSON.stringify(body) : undefined,
      credentials: 'include', // Include session cookies
    })

    const responseText = await response.text()
    
    let responseData: any
    try {
      responseData = responseText ? JSON.parse(responseText) : null
    } catch (e) {
      if (!response.ok) {
        return { status: 'error', error: responseText || response.statusText }
      }
      return { status: 'error', error: 'Failed to parse response' }
    }

    if (!response.ok) {
      if (responseData && responseData.status === 'error') {
        return responseData as Result<T>
      }
      return { status: 'error', error: responseText || response.statusText }
    }

    // Backend returns { status: "ok", data: T } or { status: "error", error: string }
    return responseData as Result<T>
  } catch (error) {
    return {
      status: 'error',
      error: error instanceof Error ? error.message : 'Network error',
    }
  }
}

// HTTP API Client implementation
export const httpClient: ApiClient = {
  // Auth
  async getSavedCredentials(): Promise<Result<Credentials | null>> {
    return httpRequest('GET', '/api/auth/credentials')
  },

  async authenticate(serverUrl: string, username: string, password: string): Promise<Result<Credentials>> {
    return httpRequest('POST', '/api/auth/login', { serverUrl, username, password })
  },

  async logout(): Promise<Result<void>> {
    return httpRequest('POST', '/api/auth/logout')
  },

  async saveCredentials(): Promise<Result<void>> {
    // Backend saves credentials automatically on successful login
    return { status: 'ok', data: undefined }
  },

  // Library
  async getLibrary(): Promise<Result<LibraryData>> {
    return httpRequest('GET', '/api/library')
  },

  async syncLibrary(): Promise<Result<void>> {
    return httpRequest('POST', '/api/library/sync')
  },

  async getSyncState(): Promise<Result<SyncStateInfo>> {
    return httpRequest('GET', '/api/library/sync-state')
  },

  async getSong(songId: string) {
    return httpRequest('GET', `/api/songs/${songId}`)
  },

  async getArtist(artistId: string) {
    return httpRequest('GET', `/api/artists/${artistId}`)
  },

  // Playlists
  async getPlaylists(): Promise<Result<Playlist[]>> {
    return httpRequest('GET', '/api/playlists')
  },

  async getPlaylistItems(playlistId: string) {
    return httpRequest('GET', `/api/playlists/${playlistId}/items`)
  },

  async createPlaylist(data: PlaylistCreateData) {
    return httpRequest('POST', '/api/playlists', data)
  },

  async updatePlaylist(playlistId: string, updates: PlaylistUpdateData) {
    return httpRequest('PATCH', `/api/playlists/${playlistId}`, updates)
  },

  async deletePlaylist(playlistId: string) {
    return httpRequest('DELETE', `/api/playlists/${playlistId}`)
  },

  async addPlaylistItems(playlistId: string, itemIds: string[]) {
    return httpRequest('POST', `/api/playlists/${playlistId}/items`, { itemIds })
  },

  async removePlaylistItems(playlistId: string, itemIds: string[]) {
    return httpRequest('DELETE', `/api/playlists/${playlistId}/items`, { itemIds })
  },

  // Home
  async getHomeViewData() {
    return httpRequest('GET', '/api/home')
  },

  async getRecentlyPlayed(serverUrl: string, token: string, userId: string) {
    return httpRequest('GET', `/api/home/recently-played?serverUrl=${encodeURIComponent(serverUrl)}&token=${encodeURIComponent(token)}&userId=${encodeURIComponent(userId)}`)
  },

  // Audio - Returns Jellyfin stream URL directly
  async getAudioStreamUrl(params: AudioStreamParams): Promise<Result<string>> {
    const payload = {
      serverUrl: params.serverUrl,
      token:     params.token,
      itemId:    params.itemId,
      container: params.container || null,
    }
    const result = await httpRequest<string>('POST', '/api/audio/stream-url', payload)
    
    if (result.status === 'ok' && result.data) {
      // Proxy the URL through our backend to avoid CORS issues
      const proxiedUrl = `/api/audio/proxy?url=${encodeURIComponent(result.data)}`
      return { status: 'ok', data: proxiedUrl }
    }
    
    return result
  },

  async getSavedVolume() {
    // Store in localStorage for web
    const volume = localStorage.getItem('volume')
    return { status: 'ok' as const, data: volume ? parseFloat(volume) : null }
  },

  async saveVolume(volume: number) {
    localStorage.setItem('volume', volume.toString())
    return { status: 'ok' as const, data: undefined }
  },

  // Images - Return direct Jellyfin URL
  async getImage(params: ImageParams): Promise<Result<string | null>> {
    const { serverUrl, token, itemId, imageType, width, quality } = params
    let url = `${serverUrl}/Items/${itemId}/Images/${imageType}?api_key=${token}`
    if (width) url += `&width=${width}`
    if (quality) url += `&quality=${quality}`
    return { status: 'ok', data: url }
  },

  async clearCache(): Promise<Result<void>> {
    return httpRequest('POST', '/api/library/clear-cache')
  },

  async clearImageFromCache() {
    // No-op for web - browser handles caching
    return { status: 'ok', data: undefined }
  },

  async getImageCacheStats() {
    return { status: 'ok', data: 'Browser cache' }
  },

  // Lyrics
  async getLyrics(id: string, artist: string, title: string, path?: string) {
    return httpRequest('POST', '/api/lyrics', { id, artist, title, path })
  },

  // Favorites
  async toggleFavoriteStatus(itemId: string, isFavorite: boolean) {
    return httpRequest('POST', `/api/songs/${itemId}/favorite`, { isFavorite })
  },

  // Instant Mix
  async getInstantMix(itemId: string) {
    return httpRequest('GET', `/api/songs/${itemId}/instant-mix`)
  },

  // Related Artists
  async getRelatedArtists(artistId: string) {
    return httpRequest('GET', `/api/artists/${artistId}/related`)
  },

  // Share URLs
  async getSongShareUrls(songId: string) {
    return httpRequest('GET', `/api/songs/${songId}/share-urls`)
  },

  // Last.fm - These would need to be implemented in the backend
  // or called directly from browser to Last.fm API
  async lastfmSetCredentials(credentials: LastFmCredentials) {
    return httpRequest('POST', '/api/integrations/lastfm/credentials', credentials)
  },

  async lastfmClearCredentials() {
    return httpRequest('DELETE', '/api/integrations/lastfm/credentials')
  },

  async lastfmIsAuthenticated() {
    return httpRequest('GET', '/api/integrations/lastfm/authenticated')
  },

  async lastfmAuthenticate() {
    return httpRequest('POST', '/api/integrations/lastfm/auth')
  },

  async lastfmScrobble(artist: string, track: string, album?: string, timestamp?: number) {
    return httpRequest('POST', '/api/integrations/lastfm/scrobble', { artist, track, album, timestamp })
  },

  async lastfmUpdateNowPlaying(artist: string, track: string, album?: string) {
    return httpRequest('POST', '/api/integrations/lastfm/now-playing', { artist, track, album })
  },

  // ListenBrainz
  async listenbrainzSetCredentials(credentials: ListenBrainzCredentials) {
    return httpRequest('POST', '/api/integrations/listenbrainz/credentials', credentials)
  },

  async listenbrainzClearCredentials() {
    return httpRequest('DELETE', '/api/integrations/listenbrainz/credentials')
  },

  async listenbrainzIsAuthenticated() {
    return httpRequest('GET', '/api/integrations/listenbrainz/authenticated')
  },

  async listenbrainzValidateToken(userToken: string) {
    return httpRequest('POST', '/api/integrations/listenbrainz/validate', { userToken })
  },

  async listenbrainzSubmitListen(listen: ListenBrainzListen, timestamp: number) {
    return httpRequest('POST', '/api/integrations/listenbrainz/submit', { listen, timestamp })
  },

  async listenbrainzPlayingNow(artist: string, track: string, album?: string) {
    return httpRequest('POST', '/api/integrations/listenbrainz/playing-now', { artist, track, album })
  },

  // Audio - Not implemented for web (uses Web Audio API directly)
  audioInit: undefined,
  audioPlay: undefined,
  audioPause: undefined,
  audioResume: undefined,
  audioStop: undefined,
  audioGetPosition: undefined,
  audioSeek: undefined,
  audioGetVolume: undefined,
  audioSetVolume: undefined,
  audioSetEqBand: undefined,
  audioGetEqBand: undefined,
  audioGetAllEqBands: undefined,
  audioSetEqEnabled: undefined,
  audioIsEqEnabled: undefined,
  audioSetEqPreset: undefined,
  audioGetEqPreset: undefined,
  audioIsAnalyzerEnabled: undefined,
  audioSetAnalyzerEnabled: undefined,
  audioPrepareNext: undefined,
  audioAdvanceGapless: undefined,

  // Session - Web handles this directly via WebSocket
  async registerClientCapabilities() { return { status: 'ok', data: undefined } },
  async reportPlaybackStart() { return { status: 'ok', data: undefined } },
  async reportPlaybackProgress() { return { status: 'ok', data: undefined } },
  async reportPlaybackStop() { return { status: 'ok', data: undefined } },
  async markItemPlayed() { return { status: 'ok', data: undefined } },

  // System Tray / Window Management - No-op for web
  async showMainWindow() { return { status: 'ok', data: undefined } },
  async hideMainWindow() { return { status: 'ok', data: undefined } },
  async quitApplication() { return { status: 'ok', data: undefined } },
  async setMinimizeToTray() { return { status: 'ok', data: undefined } },
  async setCloseToTray() { return { status: 'ok', data: undefined } },
}

// WebSocket Client for real-time updates
export class WebSocketClient {
  private ws: WebSocket | null = null
  private syncStateCallbacks: Set<(state: SyncStateInfo) => void> = new Set()
  private progressCallbacks: Set<(progress: PlaybackProgress) => void> = new Set()
  private libraryCallbacks: Set<() => void> = new Set()
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5

  onSyncState(callback: (state: SyncStateInfo) => void): () => void {
    this.syncStateCallbacks.add(callback)
    return () => this.syncStateCallbacks.delete(callback)
  }

  onPlaybackProgress(callback: (progress: PlaybackProgress) => void): () => void {
    this.progressCallbacks.add(callback)
    return () => this.progressCallbacks.delete(callback)
  }

  onLibraryUpdate(callback: () => void): () => void {
    this.libraryCallbacks.add(callback)
    return () => this.libraryCallbacks.delete(callback)
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const wsUrl = API_BASE_URL
        ? API_BASE_URL.replace('http', 'ws')
        : `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}`
      this.ws = new WebSocket(`${wsUrl}/ws`)

      this.ws.onopen = () => {
        this.reconnectAttempts = 0
        resolve()
      }

      this.ws.onmessage = (event) => {
        const message = JSON.parse(event.data)
        this.handleMessage(message)
      }

      this.ws.onclose = () => {
        this.attemptReconnect()
      }

      this.ws.onerror = (error) => {
        reject(error)
      }
    })
  }

  disconnect(): void {
    this.ws?.close()
    this.ws = null
  }

  private handleMessage(message: any): void {
    switch (message.type) {
      case 'syncState':
        this.syncStateCallbacks.forEach(cb => cb(message.data))
        break
      case 'playbackProgress':
        this.progressCallbacks.forEach(cb => cb(message.data))
        break
      case 'libraryUpdate':
        this.libraryCallbacks.forEach(cb => cb())
        break
    }
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) return

    this.reconnectAttempts++
    setTimeout(() => {
      this.connect().catch(() => {
        // Reconnect failed, will try again if under max attempts
      })
    }, 1000 * this.reconnectAttempts)
  }
}

export { ApiClient, Result }
export * from './types'
