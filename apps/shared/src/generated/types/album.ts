/**
 * Consolidated album type with all information
 */
export type Album = { 
/**
 * Album ID from Jellyfin
 */
id: string | null; 
/**
 * Album name
 */
name: string; 
/**
 * Primary artist name
 */
artist: string; 
/**
 * Primary artist ID
 */
artistId: string | null; 
/**
 * URL to album artwork
 */
albumArtUrl: string | null; 
/**
 * Number of songs in album
 */
songCount: number; 
/**
 * Optional list of songs in this album (only populated when needed)
 */
songs: Song[] | null; 
/**
 * Image tags
 */
imageTags: Partial<{ [key in string]: string }> | null; 
/**
 * External provider IDs (`MusicBrainz`, etc.)
 */
providerIds: Partial<{ [key in string]: string }> | null; 
/**
 * Date created (when added to server)
 */
dateCreated: string | null; 
/**
 * Date last modified on server
 */
dateLastModified: string | null }