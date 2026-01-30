//! Music-related command handlers

use aurelia_core::db;
use aurelia_core::models::library::{HomeViewData, LibraryData};
use aurelia_core::models::{
    Album, Artist, Song,
    jellyfin::{ClientCapabilities, DeviceProfile, DirectPlayProfile, TranscodingProfile},
};
use aurelia_core::services::JellyfinClient;
use aurelia_core::state::AppState;
use rand::seq::SliceRandom;

use tauri::{AppHandle, State};
use tracing::{error, info, warn};

#[tauri::command]
#[specta::specta]
pub async fn get_library(app_state: State<'_, AppState>) -> Result<LibraryData, String> {
    info!("get_library command called");
    let songs = app_state.songs.lock().unwrap().clone();
    info!("get_library: got {} songs from state", songs.len());

    // Derive albums from songs (hybrid lazy-load approach)
    let mut album_map: std::collections::HashMap<String, Vec<Song>> =
        std::collections::HashMap::new();
    for song in &songs {
        if let Some(album_id) = &song.album_id {
            album_map
                .entry(album_id.clone())
                .or_default()
                .push(song.clone());
        }
    }

    let albums: Vec<Album> = album_map
        .iter()
        .filter_map(|(album_id, album_songs)| {
            let first_song = album_songs
                .iter()
                .max_by_key(|s| s.date_created.as_deref().unwrap_or(""))?;
            Some(Album {
                id: Some(album_id.clone()),
                name: first_song
                    .album
                    .clone()
                    .unwrap_or_else(|| "Unknown Album".to_string()),
                artist: first_song
                    .artists
                    .as_ref()
                    .and_then(|a| a.first())
                    .cloned()
                    .unwrap_or_else(|| "Unknown Artist".to_string()),
                artist_id: first_song
                    .artist_ids
                    .as_ref()
                    .and_then(|ids| ids.first())
                    .cloned(),
                album_art_url: first_song.album_art_url.clone(),
                song_count: album_songs.len() as i64,
                songs: None, // Don't include songs in library response to reduce payload size
                image_tags: None,
                provider_ids: None,
                date_created: first_song.date_created.clone(),
                date_modified: None,
            })
        })
        .collect();

    // Derive artists from songs
    let mut artist_map: std::collections::HashMap<String, (String, Option<String>)> =
        std::collections::HashMap::new();
    for song in &songs {
        if let Some(artist_ids) = &song.artist_ids {
            let artist_names = song.artists.as_ref();
            for (i, artist_id) in artist_ids.iter().enumerate() {
                if !artist_map.contains_key(artist_id) {
                    let name = artist_names
                        .and_then(|names| names.get(i))
                        .cloned()
                        .unwrap_or_else(|| "Unknown Artist".to_string());
                    let image_url = if i == 0 {
                        song.album_art_url.clone()
                    } else {
                        None
                    };
                    artist_map.insert(artist_id.clone(), (name, image_url));
                }
            }
        }
    }

    let artists: Vec<Artist> = artist_map
        .into_iter()
        .map(|(id, (name, image_url))| Artist {
            id: id.clone(),
            name,
            image_url,
            image_tags: None,
            overview: None,
            provider_ids: None,
            community_rating: None,
            song_count: None,
            date_modified: None,
            songs: None,
        })
        .collect();

    info!(
        "get_library: derived {} albums and {} artists from songs",
        albums.len(),
        artists.len()
    );

    Ok(LibraryData {
        songs,
        artists,
        albums,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_home_view_data(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
) -> Result<HomeViewData, String> {
    info!("get_home_view_data command called");
    let all_songs = app_state.songs.lock().unwrap().clone();
    info!("get_home_view_data: working with {} songs", all_songs.len());

    // If songs are empty, library hasn't been loaded yet
    if all_songs.is_empty() {
        warn!("get_home_view_data: songs list is empty, library may not be loaded yet");
        return Err(
            "Library not loaded yet. Please wait for library to load before requesting home data."
                .to_string(),
        );
    }

    // Derive albums from songs (like Android does) - group by album_id
    let mut album_map: std::collections::HashMap<String, Vec<Song>> =
        std::collections::HashMap::new();
    for song in &all_songs {
        if let Some(album_id) = &song.album_id {
            album_map
                .entry(album_id.clone())
                .or_default()
                .push(song.clone());
        }
    }

    // Build Album objects from song data
    let derived_albums: Vec<Album> = album_map
        .iter()
        .filter_map(|(album_id, songs)| {
            let first_song = songs
                .iter()
                .max_by_key(|s| s.date_created.as_deref().unwrap_or(""))?;
            Some(Album {
                id: Some(album_id.clone()),
                name: first_song
                    .album
                    .clone()
                    .unwrap_or_else(|| "Unknown Album".to_string()),
                artist: first_song
                    .artists
                    .as_ref()
                    .and_then(|a| a.first())
                    .cloned()
                    .unwrap_or_else(|| "Unknown Artist".to_string()),
                artist_id: first_song
                    .artist_ids
                    .as_ref()
                    .and_then(|ids| ids.first())
                    .cloned(),
                album_art_url: first_song.album_art_url.clone(),
                song_count: songs.len() as i64,
                songs: Some(songs.clone()),
                image_tags: None,
                provider_ids: None,
                date_created: first_song.date_created.clone(),
                date_modified: None,
            })
        })
        .collect();

    info!(
        "get_home_view_data: derived {} albums from songs",
        derived_albums.len()
    );

    // Recently added - sort by date_created
    let mut recently_added = derived_albums.clone();
    recently_added.sort_by(|a, b| {
        let date_a = a
            .date_created
            .as_ref()
            .map(|d| {
                d.parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let date_b = b
            .date_created
            .as_ref()
            .map(|d| {
                d.parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        date_b.cmp(&date_a)
    });

    let mut random_albums = derived_albums.clone();
    random_albums.shuffle(&mut rand::rng());

    let mut featured_albums = derived_albums;
    featured_albums.shuffle(&mut rand::rng());

    let (server_url, token, user_id) = match super::auth::get_credentials_cached(&app).await {
        Ok(Some(creds)) => (creds.server_url, creds.token, creds.user_id),
        _ => return Err("No saved credentials found".to_string()),
    };
    let recently_played = get_recently_played(server_url, token, user_id).await?;

    let result = HomeViewData {
        recently_added: recently_added.into_iter().take(10).collect(),
        random_albums: random_albums.into_iter().take(10).collect(),
        featured_albums: featured_albums.into_iter().take(10).collect(),
        recently_played,
    };
    info!(
        "get_home_view_data: returning {} recently_added, {} random_albums, {} featured_albums, {} recently_played",
        result.recently_added.len(),
        result.random_albums.len(),
        result.featured_albums.len(),
        result.recently_played.len()
    );
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_song(app_state: State<'_, AppState>, song_id: String) -> Result<Song, String> {
    let songs = app_state.songs.lock().unwrap().clone();

    songs
        .into_iter()
        .find(|song| song.id == song_id)
        .ok_or_else(|| format!("Song with ID '{}' not found", song_id))
}

#[tauri::command]
#[specta::specta]
pub async fn get_album(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    album_id: String,
    include_songs: Option<bool>,
) -> Result<Album, String> {
    // Try app state first (fast path)
    let cached_in_state = {
        let albums = app_state.albums.lock().unwrap();
        albums
            .iter()
            .find(|a| a.id.as_ref() == Some(&album_id))
            .cloned()
    };

    let mut album = if let Some(album) = cached_in_state {
        album
    } else {
        // Try database cache
        if let Ok(Some(album)) = db::albums::get_by_id(&album_id) {
            // Update app state with cached album
            app_state.albums.lock().unwrap().push(album.clone());
            album
        } else {
            // Fetch from server and cache
            info!("Lazy-loading album {} from server", album_id);
            let (server_url, token, user_id) = match super::auth::get_credentials_cached(&app).await
            {
                Ok(Some(creds)) => (creds.server_url, creds.token, creds.user_id),
                _ => return Err("No saved credentials found".to_string()),
            };

            let client = JellyfinClient::with_auth(server_url, token);
            let fetched_album = client
                .get_album_details(&user_id, &album_id)
                .await
                .map_err(|e| format!("Failed to fetch album: {}", e))?;

            // Cache in database
            db::albums::cache(&fetched_album).map_err(|e| e.to_string())?;

            // Update app state
            app_state.albums.lock().unwrap().push(fetched_album.clone());

            fetched_album
        }
    };

    if include_songs.unwrap_or(false) {
        // Use server-side filtering via AlbumIds query parameter
        // Includes workaround for Jellyfin bug where it also matches album names
        let (server_url, token, user_id) = match super::auth::get_credentials_cached(&app).await {
            Ok(Some(creds)) => (creds.server_url, creds.token, creds.user_id),
            _ => return Err("No saved credentials found".to_string()),
        };

        let client = JellyfinClient::with_auth(server_url, token);
        let songs = client
            .get_songs_for_album(&user_id, &album_id)
            .await
            .map_err(|e| format!("Failed to fetch songs for album: {}", e))?;
        album.songs = Some(songs);
    }

    Ok(album)
}

#[tauri::command]
#[specta::specta]
pub async fn get_artist(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    artist_id: String,
    include_songs: Option<bool>,
) -> Result<Artist, String> {
    // Try app state first (fast path)
    let cached_in_state = {
        let artists = app_state.artists.lock().unwrap();
        artists.iter().find(|a| a.id == artist_id).cloned()
    };

    let mut artist = if let Some(artist) = cached_in_state {
        artist
    } else {
        // Try database cache
        if let Ok(Some(artist)) = db::artists::get_by_id(&artist_id) {
            // Update app state with cached artist
            app_state.artists.lock().unwrap().push(artist.clone());
            artist
        } else {
            // Fetch from server and cache
            info!("Lazy-loading artist {} from server", artist_id);
            let (server_url, token, user_id) = match super::auth::get_credentials_cached(&app).await
            {
                Ok(Some(creds)) => (creds.server_url, creds.token, creds.user_id),
                _ => return Err("No saved credentials found".to_string()),
            };

            let client = JellyfinClient::with_auth(server_url, token);
            let fetched_artist = client
                .get_artist_details(&user_id, &artist_id)
                .await
                .map_err(|e| format!("Failed to fetch artist: {}", e))?;

            // Cache in database
            db::artists::cache(&fetched_artist).map_err(|e| e.to_string())?;

            // Update app state
            app_state
                .artists
                .lock()
                .unwrap()
                .push(fetched_artist.clone());

            fetched_artist
        }
    };

    if include_songs.unwrap_or(false) {
        // Use server-side filtering via AlbumArtistIds query parameter
        let (server_url, token, user_id) = match super::auth::get_credentials_cached(&app).await {
            Ok(Some(creds)) => (creds.server_url, creds.token, creds.user_id),
            _ => return Err("No saved credentials found".to_string()),
        };

        let client = JellyfinClient::with_auth(server_url, token);
        let songs = client
            .get_songs_for_album_artist(&user_id, &artist_id)
            .await
            .map_err(|e| format!("Failed to fetch songs for artist: {}", e))?;
        artist.songs = Some(songs);
    }

    Ok(artist)
}

#[tauri::command]
#[specta::specta]
pub async fn get_related_artists(
    app_state: State<'_, AppState>,
    artist_id: String,
) -> Result<Vec<Artist>, String> {
    info!(
        "get_related_artists command called for artist {}",
        artist_id
    );
    let all_artists = app_state.artists.lock().unwrap().clone();
    let all_songs = app_state.songs.lock().unwrap().clone();

    let current_artist = all_artists
        .iter()
        .find(|a| a.id == artist_id)
        .ok_or_else(|| "Artist not found".to_string())?;

    const COLLABORATION_SCORE: i32 = 10;
    const SHARED_GENRE_SCORE: i32 = 5;
    const SHARED_ALBUM_SCORE: i32 = 2;

    let current_artist_songs: Vec<&Song> = all_songs
        .iter()
        .filter(|s| {
            s.artists
                .as_ref()
                .is_some_and(|a| a.contains(&current_artist.name))
        })
        .collect();

    let current_artist_genres: std::collections::HashSet<&String> = current_artist_songs
        .iter()
        .flat_map(|s| {
            s.genres
                .as_ref()
                .map_or_else(Vec::new, |g| g.iter().collect())
        })
        .collect();

    let current_artist_albums: std::collections::HashSet<&String> = current_artist_songs
        .iter()
        .filter_map(|s| s.album.as_ref())
        .collect();

    let mut artist_scores: std::collections::HashMap<String, i32> =
        std::collections::HashMap::new();

    for other_artist in &all_artists {
        if other_artist.id == current_artist.id {
            continue;
        }

        let mut score = 0;
        let other_artist_songs: Vec<&Song> = all_songs
            .iter()
            .filter(|s| {
                s.artists
                    .as_ref()
                    .is_some_and(|a| a.contains(&other_artist.name))
            })
            .collect();

        if other_artist_songs.is_empty() {
            continue;
        }

        let collaborations = current_artist_songs
            .iter()
            .filter(|s| {
                s.artists
                    .as_ref()
                    .is_some_and(|a| a.contains(&other_artist.name))
            })
            .count();
        score += collaborations as i32 * COLLABORATION_SCORE;

        let other_artist_genres: std::collections::HashSet<&String> = other_artist_songs
            .iter()
            .flat_map(|s| {
                s.genres
                    .as_ref()
                    .map_or_else(Vec::new, |g| g.iter().collect())
            })
            .collect();

        for genre in &other_artist_genres {
            if current_artist_genres.contains(genre) {
                score += SHARED_GENRE_SCORE;
            }
        }

        let other_artist_albums: std::collections::HashSet<&String> = other_artist_songs
            .iter()
            .filter_map(|s| s.album.as_ref())
            .collect();

        for album in &other_artist_albums {
            if current_artist_albums.contains(album) && collaborations == 0 {
                score += SHARED_ALBUM_SCORE;
            }
        }

        if score > 0 {
            artist_scores.insert(other_artist.id.clone(), score);
        }
    }

    let mut sorted_artists: Vec<_> = artist_scores.into_iter().collect();
    sorted_artists.sort_by(|a, b| b.1.cmp(&a.1));

    let result: Vec<Artist> = sorted_artists
        .iter()
        .take(6)
        .filter_map(|(artist_id, _)| all_artists.iter().find(|a| a.id == *artist_id).cloned())
        .collect();

    info!(
        "get_related_artists: found {} related artists",
        result.len()
    );

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_recently_played(
    server_url: String,
    token: String,
    user_id: String,
) -> Result<Vec<Song>, String> {
    let client = JellyfinClient::with_auth(server_url, token);

    client
        .get_recently_played(&user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_instant_mix(app: tauri::AppHandle, item_id: String) -> Result<Vec<Song>, String> {
    let (server_url, token) = match super::auth::get_credentials_cached(&app).await {
        Ok(Some(creds)) => (creds.server_url, creds.token),
        _ => return Err("No saved credentials found".to_string()),
    };

    let client = JellyfinClient::with_auth(server_url, token);
    client
        .get_instant_mix(&item_id)
        .await
        .map_err(|e| e.to_string())
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
    app_state: State<'_, AppState>,
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

    // Update the database
    db::songs::update_favorite_status(&item_id, is_favorite)
        .map_err(|e| format!("Failed to update database: {}", e))?;

    // Update the in-memory state
    let mut songs = app_state.songs.lock().unwrap();
    if let Some(song) = songs.iter_mut().find(|s| s.id == item_id) {
        song.is_favorite = Some(is_favorite);
    }

    Ok(is_favorite)
}

/// Sync music library - only syncs songs for fast startup
/// Artists/albums are fetched on-demand when viewing detail pages
#[tauri::command]
#[specta::specta]
pub async fn sync_library(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    server_url: String,
    token: String,
) -> Result<(), String> {
    info!("Starting library sync (songs only)...");

    // Get user_id from saved credentials
    let user_id = match super::auth::get_credentials_cached(&app).await {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };

    let client = JellyfinClient::with_auth(server_url.clone(), token.clone());

    // Only fetch songs - artists/albums are lazy-loaded on demand
    let songs = client
        .get_music_library(&user_id)
        .await
        .map_err(|e| e.to_string())?;

    // Use incremental sync for songs only
    info!("Syncing {} songs", songs.len());
    let was_full_sync = db::sync_songs_only(&songs).map_err(|e| e.to_string())?;

    if was_full_sync {
        info!("Performed full songs sync");
    } else {
        info!("Performed incremental songs sync");
    }

    // Update app state with songs
    // Note: artists/albums in app state will be populated lazily when needed
    let song_count = songs.len() as u32;
    *app_state.songs.lock().unwrap() = songs;

    // Save sync state with timestamp
    let sync_state = aurelia_core::domain::SyncState {
        last_sync_time: chrono::Utc::now().to_rfc3339(),
        last_full_sync_time: if was_full_sync {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        },
        last_sync_version: None,
        song_count,
        artist_count: 0,
        album_count: 0,
    };

    let state_json = serde_json::to_string(&sync_state).map_err(|e| e.to_string())?;
    use tauri::Manager;
    let app_data_path = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    let sync_state_path = app_data_path.join("sync_state.json");
    std::fs::write(&sync_state_path, state_json).map_err(|e| e.to_string())?;

    info!("Library sync completed successfully.");
    Ok(())
}

/// Clear the music cache, then re-fetch and cache the library
#[tauri::command]
#[specta::specta]
pub async fn clear_cache(
    app: AppHandle,
    app_state: State<'_, AppState>,
    server_url: String,
    token: String,
) -> Result<(), String> {
    info!("Clearing all caches...");

    // Clear database
    db::songs::clear().map_err(|e| e.to_string())?;
    db::artists::clear().map_err(|e| e.to_string())?;
    db::albums::clear().map_err(|e| e.to_string())?;

    // Clear app state
    *app_state.songs.lock().unwrap() = vec![];
    *app_state.artists.lock().unwrap() = vec![];
    *app_state.albums.lock().unwrap() = vec![];

    info!("In-memory cache cleared.");

    // Clear image cache
    info!("Clearing image cache...");
    if let Err(e) = super::images::clear_image_cache(app.clone()).await {
        warn!("Failed to clear image cache: {}", e);
    }

    // Re-fetch and cache library
    sync_library(app.clone(), app_state, server_url, token).await?;

    info!("Cache cleared and library re-cached successfully.");
    Ok(())
}

/// Register client capabilities with Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn register_client_capabilities(
    server_url: String,
    token: String,
    device_id: String,
) -> Result<(), String> {
    let client = JellyfinClient::with_auth(server_url, token);

    let capabilities = ClientCapabilities {
        playable_media_types: vec!["Audio".to_string()],
        supported_commands: vec![
            "Play".to_string(),
            "SetRepeatMode".to_string(),
            "SetShuffleQueue".to_string(),
            "VolumeUp".to_string(),
            "VolumeDown".to_string(),
            "Mute".to_string(),
            "Unmute".to_string(),
            "ToggleMute".to_string(),
            "SetVolume".to_string(),
        ],
        supports_media_control: true,
        supports_persistent_identifier: true,
        device_profile: DeviceProfile {
            name: Some("Tauri Music Player".to_string()),
            id: Some(device_id),
            max_streaming_bitrate: Some(320000),
            max_static_bitrate: Some(320000),
            music_streaming_transcoding_bitrate: Some(128000),
            max_static_music_bitrate: Some(320000),
            direct_play_profiles: vec![
                DirectPlayProfile {
                    container: "mp3".to_string(),
                    audio_codec: Some("mp3".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
                DirectPlayProfile {
                    container: "flac".to_string(),
                    audio_codec: Some("flac".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
                DirectPlayProfile {
                    container: "aac".to_string(),
                    audio_codec: Some("aac".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
                DirectPlayProfile {
                    container: "ogg".to_string(),
                    audio_codec: Some("vorbis".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
                DirectPlayProfile {
                    container: "m4a".to_string(),
                    audio_codec: Some("aac".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
            ],
            transcoding_profiles: vec![TranscodingProfile {
                container: "aac".to_string(),
                profile_type: "Audio".to_string(),
                video_codec: None,
                audio_codec: Some("aac".to_string()),
                protocol: "http".to_string(),
                estimate_content_length: Some(false),
                enable_mpegts_m2_ts_mode: Some(false),
                transcode_seek_info: Some("Auto".to_string()),
                copy_timestamps: Some(false),
                context: Some("Streaming".to_string()),
                enable_subtitles_in_manifest: Some(false),
                max_audio_channels: Some("2".to_string()),
                min_segments: Some(0),
                segment_length: Some(0),
                break_on_non_key_frames: Some(false),
                conditions: vec![],
                enable_audio_vbr_encoding: Some(true),
            }],
            container_profiles: vec![],
            codec_profiles: vec![],
            subtitle_profiles: vec![],
        },
        app_store_url: None,
        icon_url: None,
    };

    client
        .register_capabilities(&capabilities)
        .await
        .map_err(|e| {
            error!("Failed to register client capabilities: {}", e);
            e.to_string()
        })?;

    info!("Successfully registered client capabilities");
    Ok(())
}

/// Report playback start to Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn report_playback_start(
    server_url: String,
    token: String,
    item_id: String,
    position_ticks: Option<f64>,
) -> Result<(), String> {
    let client = JellyfinClient::with_auth(server_url, token);

    #[allow(clippy::cast_possible_truncation)]
    let position_ticks_i64 = position_ticks.map(|p| p.floor() as i64);

    client
        .report_playback_start(&item_id, position_ticks_i64)
        .await
        .map_err(|e| {
            error!("Failed to report playback start: {}", e);
            e.to_string()
        })?;

    info!("Successfully reported playback start");
    Ok(())
}

/// Report playback progress to Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn report_playback_progress(
    server_url: String,
    token: String,
    item_id: String,
    position_ticks: Option<f64>,
    event_name: Option<String>,
    is_paused: Option<bool>,
) -> Result<(), String> {
    let client = JellyfinClient::with_auth(server_url, token);

    #[allow(clippy::cast_possible_truncation)]
    let position_ticks_i64 = position_ticks.map(|ticks| ticks as i64);

    client
        .report_playback_progress(
            &item_id,
            position_ticks_i64,
            event_name.as_deref(),
            is_paused,
        )
        .await
        .map_err(|e| {
            error!("Failed to report playback progress: {}", e);
            e.to_string()
        })?;

    Ok(())
}

/// Report playback stop to Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn report_playback_stop(
    server_url: String,
    token: String,
    item_id: String,
    position_ticks: Option<f64>,
) -> Result<(), String> {
    let client = JellyfinClient::with_auth(server_url, token);

    #[allow(clippy::cast_possible_truncation)]
    let position_ticks_i64 = position_ticks.map(|ticks| ticks as i64);

    client
        .report_playback_stop(&item_id, position_ticks_i64)
        .await
        .map_err(|e| {
            error!("Failed to report playback stop: {}", e);
            e.to_string()
        })?;

    info!("Successfully reported playback stop");
    Ok(())
}

/// Mark item as played in Jellyfin
#[tauri::command]
#[specta::specta]
pub async fn mark_item_played(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
) -> Result<(), String> {
    let client = JellyfinClient::with_auth(server_url, token);

    client
        .mark_item_played(&user_id, &item_id)
        .await
        .map_err(|e| {
            error!("Failed to mark item as played: {}", e);
            e.to_string()
        })?;

    info!("Successfully marked item {} as played", item_id);
    Ok(())
}

/// Get share URLs for a song
#[tauri::command]
#[specta::specta]
pub async fn get_song_share_urls(
    app_state: State<'_, AppState>,
    song_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let song = get_song(app_state, song_id).await?;
    aurelia_core::services::MusicBrainzService::get_song_share_urls(&song).await
}

/// Get share URLs for an album
#[tauri::command]
#[specta::specta]
pub async fn get_album_share_urls(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    album_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let album = get_album(app, app_state, album_id, None).await?;
    aurelia_core::services::MusicBrainzService::get_album_share_urls(&album).await
}

/// Get share URLs for an artist
#[tauri::command]
#[specta::specta]
pub async fn get_artist_share_urls(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    artist_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let artist = get_artist(app, app_state, artist_id, None).await?;
    aurelia_core::services::MusicBrainzService::get_artist_share_urls(&artist).await
}

/// Sync state for UI display
#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncStateInfo {
    pub last_sync_time: Option<String>,
    pub song_count: u32,
    pub artist_count: u32,
    pub album_count: u32,
}

/// Get the current sync state for UI display
#[tauri::command]
#[specta::specta]
pub async fn get_sync_state(app: tauri::AppHandle) -> Result<SyncStateInfo, String> {
    use tauri::Manager;

    // Get app data path from tauri
    let app_data_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let sync_state_path = app_data_path.join("sync_state.json");

    if sync_state_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&sync_state_path) {
            if let Ok(parsed) = serde_json::from_str::<aurelia_core::domain::SyncState>(&content) {
                return Ok(SyncStateInfo {
                    last_sync_time: Some(parsed.last_sync_time),
                    song_count: parsed.song_count,
                    artist_count: parsed.artist_count,
                    album_count: parsed.album_count,
                });
            }
        }
    }

    let state = aurelia_core::domain::SyncState::default();
    Ok(SyncStateInfo {
        last_sync_time: None,
        song_count: state.song_count,
        artist_count: state.artist_count,
        album_count: state.album_count,
    })
}
