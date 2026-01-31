/**
 * Consolidated artist type with all information
 */
export type Artist = { 
/**
 * Artist name
 */
name: string; 
/**
 * Artist ID
 */
id: string; 
/**
 * Image tags (metadata about available images)
 */
imageTags: Partial<{ [key in string]: string }> | null; 
/**
 * URL to artist image
 */
imageUrl: string | null; 
/**
 * Artist biography/description
 */
overview: string | null; 
/**
 * External provider IDs (`MusicBrainz`, etc.)
 */
providerIds: Partial<{ [key in string]: string }> | null; 
/**
 * Community rating
 */
communityRating: number | null; 
/**
 * Number of songs by this artist
 */
songCount: number | null; 
/**
 * Date last modified on server
 */
dateLastModified: string | null; 
/**
 * Optional list of songs by this artist (only populated when needed)
 */
songs: Song[] | null }