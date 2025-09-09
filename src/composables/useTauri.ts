import { commands } from '../bindings'
import type {
  MusicItem,
  ArtistInfo,
  AlbumWithSongs,
  ArtistWithSongs,
  Credentials,
  LoginResponse,
} from '../bindings'

// Re-export types for convenience
export type {
  MusicItem,
  ArtistInfo,
  AlbumWithSongs,
  ArtistWithSongs,
  Credentials,
  LoginResponse,
}

// Create a typed Tauri client
export const useTauri = () => {
  // Authentication commands
  const loginToJellyfin = async (serverUrl: string, username: string, password: string) => {
    const result = await commands.loginToJellyfin(serverUrl, username, password)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const saveCredentials = async (serverUrl: string, username: string, token: string, userId: string) => {
    const result = await commands.saveCredentials(serverUrl, username, token, userId)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getSavedCredentials = async (): Promise<Credentials | null> => {
    const result = await commands.getSavedCredentials()
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  // Music library commands
  const getMusicLibrary = async (serverUrl: string, token: string): Promise<MusicItem[]> => {
    const result = await commands.getMusicLibrary(serverUrl, token)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getAllArtists = async (serverUrl: string, token: string): Promise<ArtistInfo[]> => {
    const result = await commands.getAllArtists(serverUrl, token)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getArtistDetails = async (
    serverUrl: string,
    token: string,
    userId: string,
    artistId: string,
  ): Promise<ArtistInfo> => {
    const result = await commands.getArtistDetails(serverUrl, token, userId, artistId)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getAlbumsWithSongs = async (serverUrl: string, token: string): Promise<AlbumWithSongs[]> => {
    const result = await commands.getAlbumsWithSongs(serverUrl, token)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getArtistsWithSongs = async (
    serverUrl: string,
    token: string,
    albumArtistsOnly: boolean = false,
  ): Promise<ArtistWithSongs[]> => {
    const result = await commands.getArtistsWithSongs(serverUrl, token, albumArtistsOnly)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getAudioStreamUrl = async (
    serverUrl: string,
    token: string,
    itemId: string,
    container?: string | null,
  ): Promise<string> => {
    const result = await commands.getAudioStreamUrl(serverUrl, token, itemId, container || null)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  // Volume commands
  const saveVolume = async (volume: number) => {
    const result = await commands.saveVolume(volume)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getSavedVolume = async (): Promise<number | null> => {
    const result = await commands.getSavedVolume()
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  // Favorites commands
  const toggleFavoriteStatus = async (
    serverUrl: string,
    token: string,
    userId: string,
    itemId: string,
    isFavorite: boolean,
  ): Promise<boolean> => {
    const result = await commands.toggleFavoriteStatus(serverUrl, token, userId, itemId, isFavorite)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  // Cache commands
  const clearMusicCache = async () => {
    const result = await commands.clearMusicCache()
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  // Lyrics commands
  const getLyrics = async (
    id: string,
    artist: string,
    title: string,
    path?: string | null,
  ): Promise<string> => {
    const result = await commands.getLyrics(id, artist, title, path || null)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  return {
    // Auth
    loginToJellyfin,
    saveCredentials,
    getSavedCredentials,

    // Music
    getMusicLibrary,
    getAllArtists,
    getArtistDetails,
    getAlbumsWithSongs,
    getArtistsWithSongs,
    getAudioStreamUrl,

    // Volume
    saveVolume,
    getSavedVolume,

    // Favorites
    toggleFavoriteStatus,

    // Cache
    clearMusicCache,

    // Lyrics
    getLyrics,
  }
}
