//! Music-related command handlers

use crate::cache;
use crate::models::{Album, Artist, Song};
use crate::services::JellyfinClient;
use std::collections::HashMap;

/// Helper function to fetch library from Jellyfin and cache it
async fn fetch_and_cache_library(server_url: String, token: String) -> Result<Vec<Song>, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    let user_id = match crate::handlers::auth::get_saved_credentials() {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };
    let items = client
        .get_music_library(&user_id)
        .await
        .map_err(|e| e.to_string())?;

    if let Err(e) = cache::cache_library(&items).await {
        eprintln!("Failed to cache music library: {}", e);
        return Err(e);
    }

    Ok(items)
}

/// Get songs with optional filtering
#[tauri::command]
#[specta::specta]
pub async fn get_songs(
    server_url: Option<String>,
    token: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    album_id: Option<String>,
    artist_id: Option<String>,
) -> Result<Vec<Song>, String> {
    // Get all songs from cache first
    let all_songs = match cache::get_songs().await {
        Ok(items) if !items.is_empty() => items,
        _ => {
            // Fetch from server if cache is empty or fails
            if let (Some(server_url), Some(token)) = (server_url, token) {
                fetch_and_cache_library(server_url, token).await?
            } else {
                return Err(
                    "No cached songs available and no server credentials provided".to_string(),
                );
            }
        }
    };

    let mut filtered_songs = all_songs;

    // Apply filters
    if let Some(album_id) = album_id {
        filtered_songs.retain(|song| song.album_id.as_ref() == Some(&album_id));
    }

    if let Some(artist_id) = artist_id {
        filtered_songs.retain(|song| {
            song.artist_ids
                .as_ref()
                .map(|ids| ids.contains(&artist_id))
                .unwrap_or(false)
        });
    }

    // Apply pagination
    if let Some(offset) = offset {
        filtered_songs = filtered_songs.into_iter().skip(offset as usize).collect();
    }

    if let Some(limit) = limit {
        filtered_songs = filtered_songs.into_iter().take(limit as usize).collect();
    }

    Ok(filtered_songs)
}

/// Get single song by ID
#[tauri::command]
#[specta::specta]
pub async fn get_song(song_id: String) -> Result<Song, String> {
    let songs = cache::get_songs().await.map_err(|e| e.to_string())?;

    songs
        .into_iter()
        .find(|song| song.id == song_id)
        .ok_or_else(|| format!("Song with ID '{}' not found", song_id))
}

/// Get albums with optional filtering and song inclusion
#[tauri::command]
#[specta::specta]
pub async fn get_albums(
    server_url: Option<String>,
    token: Option<String>,
    include_songs: Option<bool>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Album>, String> {
    let mut albums = if let (Some(server_url), Some(token)) = (server_url, token) {
        // Fetch fresh albums from Jellyfin
        let client = crate::services::JellyfinClient::with_auth(server_url, token);
        let user_id = crate::handlers::auth::get_saved_credentials()
            .map_err(|e| e.to_string())?
            .map(|creds| creds.user_id)
            .ok_or("No saved credentials found")?;

        client
            .get_albums(&user_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        // Fall back to cached albums
        cache::get_albums().await.map_err(|e| e.to_string())?
    };

    // Apply pagination
    if let Some(offset) = offset {
        albums = albums.into_iter().skip(offset as usize).collect();
    }

    if let Some(limit) = limit {
        albums = albums.into_iter().take(limit as usize).collect();
    }

    // Include songs if requested
    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

        // Group songs by album
        let mut album_songs: std::collections::HashMap<String, Vec<Song>> =
            std::collections::HashMap::new();
        for song in all_songs {
            if let Some(album_id) = &song.album_id {
                album_songs.entry(album_id.clone()).or_default().push(song);
            }
        }

        // Add songs to albums
        for album in &mut albums {
            if let Some(id) = &album.id {
                if let Some(songs) = album_songs.get(id) {
                    album.songs = Some(songs.clone());
                }
            }
        }
    }

    Ok(albums)
}

/// Get single album by ID
#[tauri::command]
#[specta::specta]
pub async fn get_album(album_id: String, include_songs: Option<bool>) -> Result<Album, String> {
    let albums = cache::get_albums().await.map_err(|e| e.to_string())?;

    let mut album = albums
        .into_iter()
        .find(|album| album.id.as_ref() == Some(&album_id))
        .ok_or_else(|| format!("Album with ID '{}' not found", album_id))?;

    // Include songs if requested
    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

        let album_songs: Vec<Song> = all_songs
            .into_iter()
            .filter(|song| song.album_id.as_ref() == Some(&album_id))
            .collect();

        album.songs = Some(album_songs);
    }

    Ok(album)
}

/// Get recently played songs
#[tauri::command]
#[specta::specta]
pub async fn get_recently_played(server_url: String, token: String) -> Result<Vec<Song>, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    let user_id = match crate::handlers::auth::get_saved_credentials() {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };

    client
        .get_recently_played(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get artists with optional filtering and song inclusion
#[tauri::command]
#[specta::specta]
pub async fn get_artists(
    server_url: Option<String>,
    token: Option<String>,
    include_songs: Option<bool>,
    album_artists_only: Option<bool>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Artist>, String> {
    // Choose the correct endpoint based on album_artists_only flag
    let artists = if album_artists_only.unwrap_or(false) {
        // Album artists only - use Jellyfin's album artists endpoint
        if let (Some(server_url), Some(token)) = (server_url, token) {
            let client = JellyfinClient::with_auth(server_url, token);
            client
                .get_album_artists()
                .await
                .map_err(|e| e.to_string())?
        } else {
            return Err("No server credentials provided for album artists".to_string());
        }
    } else {
        // All artists - use cached data or fetch from API
        match crate::cache::get_artists().await {
            Ok(cached_artists) if !cached_artists.is_empty() => cached_artists,
            _ => {
                if let (Some(server_url), Some(token)) = (server_url, token) {
                    let client = JellyfinClient::with_auth(server_url, token);
                    let fresh_artists =
                        client.get_all_artists().await.map_err(|e| e.to_string())?;

                    // Cache the artists
                    if let Err(e) = crate::cache::cache_artists(&fresh_artists).await {
                        eprintln!("Warning: Failed to cache artists: {}", e);
                    }

                    fresh_artists
                } else {
                    return Err(
                        "No cached artists available and no server credentials provided"
                            .to_string(),
                    );
                }
            }
        }
    };

    let mut result_artists = artists;

    // Apply pagination
    if let Some(offset) = offset {
        result_artists = result_artists.into_iter().skip(offset as usize).collect();
    }

    if let Some(limit) = limit {
        result_artists = result_artists.into_iter().take(limit as usize).collect();
    }

    // Include songs if requested
    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

        // Group songs by artist
        let mut artist_map: HashMap<String, Vec<Song>> = HashMap::new();

        for song in &all_songs {
            if let Some(artists) = &song.artists {
                for artist_name in artists {
                    artist_map
                        .entry(artist_name.clone())
                        .or_default()
                        .push(song.clone());
                }
            } else {
                // Handle songs with no artist
                artist_map
                    .entry("Unknown Artist".to_string())
                    .or_default()
                    .push(song.clone());
            }
        }

        // Update artists with songs
        for artist in &mut result_artists {
            if let Some(songs) = artist_map.get(&artist.name) {
                artist.song_count = Some(songs.len() as i32);
                artist.songs = Some(songs.clone());
            } else {
                artist.song_count = Some(0);
                artist.songs = Some(Vec::new());
            }
        }
    }

    // Sort artists by name
    result_artists.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result_artists)
}

/// Get single artist by ID
#[tauri::command]
#[specta::specta]
pub async fn get_artist(
    artist_id: String,
    include_songs: Option<bool>,
    album_artists_only: Option<bool>,
) -> Result<Artist, String> {
    let artists = crate::cache::get_artists()
        .await
        .map_err(|e| e.to_string())?;

    let mut artist = artists
        .into_iter()
        .find(|artist| artist.id == artist_id)
        .ok_or_else(|| format!("Artist with ID '{}' not found", artist_id))?;

    // Include songs if requested
    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

        let artist_songs: Vec<Song> = if album_artists_only.unwrap_or(false) {
            // Album artists mode: check if this artist is actually an album artist
            let mut is_album_artist = false;
            for song in &all_songs {
                if let Some(artists) = &song.album_artists {
                    if artists.iter().any(|a| a.id == artist_id) {
                        is_album_artist = true;
                        break;
                    }
                }
            }

            if is_album_artist {
                all_songs
                    .into_iter()
                    .filter(|song| {
                        song.artist_ids
                            .as_ref()
                            .map(|ids| ids.contains(&artist_id))
                            .unwrap_or(false)
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            all_songs
                .into_iter()
                .filter(|song| {
                    song.artist_ids
                        .as_ref()
                        .map(|ids| ids.contains(&artist_id))
                        .unwrap_or(false)
                })
                .collect()
        };

        artist.song_count = Some(artist_songs.len() as i32);
        artist.songs = Some(artist_songs);
    }

    Ok(artist)
}

/// Get audio stream URL
#[tauri::command]
#[specta::specta]
pub async fn get_audio_stream_url(
    server_url: String,
    token: String,
    item_id: String,
    container: Option<String>,
) -> Result<String, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    Ok(client.get_audio_stream_url(&item_id, container.as_deref()))
}

/// Toggle favorite status for a track
#[tauri::command]
#[specta::specta]
pub async fn toggle_favorite_status(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    is_favorite: bool,
) -> Result<bool, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    client
        .toggle_favorite(&user_id, &item_id, is_favorite)
        .await
        .map_err(|e| e.to_string())?;

    // Return the new favorite status
    Ok(is_favorite)
}

/// Sync music library - update existing data without clearing cache
#[tauri::command]
#[specta::specta]
pub async fn sync_library(server_url: String, token: String) -> Result<(), String> {
    // Fetch fresh data from server and update cache
    fetch_and_cache_library(server_url, token).await?;
    Ok(())
}

/// Clear the music cache, then re-fetch and cache the library
#[tauri::command]
#[specta::specta]
pub async fn clear_cache(server_url: String, token: String) -> Result<(), String> {
    cache::clear_cache().await?;
    // Re-populate the cache immediately
    fetch_and_cache_library(server_url, token).await?;
    Ok(())
}
