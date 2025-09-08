//! Music-related command handlers

use crate::db;
use crate::models::{AlbumWithSongs, ArtistInfo, ArtistWithSongs, MusicItem};
use crate::services::JellyfinClient;
use std::collections::HashMap;

/// Get music library, using cache if available
#[tauri::command]
pub async fn get_music_library(
    server_url: String,
    token: String,
) -> Result<Vec<MusicItem>, String> {
    // Try to get from cache first
    match db::get_cached_music_library() {
        Ok(items) if !items.is_empty() => {
            return Ok(items);
        }
        _ => {
            // Cache is empty or there was an error, proceed to fetch from server
        }
    }

    // Fetch from server
    let client = JellyfinClient::with_auth(server_url, token);

    // Get user ID from saved credentials
    let user_id = match crate::handlers::auth::get_saved_credentials() {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };

    let items = client
        .get_music_library(&user_id)
        .await
        .map_err(|e| e.to_string())?;

    // Cache the results
    if let Err(e) = db::cache_music_library(&items) {
        eprintln!("Failed to cache music library: {}", e);
    }

    Ok(items)
}

/// Get artist details
#[tauri::command]
pub async fn get_artist_details(
    server_url: String,
    token: String,
    user_id: String,
    artist_id: String,
) -> Result<ArtistInfo, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    client
        .get_artist_details(&user_id, &artist_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get all artists from the server
#[tauri::command]
pub async fn get_all_artists(server_url: String, token: String) -> Result<Vec<ArtistInfo>, String> {
    let client = JellyfinClient::with_auth(server_url, token);
    client.get_all_artists().await.map_err(|e| e.to_string())
}

/// Get albums with their songs
#[tauri::command]
pub async fn get_albums_with_songs(
    server_url: String,
    token: String,
) -> Result<Vec<AlbumWithSongs>, String> {
    // Get all songs
    let songs = get_music_library(server_url.clone(), token.clone()).await?;

    // Group songs by album
    let mut album_map: HashMap<String, Vec<MusicItem>> = HashMap::new();

    for song in songs {
        let album_name = song
            .album
            .clone()
            .unwrap_or_else(|| "Unknown Album".to_string());
        album_map.entry(album_name).or_default().push(song);
    }

    // Convert to AlbumWithSongs
    let mut albums_with_songs: Vec<AlbumWithSongs> = Vec::new();

    for (album_name, songs) in album_map {
        if songs.is_empty() {
            continue;
        }

        // Get primary artist info from first song
        let primary_song = &songs[0];
        let primary_artist_name = primary_song
            .artists
            .as_ref()
            .and_then(|artists| artists.first())
            .unwrap_or(&"Unknown Artist".to_string())
            .clone();

        let primary_artist_id = primary_song
            .artist_ids
            .as_ref()
            .and_then(|artist_ids| artist_ids.first())
            .cloned();

        let album_art_url = primary_song.album_art_url.clone();

        // Sort songs by track number
        let mut sorted_songs = songs;
        sorted_songs.sort_by(|a, b| {
            let a_track = a.track_number.unwrap_or(0);
            let b_track = b.track_number.unwrap_or(0);
            a_track.cmp(&b_track)
        });

        let album_with_songs = AlbumWithSongs {
            name: album_name,
            artist: primary_artist_name,
            artist_id: primary_artist_id,
            album_art_url,
            song_count: sorted_songs.len() as i32,
            songs: sorted_songs,
        };

        albums_with_songs.push(album_with_songs);
    }

    // Sort albums by name
    albums_with_songs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(albums_with_songs)
}

/// Get artists with their songs
#[tauri::command]
pub async fn get_artists_with_songs(
    server_url: String,
    token: String,
    album_artists_only: bool,
) -> Result<Vec<ArtistWithSongs>, String> {
    // Get all songs
    let songs = get_music_library(server_url.clone(), token.clone()).await?;

    // Get all artists to get artist metadata
    let artists = get_all_artists(server_url, token).await?;

    // Create case-insensitive maps for artist name lookup
    let mut artist_name_to_info: HashMap<String, ArtistInfo> = HashMap::new();
    let mut artist_name_lower_to_original: HashMap<String, String> = HashMap::new();

    for artist in artists {
        let original_name = artist.name.clone();
        let lower_name = original_name.to_lowercase();

        artist_name_to_info.insert(original_name.clone(), artist);
        artist_name_lower_to_original.insert(lower_name, original_name);
    }

    // Group songs by artist
    let mut artist_map: HashMap<String, Vec<MusicItem>> = HashMap::new();

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

    // Convert to ArtistWithSongs
    let mut artists_with_songs: Vec<ArtistWithSongs> = Vec::new();

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

        let artist_with_songs = ArtistWithSongs {
            id: artist_info.map(|a| a.id.clone()).unwrap_or_default(),
            name: artist_name,
            song_count: songs.len() as i32,
            image_url: artist_info.and_then(|a| a.image_url.clone()),
            songs,
        };

        artists_with_songs.push(artist_with_songs);
    }

    // Sort artists by name
    artists_with_songs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(artists_with_songs)
}

/// Get audio stream URL
#[tauri::command]
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

/// Clear the music cache
#[tauri::command]
pub fn clear_music_cache() -> Result<(), String> {
    db::clear_music_cache()
}
