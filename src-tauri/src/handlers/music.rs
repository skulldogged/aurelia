//! Music-related command handlers

use crate::db;
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

    if let Err(e) = db::cache_music_library(&items).await {
        eprintln!("Failed to cache music library: {}", e);
    }

    Ok(items)
}

/// Get music library, using cache if available
#[tauri::command]
#[specta::specta]
pub async fn get_music_library(server_url: String, token: String) -> Result<Vec<Song>, String> {
    // Try to get from cache first
    match db::get_cached_music_library().await {
        Ok(items) if !items.is_empty() => return Ok(items),
        _ => {}
    }

    // Fetch from server if cache is empty or fails
    fetch_and_cache_library(server_url, token).await
}

/// Get all albums
#[tauri::command]
#[specta::specta]
pub async fn get_all_albums() -> Result<Vec<Album>, String> {
    db::get_all_albums().await.map_err(|e| e.to_string())
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

/// Get all artists from the server
#[tauri::command]
#[specta::specta]
pub async fn get_all_artists(server_url: String, token: String) -> Result<Vec<Artist>, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    let artists = client.get_all_artists().await.map_err(|e| e.to_string())?;
    Ok(artists.into_iter().collect())
}

/// Get artists with their songs
#[tauri::command]
#[specta::specta]
pub async fn get_artists_with_songs(
    server_url: String,
    token: String,
    album_artists_only: bool,
) -> Result<Vec<Artist>, String> {
    // Get all songs
    let songs = get_music_library(server_url.clone(), token.clone()).await?;

    // Get all artists to get artist metadata
    let artists = get_all_artists(server_url, token).await?;

    // Create case-insensitive maps for artist name lookup
    let mut artist_name_to_info: HashMap<String, Artist> = HashMap::new();
    let mut artist_name_lower_to_original: HashMap<String, String> = HashMap::new();

    for artist in artists {
        let original_name = artist.name.clone();
        let lower_name = original_name.to_lowercase();

        artist_name_to_info.insert(original_name.clone(), artist);
        artist_name_lower_to_original.insert(lower_name, original_name);
    }

    // Group songs by artist
    let mut artist_map: HashMap<String, Vec<Song>> = HashMap::new();

    if album_artists_only {
        for song in &songs {
            if let Some(artists) = &song.album_artists {
                for artist in artists {
                    artist_map
                        .entry(artist.name.clone())
                        .or_default()
                        .push(song.clone());
                }
            }
        }
    } else {
        for song in &songs {
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
    }

    // Convert to Artist
    let mut artists_with_songs: Vec<Artist> = Vec::new();

    for (artist_name, songs) in artist_map {
        if songs.is_empty() {
            continue;
        }

        // Get artist info using case-insensitive lookup
        let artist_info = artist_name_to_info.get(&artist_name).or_else(|| {
            // Try case-insensitive lookup
            let lower_name = artist_name.to_lowercase();
            artist_name_lower_to_original
                .get(&lower_name)
                .and_then(|original_name| artist_name_to_info.get(original_name))
        });

        let artist_with_songs = Artist {
            name: artist_name,
            id: artist_info.map(|a| a.id.clone()).unwrap_or_default(),
            image_tags: None,
            image_url: artist_info.and_then(|a| a.image_url.clone()),
            overview: artist_info.and_then(|a| a.overview.clone()),
            provider_ids: artist_info.and_then(|a| a.provider_ids.clone()),
            community_rating: artist_info.and_then(|a| a.community_rating),
            song_count: Some(songs.len() as i32),
            songs: Some(songs),
        };

        artists_with_songs.push(artist_with_songs);
    }

    // Sort artists by name
    artists_with_songs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(artists_with_songs)
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

/// Get details for a single artist
#[tauri::command]
#[specta::specta]
pub async fn get_artist_details(
    server_url: String,
    token: String,
    artist_id: String,
) -> Result<Artist, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    let user_id = match crate::handlers::auth::get_saved_credentials() {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };
    client
        .get_artist_details(&user_id, &artist_id)
        .await
        .map_err(|e| e.to_string())
}

/// Clear the music cache, then re-fetch and cache the library
#[tauri::command]
#[specta::specta]
pub async fn clear_music_cache(server_url: String, token: String) -> Result<(), String> {
    db::clear_music_cache().await?;
    // Re-populate the cache immediately
    fetch_and_cache_library(server_url, token).await?;
    Ok(())
}
