import { commands } from '../bindings'
import type { Artist, Song, Credentials, LoginResponse, Album } from '../bindings'

// Re-export types for convenience
export type { Song as MusicItem, Artist, Album, Credentials, LoginResponse }

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
  const getMusicLibrary = async (serverUrl: string, token: string): Promise<Song[]> => {
    const credentials = await getSavedCredentials()
    if (!credentials) throw new Error('No saved credentials found')
    const result = await commands.getMusicLibrary(serverUrl, token)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getRecentlyPlayed = async (serverUrl: string, token: string): Promise<Song[]> => {
    const result = await commands.getRecentlyPlayed(serverUrl, token)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getAllArtists = async (serverUrl: string, token: string): Promise<Artist[]> => {
    const credentials = await getSavedCredentials()
    if (!credentials) throw new Error('No saved credentials found')
    const result = await commands.getAllArtists(serverUrl, token)
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getAllAlbums = async (): Promise<Album[]> => {
    const result = await commands.getAllAlbums()
    if (result.status === 'error') {
      throw new Error(result.error)
    }
    return result.data
  }

  const getArtistsWithSongs = async (
    serverUrl: string,
    token: string,
    albumArtistsOnly: boolean = false,
  ): Promise<Artist[]> => {
    const credentials = await getSavedCredentials()
    if (!credentials) throw new Error('No saved credentials found')
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

  // Artist commands
  const getArtistDetails = async (
    serverUrl: string,
    token: string,
    artistId: string,
  ): Promise<Artist> => {
    const result = await commands.getArtistDetails(serverUrl, token, artistId)
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
  const clearMusicCache = async (serverUrl: string, token: string) => {
    const result = await commands.clearMusicCache(serverUrl, token)
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
    getRecentlyPlayed,
    getAllArtists,
    getAllAlbums,
    getArtistsWithSongs,
    getAudioStreamUrl,
    getArtistDetails,

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
