/**
 * Playlist representing a collection of items
 */
export type Playlist = { 
/**
 * Playlist name
 */
name: string; 
/**
 * Server ID
 */
serverId: string; 
/**
 * Playlist ID
 */
id: string; 
/**
 * Whether playlist can be deleted
 */
canDelete: boolean | null; 
/**
 * Sort name
 */
sortName: string | null; 
/**
 * Whether this is a folder (playlists are folders containing items)
 */
isFolder: boolean; 
/**
 * Item type (should be "Playlist")
 */
itemType: string; 
/**
 * User data
 */
userData: UserData | null; 
/**
 * Runtime ticks (total duration)
 */
runTimeTicks: number | null; 
/**
 * Child count (number of items in playlist)
 */
childCount: number | null; 
/**
 * Image tags
 */
imageTags: Partial<{ [key in string]: string }> | null; 
/**
 * Backdrop image tags
 */
backdropImageTags: string[] | null; 
/**
 * Image blur hashes
 */
imageBlurHashes: Partial<{ [key in string]: Partial<{ [key in string]: string }> }> | null; 
/**
 * Location type
 */
locationType: string; 
/**
 * Media type
 */
mediaType: string | null; 
/**
 * Date created
 */
dateCreated: string | null; 
/**
 * Date last modified
 */
dateLastSaved: string | null; 
/**
 * Whether playlist is favorited
 */
isFavorite: boolean | null; 
/**
 * Playlist description
 */
description: string | null; 
/**
 * Songs in the playlist
 */
songs: Song[] | null }