/**
 * User credentials for Jellyfin authentication
 */
export type Credentials = { 
/**
 * Jellyfin server URL
 */
serverUrl: string; 
/**
 * Username
 */
username: string; 
/**
 * Authentication token
 */
token: string; 
/**
 * User ID
 */
userId: string }