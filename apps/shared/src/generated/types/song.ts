/**
 * Song representing a music track or audio file
 */
export type Song = { 
/**
 * Unique identifier
 */
id: string; 
/**
 * Song title
 */
name: string; 
/**
 * Type of item (usually "Audio")
 */
itemType: string; 
/**
 * Album name
 */
album: string | null; 
/**
 * Album ID
 */
albumId: string | null; 
/**
 * List of artist names
 */
artists: string[] | null; 
/**
 * List of artist IDs corresponding to artists
 */
artistIds: string[] | null; 
/**
 * File path
 */
path: string | null; 
/**
 * Duration in seconds
 */
duration: number | null; 
/**
 * URL to album artwork
 */
albumArtUrl: string | null; 
/**
 * Release year
 */
year: number | null; 
/**
 * Number of times played
 */
playCount: number | null; 
/**
 * Whether this item is marked as favorite
 */
isFavorite: boolean | null; 
/**
 * Disc number in album
 */
discNumber: number | null; 
/**
 * Track number in album
 */
trackNumber: number | null; 
/**
 * Audio container/format
 */
container: string | null; 
/**
 * Audio bitrate
 */
bitRate: number | null; 
/**
 * Audio sample rate
 */
sampleRate: number | null; 
/**
 * Audio codec
 */
codec: string | null; 
/**
 * Music genres
 */
genres: string[] | null; 
/**
 * Premiere/release date
 */
premiereDate: string | null; 
/**
 * Last played date
 */
datePlayed: string | null; 
/**
 * Date created (when added to server)
 */
dateCreated: string | null; 
/**
 * Date last modified on server
 */
dateLastModified: string | null; 
/**
 * Album artists (different from track artists)
 */
albumArtists: NameIdPair[] | null; 
/**
 * Song lyrics
 */
lyrics: string | null; 
/**
 * Image tags
 */
imageTags: Partial<{ [key in string]: string }> | null }