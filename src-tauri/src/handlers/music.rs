//! Music-related command handlers

use crate::cache;
use crate::models::{
    Album, Artist, Song,
    jellyfin::{ClientCapabilities, DeviceProfile, DirectPlayProfile, TranscodingProfile},
};
use crate::services::JellyfinClient;
use crate::utils::pagination;
use std::collections::HashMap;
use tauri::AppHandle;
use tracing::{error, info, warn};

/// Helper function to fetch library from Jellyfin
async fn fetch_library(
    server_url: String,
    token: String,
) -> Result<(Vec<Song>, Vec<Artist>, Vec<Album>), String> {
    let client = JellyfinClient::with_auth(server_url, token);
    let user_id = match crate::handlers::auth::get_saved_credentials().await {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };

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

    Ok((songs, artists, albums))
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
    let all_songs = match cache::get_songs().await.map_err(|e| e.to_string()) {
        Ok(items) if !items.is_empty() => items,
        _ => {
            if let (Some(server_url), Some(token)) = (server_url, token) {
                let (songs, artists, albums) = fetch_library(server_url, token).await?;
                cache::cache_library(&songs)
                    .await
                    .map_err(|e| e.to_string())?;
                cache::cache_artists(&artists)
                    .await
                    .map_err(|e| e.to_string())?;
                cache::cache_albums(&albums)
                    .await
                    .map_err(|e| e.to_string())?;
                songs
            } else {
                return Err(
                    "No cached songs available and no server credentials provided".to_string(),
                );
            }
        }
    };

    let mut filtered_songs = all_songs;

    if let Some(album_id) = album_id {
        filtered_songs.retain(|song| song.album_id.as_ref() == Some(&album_id));
    }

    if let Some(artist_id) = artist_id {
        filtered_songs.retain(|song| {
            song.artist_ids
                .as_ref()
                .is_some_and(|ids| ids.contains(&artist_id))
        });
    }

    pagination::validate_pagination(offset, limit)?;
    let paginated_songs = pagination::apply_pagination(filtered_songs, offset, limit);

    Ok(paginated_songs)
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
    let albums = if let (Some(server_url), Some(token)) = (server_url, token) {
        let client = crate::services::JellyfinClient::with_auth(server_url, token);
        let user_id = crate::handlers::auth::get_saved_credentials()
            .await?
            .map(|creds| creds.user_id)
            .ok_or("No saved credentials found")?;

        client
            .get_albums(&user_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        cache::get_albums().await.map_err(|e| e.to_string())?
    };

    pagination::validate_pagination(offset, limit)?;
    let mut albums = pagination::apply_pagination(albums, offset, limit);

    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

        let mut album_songs: std::collections::HashMap<String, Vec<Song>> =
            std::collections::HashMap::new();
        for song in all_songs {
            if let Some(album_id) = &song.album_id {
                album_songs.entry(album_id.clone()).or_default().push(song);
            }
        }

        for album in &mut albums {
            if let Some(id) = &album.id
                && let Some(songs) = album_songs.get(id)
            {
                album.songs = Some(songs.clone());
            }
        }
    }

    Ok(albums)
}

/// Get single album by ID
#[tauri::command]
#[specta::specta]
pub async fn get_album(album_id: String, include_songs: Option<bool>) -> Result<Album, String> {
    let mut albums = cache::get_albums().await.map_err(|e| e.to_string())?;

    let album_position = albums
        .iter()
        .position(|album| album.id.as_ref() == Some(&album_id));

    let mut album = if let Some(pos) = album_position {
        albums.swap_remove(pos)
    } else {
        return Err(format!("Album with ID '{}' not found", album_id));
    };

    // If album doesn't have provider_ids, try to refresh from server
    if album.provider_ids.is_none()
        && let Ok(Some(creds)) = crate::handlers::auth::get_saved_credentials().await
    {
        let client = JellyfinClient::with_auth(creds.server_url, creds.token);
        match client.get_albums(&creds.user_id).await {
            Ok(fresh_albums) => {
                // Update cache with fresh data
                if let Err(e) = cache::cache_albums(&fresh_albums).await {
                    warn!("Failed to update album cache: {}", e);
                }

                // Find the fresh album
                if let Some(fresh_album) = fresh_albums
                    .into_iter()
                    .find(|a| a.id.as_ref() == Some(&album_id))
                {
                    album = fresh_album;
                    println!(
                        "Refreshed album data from server - now has provider_ids: {:?}",
                        album.provider_ids
                    );
                }
            }
            Err(e) => {
                warn!("Failed to refresh album data from server: {}", e);
            }
        }
    }

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
    let user_id = match crate::handlers::auth::get_saved_credentials().await {
        Ok(Some(creds)) => creds.user_id,
        _ => return Err("No saved credentials found".to_string()),
    };

    client
        .get_recently_played(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get instant mix (similar songs) for a given item
#[tauri::command]
#[specta::specta]
pub async fn get_instant_mix(item_id: String) -> Result<Vec<Song>, String> {
    let (server_url, token) = match crate::handlers::auth::get_saved_credentials().await {
        Ok(Some(creds)) => (creds.server_url, creds.token),
        _ => return Err("No saved credentials found".to_string()),
    };

    let client = JellyfinClient::with_auth(server_url, token);
    client
        .get_instant_mix(&item_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get artists with optional filtering and song inclusion
#[tauri::command]
#[specta::specta]
#[allow(clippy::cast_possible_wrap)]
pub async fn get_artists(
    server_url: Option<String>,
    token: Option<String>,
    include_songs: Option<bool>,
    album_artists_only: Option<bool>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Artist>, String> {
    let artists = if let (Some(server_url), Some(token)) = (server_url, token) {
        match crate::cache::get_artists().await.map_err(|e| e.to_string()) {
            Ok(cached_artists) if !cached_artists.is_empty() => cached_artists,
            _ => {
                let client = JellyfinClient::with_auth(server_url, token);
                let user_id = crate::handlers::auth::get_saved_credentials()
                    .await?
                    .map(|creds| creds.user_id)
                    .ok_or("No saved credentials found")?;

                // Fetch both lists and merge them
                let all_artists_fut = client.get_all_artists();
                let user_artists_fut = client.get_all_artists_for_user(&user_id);

                let (all_artists_res, user_artists_res) =
                    tokio::join!(all_artists_fut, user_artists_fut);

                let all_artists = all_artists_res.map_err(|e| e.to_string())?;
                let user_artists = user_artists_res.map_err(|e| e.to_string())?;

                let mut user_artists_map: HashMap<String, Artist> = user_artists
                    .into_iter()
                    .map(|artist| (artist.id.clone(), artist))
                    .collect();

                let mut merged_artists = Vec::new();
                for artist in all_artists {
                    if let Some(user_artist) = user_artists_map.remove(&artist.id) {
                        merged_artists.push(user_artist);
                    } else {
                        merged_artists.push(artist);
                    }
                }

                // Add any remaining artists from the user-specific list
                merged_artists.extend(user_artists_map.into_values());

                if let Err(e) = crate::cache::cache_artists(&merged_artists).await {
                    warn!("Failed to cache artists: {}", e);
                }

                merged_artists
            }
        }
    } else {
        crate::cache::get_artists()
            .await
            .map_err(|_| "No credentials provided and cache is unavailable".to_string())?
    };

    let mut result_artists = artists;

    if album_artists_only.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;
        let album_artist_ids: std::collections::HashSet<String> = all_songs
            .iter()
            .filter_map(|song| song.album_artists.as_ref())
            .flat_map(|artists| artists.iter().map(|a| a.id.clone()))
            .collect();

        result_artists.retain(|artist| album_artist_ids.contains(&artist.id));
    }

    pagination::validate_pagination(offset, limit)?;
    let mut result_artists = pagination::apply_pagination(result_artists, offset, limit);

    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

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
                artist_map
                    .entry("Unknown Artist".to_string())
                    .or_default()
                    .push(song.clone());
            }
        }

        for artist in &mut result_artists {
            if let Some(songs) = artist_map.get(&artist.name) {
                artist.song_count = Some(songs.len() as i64);
                artist.songs = Some(songs.clone());
            } else {
                artist.song_count = Some(0);
                artist.songs = Some(Vec::new());
            }
        }
    }

    result_artists.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result_artists)
}

/// Get single artist by ID
#[tauri::command]
#[specta::specta]
#[allow(clippy::cast_possible_wrap)]
pub async fn get_artist(
    artist_id: String,
    include_songs: Option<bool>,
    album_artists_only: Option<bool>,
) -> Result<Artist, String> {
    let mut artist_opt: Option<Artist> =
        match crate::cache::get_artists().await.map_err(|e| e.to_string()) {
            Ok(artists) => artists.into_iter().find(|a| a.id == artist_id),
            Err(e) => {
                warn!("Failed to read cached artists: {}", e);
                None
            }
        };

    if artist_opt.is_none() {
        if let Ok(Some(creds)) = crate::handlers::auth::get_saved_credentials().await {
            let client = JellyfinClient::with_auth(creds.server_url, creds.token);
            match client.get_artist_details(&creds.user_id, &artist_id).await {
                Ok(fresh) => {
                    match crate::cache::get_artists().await.map_err(|e| e.to_string()) {
                        Ok(mut current) => {
                            if !current.iter().any(|a| a.id == fresh.id) {
                                current.push(fresh.clone());
                                if let Err(e) = crate::cache::cache_artists(&current).await {
                                    warn!("Failed to cache fetched artist {}: {}", fresh.id, e);
                                }
                            }
                        }
                        Err(e) => warn!("Failed to load artists for cache update: {}", e),
                    }
                    artist_opt = Some(fresh);
                }
                Err(e) => return Err(e.to_string()),
            }
        } else {
            return Err(format!("Artist with ID '{}' not found", artist_id));
        }
    }

    let mut artist = artist_opt.expect("artist should be set by now");

    if artist.overview.is_none()
        && let Ok(Some(creds)) = crate::handlers::auth::get_saved_credentials().await
    {
        let client = JellyfinClient::with_auth(creds.server_url, creds.token);
        if let Ok(fresh) = client.get_artist_details(&creds.user_id, &artist_id).await {
            artist.overview = fresh.overview;
            artist.image_url = fresh.image_url.or(artist.image_url);
            artist.provider_ids = fresh.provider_ids.or(artist.provider_ids);
            artist.community_rating = fresh.community_rating.or(artist.community_rating);
            artist.name = fresh.name;
        }
    }

    if include_songs.unwrap_or(false) {
        let all_songs = cache::get_songs().await.map_err(|e| e.to_string())?;

        let artist_songs: Vec<Song> = if album_artists_only.unwrap_or(false) {
            let mut is_album_artist = false;
            for song in &all_songs {
                if let Some(artists) = &song.album_artists
                    && artists.iter().any(|a| a.id == artist_id)
                {
                    is_album_artist = true;
                    break;
                }
            }

            if is_album_artist {
                all_songs
                    .into_iter()
                    .filter(|song| {
                        song.artist_ids
                            .as_ref()
                            .is_some_and(|ids| ids.contains(&artist_id))
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
                        .is_some_and(|ids| ids.contains(&artist_id))
                })
                .collect()
        };

        artist.song_count = Some(artist_songs.len() as i64);
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

    // Update the cached song's favorite status
    cache::update_song_favorite_status(&item_id, is_favorite)
        .await
        .map_err(|e| format!("Failed to update cache: {}", e))?;

    Ok(is_favorite)
}

/// Sync music library - update existing data without clearing cache
#[tauri::command]
#[specta::specta]
pub async fn sync_library(server_url: String, token: String) -> Result<(), String> {
    info!("Starting library sync...");
    let (songs, artists, albums) = fetch_library(server_url, token).await?;
    cache::sync_library(&songs, &artists, &albums)
        .await
        .map_err(|e| e.to_string())?;
    info!("Library sync completed successfully.");
    Ok(())
}

/// Clear the music cache, then re-fetch and cache the library
#[tauri::command]
#[specta::specta]
pub async fn clear_cache(app: AppHandle, server_url: String, token: String) -> Result<(), String> {
    info!("Clearing all caches...");

    // Clear music library cache
    info!("Clearing music cache...");
    cache::clear_cache().await.map_err(|e| e.to_string())?;

    // Clear image cache
    info!("Clearing image cache...");
    if let Err(e) = crate::handlers::images::clear_image_cache(app).await {
        warn!("Failed to clear image cache: {}", e);
    }

    // Re-fetch and cache library
    let (songs, artists, albums) = fetch_library(server_url, token).await?;
    info!("Re-caching library...");
    cache::cache_library(&songs)
        .await
        .map_err(|e| e.to_string())?;
    cache::cache_artists(&artists)
        .await
        .map_err(|e| e.to_string())?;
    cache::cache_albums(&albums)
        .await
        .map_err(|e| e.to_string())?;
    info!("Cache cleared and library re-cached successfully.");
    Ok(())
}

/// Register client capabilities with Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn register_client_capabilities(server_url: String, token: String) -> Result<(), String> {
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
            id: None,
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
    song_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let song = get_song(song_id).await?;
    crate::services::MusicBrainzService::get_song_share_urls(&song).await
}

/// Get share URLs for an album
#[tauri::command]
#[specta::specta]
pub async fn get_album_share_urls(
    album_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let album = get_album(album_id, None).await?;
    crate::services::MusicBrainzService::get_album_share_urls(&album).await
}

/// Get share URLs for an artist
#[tauri::command]
#[specta::specta]
pub async fn get_artist_share_urls(
    artist_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let artist = get_artist(artist_id, None, None).await?;
    crate::services::MusicBrainzService::get_artist_share_urls(&artist).await
}
