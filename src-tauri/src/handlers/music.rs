//! Music-related command handlers

use crate::models::library::{HomeViewData, LibraryData};
use crate::models::{
    Album, Artist, Song,
    jellyfin::{ClientCapabilities, DeviceProfile, DirectPlayProfile, TranscodingProfile},
};
use crate::services::JellyfinClient;
use crate::state::AppState;
use rand::seq::SliceRandom;

use std::collections::HashMap;
use tauri::{AppHandle, State};
use tracing::{error, info, warn};

#[tauri::command]
#[specta::specta]
pub async fn get_library(app_state: State<'_, AppState>) -> Result<LibraryData, String> {
    info!("get_library command called");
    let songs = app_state.songs.lock().unwrap().clone();
    let artists = app_state.artists.lock().unwrap().clone();
    let albums = app_state.albums.lock().unwrap().clone();
    info!("get_library: got data from state");

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
    app_state: State<'_, AppState>
) -> Result<HomeViewData, String> {
    info!("get_home_view_data command called");
    let all_albums = app_state.albums.lock().unwrap().clone();
    let all_songs = app_state.songs.lock().unwrap().clone();
    info!("get_home_view_data: working with {} albums and {} songs", all_albums.len(), all_songs.len());
    
    // If albums are empty, library hasn't been loaded yet
    if all_albums.is_empty() {
        warn!("get_home_view_data: albums list is empty, library may not be loaded yet");
        return Err("Library not loaded yet. Please wait for library to load before requesting home data.".to_string());
    }

    // Create album-to-songs mapping
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

    let mut recently_added = all_albums.clone();
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

    let mut random_albums = all_albums.clone();
    random_albums.shuffle(&mut rand::rng());

    let mut featured_albums = all_albums.clone();
    featured_albums.shuffle(&mut rand::rng());

    // Populate songs for featured albums
    for album in &mut featured_albums {
        if let Some(album_id) = &album.id {
            album.songs = album_map.get(album_id).cloned();
        }
    }

    // Populate songs for recently added albums
    for album in &mut recently_added {
        if let Some(album_id) = &album.id {
            album.songs = album_map.get(album_id).cloned();
        }
    }

    // Populate songs for random albums
    for album in &mut random_albums {
        if let Some(album_id) = &album.id {
            album.songs = album_map.get(album_id).cloned();
        }
    }

    let (server_url, token, user_id) = match crate::handlers::auth::get_saved_credentials(app).await {
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
    info!("get_home_view_data: returning {} recently_added, {} random_albums, {} featured_albums, {} recently_played",
        result.recently_added.len(), result.random_albums.len(), result.featured_albums.len(), result.recently_played.len());
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
    app_state: State<'_, AppState>,
    album_id: String,
    include_songs: Option<bool>,
) -> Result<Album, String> {
    let albums = app_state.albums.lock().unwrap().clone();
    let mut album = albums
        .into_iter()
        .find(|album| album.id.as_ref() == Some(&album_id))
        .ok_or_else(|| format!("Album with ID '{}' not found", album_id))?;

    if include_songs.unwrap_or(false) {
        let all_songs = app_state.songs.lock().unwrap().clone();
        let album_songs: Vec<Song> = all_songs
            .into_iter()
            .filter(|song| song.album_id.as_ref() == Some(&album_id))
            .collect();
        album.songs = Some(album_songs);
    }

    Ok(album)
}

#[tauri::command]
#[specta::specta]
pub async fn get_artist(
    app_state: State<'_, AppState>,
    artist_id: String,
    include_songs: Option<bool>,
) -> Result<Artist, String> {
    let artists = app_state.artists.lock().unwrap().clone();
    let mut artist = artists
        .into_iter()
        .find(|artist| artist.id == artist_id)
        .ok_or_else(|| format!("Artist with ID '{}' not found", artist_id))?;

    if include_songs.unwrap_or(false) {
        let all_songs = app_state.songs.lock().unwrap().clone();
        let artist_songs: Vec<Song> = all_songs
            .into_iter()
            .filter(|song| {
                song.artist_ids
                    .as_ref()
                    .is_some_and(|ids| ids.contains(&artist_id))
            })
            .collect();
        artist.songs = Some(artist_songs);
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
pub async fn get_recently_played(server_url: String, token: String, user_id: String) -> Result<Vec<Song>, String> {
    let client = JellyfinClient::with_auth(server_url, token);

    client
        .get_recently_played(&user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_instant_mix(app: tauri::AppHandle, item_id: String) -> Result<Vec<Song>, String> {
    let (server_url, token) = match crate::handlers::auth::get_saved_credentials(app).await {
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
    crate::database::songs::update_favorite_status(&item_id, is_favorite)
        .map_err(|e| format!("Failed to update database: {}", e))?;

    // Update the in-memory state
    let mut songs = app_state.songs.lock().unwrap();
    if let Some(song) = songs.iter_mut().find(|s| s.id == item_id) {
        song.is_favorite = Some(is_favorite);
    }

    Ok(is_favorite)
}

/// Sync music library - update existing data without clearing cache
#[tauri::command]
#[specta::specta]
pub async fn sync_library(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    server_url: String,
    token: String,
) -> Result<(), String> {
    info!("Starting library sync...");

    // Get user_id from saved credentials
    let user_id = match crate::handlers::auth::get_saved_credentials(app).await {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };

    let client = JellyfinClient::with_auth(server_url.clone(), token.clone());

    // Fetch in parallel
    let songs_fut = client.get_music_library(&user_id);
    let all_artists_fut = client.get_all_artists();
    let user_artists_fut = client.get_all_artists_for_user(&user_id);
    let albums_fut = client.get_albums(&user_id);

    let (songs_res, all_artists_res, user_artists_res, albums_res) =
        tokio::join!(songs_fut, all_artists_fut, user_artists_fut, albums_fut);

    let songs = songs_res.map_err(|e| e.to_string())?;
    let all_artists = all_artists_res.map_err(|e| e.to_string())?;
    let user_artists = user_artists_res.map_err(|e| e.to_string())?;
    let albums = albums_res.map_err(|e| e.to_string())?;

    // Merge artist lists
    let mut user_artists_map: HashMap<String, Artist> = user_artists
        .into_iter()
        .map(|artist| (artist.id.clone(), artist))
        .collect();

    let mut artists = Vec::new();
    for artist in all_artists {
        if let Some(user_artist) = user_artists_map.remove(&artist.id) {
            artists.push(user_artist);
        } else {
            artists.push(artist);
        }
    }
    artists.extend(user_artists_map.into_values());

    // Update database
    info!("Syncing {} songs to database", songs.len());
    crate::database::songs::sync(&songs).map_err(|e| e.to_string())?;
    info!("Syncing {} artists to database", artists.len());
    crate::database::artists::sync(&artists).map_err(|e| e.to_string())?;
    info!("Syncing {} albums to database", albums.len());
    crate::database::albums::sync(&albums).map_err(|e| e.to_string())?;

    // Update app state
    *app_state.songs.lock().unwrap() = songs;
    *app_state.artists.lock().unwrap() = artists;
    *app_state.albums.lock().unwrap() = albums;

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
    crate::database::songs::clear().map_err(|e| e.to_string())?;
    crate::database::artists::clear().map_err(|e| e.to_string())?;
    crate::database::albums::clear().map_err(|e| e.to_string())?;

    // Clear app state
    *app_state.songs.lock().unwrap() = vec![];
    *app_state.artists.lock().unwrap() = vec![];
    *app_state.albums.lock().unwrap() = vec![];

    info!("In-memory cache cleared.");

    // Clear image cache
    info!("Clearing image cache...");
    if let Err(e) = crate::handlers::images::clear_image_cache(app.clone()).await {
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
pub async fn register_client_capabilities(server_url: String, token: String, device_id: String) -> Result<(), String> {
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
    crate::services::MusicBrainzService::get_song_share_urls(&song).await
}

/// Get share URLs for an album
#[tauri::command]
#[specta::specta]
pub async fn get_album_share_urls(
    app_state: State<'_, AppState>,
    album_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let album = get_album(app_state, album_id, None).await?;
    crate::services::MusicBrainzService::get_album_share_urls(&album).await
}

/// Get share URLs for an artist
#[tauri::command]
#[specta::specta]
pub async fn get_artist_share_urls(
    app_state: State<'_, AppState>,
    artist_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let artist = get_artist(app_state, artist_id, None).await?;
    crate::services::MusicBrainzService::get_artist_share_urls(&artist).await
}
