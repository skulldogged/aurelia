//! Tauri API implementation
//!
//! This module provides the Tauri-specific implementation of the Api trait.

use crate::{
    Album, Api, ApiResult, AppError, Artist, Credentials, HomeViewData, LastFmCredentials,
    LibraryData, NowPlayingPayload, Playlist, PlaylistCreateData, PlaylistUpdateData, RpcActivity,
    Song, SyncStateInfo,
};
use aurelia_core::audio::AudioState;
use aurelia_core::discord_rpc::DiscordRpcState;
use aurelia_core::listenbrainz_core::{
    ListenBrainzCredentials, ListenBrainzListen, ListenBrainzState,
};
use aurelia_core::media_controls::MediaControlsState;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

/// Helper to get cached credentials from Tauri state
fn get_credentials(app: &AppHandle) -> ApiResult<Option<Credentials>> {
    let app_state: tauri::State<'_, aurelia_core::state::AppState> = app.state();

    // Check memory cache first
    if let Some(creds) = app_state.get_credentials() {
        return Ok(Some(creds));
    }

    // Load from disk
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))?;

    match aurelia_core::load_credentials(app_dir.to_string_lossy().to_string())? {
        Some(creds) => {
            // Cache for future
            app_state.set_credentials(Some(creds.clone()));
            Ok(Some(creds))
        }
        None => Ok(None),
    }
}

/// Tauri API implementation
pub struct TauriApiImpl {
    app: AppHandle,
}

impl TauriApiImpl {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Api for TauriApiImpl {
    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>> {
        get_credentials(&self.app)
    }

    async fn login_to_jellyfin(
        &self,
        server_url: String,
        username: String,
        password: String,
        device_id: String,
    ) -> ApiResult<serde_json::Value> {
        let login_resp =
            aurelia_core::authenticate(server_url.clone(), username.clone(), password, device_id)
                .await?;
        let creds = Credentials {
            server_url: server_url.clone(),
            username: username.clone(),
            token: login_resp.token.clone(),
            user_id: login_resp.user_id.clone(),
        };

        // Save to disk and cache
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e: tauri::Error| AppError::FileSystem(e.to_string()))?;
        let _ =
            aurelia_core::save_credentials(app_dir.to_string_lossy().to_string(), creds.clone());

        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        app_state.set_credentials(Some(creds.clone()));

        // Return as JSON value
        Ok(serde_json::json!({
            "token": login_resp.token,
            "userId": login_resp.user_id
        }))
    }

    async fn save_credentials(
        &self,
        server_url: String,
        username: String,
        token: String,
        user_id: String,
    ) -> ApiResult<()> {
        let creds = Credentials {
            server_url,
            username,
            token,
            user_id,
        };

        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::save_credentials(app_dir.to_string_lossy().to_string(), creds.clone())?;

        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        app_state.set_credentials(Some(creds));

        Ok(())
    }

    async fn clear_saved_credentials(&self) -> ApiResult<()> {
        // Clear memory cache
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        app_state.set_credentials(None);

        // Clear from disk
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::clear_credentials(app_dir.to_string_lossy().to_string())
    }

    async fn save_volume(&self, volume: f64) -> ApiResult<()> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let volume_path = app_dir.join("volume.json");

        let json =
            serde_json::to_string(&volume).map_err(|e| AppError::Serialization(e.to_string()))?;

        std::fs::write(&volume_path, json).map_err(|e| AppError::FileSystem(e.to_string()))?;

        Ok(())
    }

    async fn get_saved_volume(&self) -> ApiResult<Option<f64>> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let volume_path = app_dir.join("volume.json");

        if !volume_path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&volume_path)
            .map_err(|e| AppError::FileSystem(e.to_string()))?;

        let volume: f64 =
            serde_json::from_str(&json).map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(Some(volume))
    }

    async fn get_library(&self) -> ApiResult<LibraryData> {
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = app_state.songs.lock().unwrap().clone();

        // Derive albums and artists from songs
        let mut album_map: HashMap<String, Vec<Song>> = HashMap::new();
        let mut artist_map: HashMap<String, Artist> = HashMap::new();

        for song in &songs {
            if let Some(album_id) = &song.album_id {
                album_map
                    .entry(album_id.clone())
                    .or_default()
                    .push(song.clone());
            }

            if let Some(artist_ids) = &song.artist_ids {
                for (i, artist_id) in artist_ids.iter().enumerate() {
                    if !artist_map.contains_key(artist_id) {
                        let name = song
                            .artists
                            .as_ref()
                            .and_then(|a| a.get(i))
                            .cloned()
                            .unwrap_or_else(|| "Unknown Artist".to_string());
                        artist_map.insert(
                            artist_id.clone(),
                            Artist {
                                name,
                                id: artist_id.clone(),
                                image_tags: None,
                                image_url: None,
                                overview: None,
                                provider_ids: None,
                                community_rating: None,
                                song_count: None,
                                date_modified: None,
                                songs: None,
                            },
                        );
                    }
                }
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
                    artist: first_song.artists.as_ref()?.first()?.clone(),
                    artist_id: first_song.artist_ids.as_ref()?.first().cloned(),
                    album_art_url: first_song.album_art_url.clone(),
                    song_count: album_songs.len() as i64,
                    songs: None,
                    image_tags: None,
                    provider_ids: None,
                    date_created: first_song.date_created.clone(),
                    date_modified: None,
                })
            })
            .collect();

        let artists: Vec<Artist> = artist_map.into_values().collect();

        Ok(LibraryData {
            albums,
            artists,
            songs,
        })
    }

    async fn sync_library(&self) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;

        // Fetch songs from Jellyfin
        let songs = aurelia_core::fetch_songs(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
            app_dir.to_string_lossy().to_string(),
        )
        .await?;

        // Update in-memory state
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        *app_state.songs.lock().unwrap() = songs;

        Ok(())
    }

    async fn get_sync_state(&self) -> ApiResult<SyncStateInfo> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        // Get the internal sync state and convert to SyncStateInfo
        let internal_state = aurelia_core::get_sync_state(app_dir.to_string_lossy().to_string())?;
        Ok(SyncStateInfo {
            last_sync_time: Some(internal_state.last_sync_time),
            song_count: internal_state.song_count,
            artist_count: internal_state.artist_count,
            album_count: internal_state.album_count,
        })
    }

    async fn get_song(&self, song_id: String) -> ApiResult<Song> {
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = app_state.songs.lock().unwrap().clone();

        songs
            .into_iter()
            .find(|s| s.id == song_id)
            .ok_or_else(|| AppError::General("Song not found".to_string()))
    }

    async fn toggle_favorite_status(&self, item_id: String, is_favorite: bool) -> ApiResult<bool> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let new_state = aurelia_core::toggle_favorite(
            creds.server_url,
            creds.token,
            creds.user_id,
            item_id,
            is_favorite,
        )
        .await?;
        Ok(new_state)
    }

    async fn get_instant_mix(&self, item_id: String) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_instant_mix(creds.server_url, creds.token, item_id).await
    }

    async fn get_artist_share_urls(&self, artist_id: String) -> ApiResult<HashMap<String, String>> {
        // Get artist from cache
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;

        let artist =
            aurelia_core::get_cached_artist(app_dir.to_string_lossy().to_string(), artist_id)?
                .ok_or_else(|| AppError::General("Artist not found".to_string()))?;

        // Use MusicBrainz to get share URLs
        aurelia_core::services::MusicBrainzService::get_artist_share_urls(&artist)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn clear_image_from_cache(&self, item_id: String, image_type: String) -> ApiResult<()> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let cache_dir = app_dir.join("image_cache");
        let prefix = format!("{}_{}", item_id, image_type);
        if cache_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&cache_dir)
        {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(&prefix) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
        Ok(())
    }

    async fn report_playback_start(
        &self,
        item_id: String,
        position_ticks: Option<i64>,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let client =
            aurelia_core::services::JellyfinClient::with_auth(creds.server_url, creds.token);
        client
            .report_playback_start(&item_id, position_ticks)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn report_playback_progress(
        &self,
        item_id: String,
        position_ticks: i64,
        is_paused: bool,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let client =
            aurelia_core::services::JellyfinClient::with_auth(creds.server_url, creds.token);
        client
            .report_playback_progress(&item_id, Some(position_ticks), None, Some(is_paused))
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn report_playback_stop(&self, item_id: String, position_ticks: i64) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let client =
            aurelia_core::services::JellyfinClient::with_auth(creds.server_url, creds.token);
        client
            .report_playback_stop(&item_id, Some(position_ticks))
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn mark_item_played(&self, item_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let client = aurelia_core::services::JellyfinClient::with_auth(
            creds.server_url.clone(),
            creds.token.clone(),
        );
        client
            .mark_item_played(&creds.user_id, &item_id)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>> {
        // First get the song from cache
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = app_state.songs.lock().unwrap().clone();

        let song = songs
            .into_iter()
            .find(|s| s.id == item_id)
            .ok_or_else(|| AppError::General("Song not found".to_string()))?;

        aurelia_core::get_song_share_urls(song).await
    }

    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist> {
        // Try cache first
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;

        if let Ok(Some(artist)) = aurelia_core::get_cached_artist(
            app_dir.to_string_lossy().to_string(),
            artist_id.clone(),
        ) {
            return Ok(artist);
        }

        // Fetch from server
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_artist(
            creds.server_url,
            creds.token,
            creds.user_id,
            artist_id,
            app_dir.to_string_lossy().to_string(),
        )
        .await
    }

    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::get_related_artists(app_dir.to_string_lossy().to_string(), artist_id).await
    }

    async fn get_album(&self, album_id: String) -> ApiResult<Album> {
        // Try cache first
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;

        if let Ok(Some(album)) =
            aurelia_core::get_cached_album(app_dir.to_string_lossy().to_string(), album_id.clone())
        {
            return Ok(album);
        }

        // Fetch from server
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_album(
            creds.server_url,
            creds.token,
            creds.user_id,
            album_id,
            app_dir.to_string_lossy().to_string(),
        )
        .await
    }

    async fn get_album_share_urls(&self, album_id: String) -> ApiResult<HashMap<String, String>> {
        // Get album from cache
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;

        let album =
            aurelia_core::get_cached_album(app_dir.to_string_lossy().to_string(), album_id)?
                .ok_or_else(|| AppError::General("Album not found".to_string()))?;

        // Use MusicBrainz to get share URLs
        aurelia_core::services::MusicBrainzService::get_album_share_urls(&album)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn get_playlists(&self) -> ApiResult<Vec<Playlist>> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlists(creds.server_url, creds.token, creds.user_id).await
    }

    async fn get_playlist_items(&self, playlist_id: String) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlist_items(creds.server_url, creds.token, playlist_id).await
    }

    async fn create_playlist(&self, data: PlaylistCreateData) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let core_data = aurelia_core::models::PlaylistCreateData {
            name: data.name,
            ids: data.ids,
            user_id: creds.user_id,
            is_public: None,
        };
        aurelia_core::create_playlist(creds.server_url, creds.token, core_data).await
    }

    async fn update_playlist(
        &self,
        playlist_id: String,
        updates: PlaylistUpdateData,
    ) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let core_updates = aurelia_core::models::PlaylistUpdateData {
            name: updates.name,
            ids: updates.ids,
            user_id: None,
            is_public: None,
            songs: None,
            is_favorite: None,
        };
        aurelia_core::update_playlist(creds.server_url, creds.token, playlist_id, core_updates)
            .await
    }

    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::delete_playlist(creds.server_url, creds.token, playlist_id).await
    }

    async fn add_playlist_items(
        &self,
        playlist_id: String,
        song_ids: Vec<String>,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::add_playlist_items(creds.server_url, creds.token, playlist_id, song_ids).await
    }

    async fn remove_playlist_items(
        &self,
        playlist_id: String,
        song_ids: Vec<String>,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::remove_playlist_items(creds.server_url, creds.token, playlist_id, song_ids)
            .await
    }

    async fn get_home_view_data(&self) -> ApiResult<HomeViewData> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let all_songs = app_state.songs.lock().unwrap().clone();

        // Get recently played from server
        let recently_played = aurelia_core::get_recently_played(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
        )
        .await
        .unwrap_or_default();

        // Build albums with song counts
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        let mut albums: Vec<Album> = Vec::new();
        let mut album_song_counts: HashMap<String, usize> = HashMap::new();

        // First pass: count songs per album
        for song in &all_songs {
            if let Some(album_id) = &song.album_id {
                let count = album_song_counts.entry(album_id.clone()).or_insert(0);
                *count += 1;
            }
        }

        // Second pass: build album list with correct song counts
        let mut seen_albums: HashMap<String, bool> = HashMap::new();
        for song in &all_songs {
            if let Some(album_id) = &song.album_id
                && !seen_albums.contains_key(album_id)
            {
                seen_albums.insert(album_id.clone(), true);
                let song_count = album_song_counts.get(album_id).copied().unwrap_or(0) as i64;
                albums.push(Album {
                    id: Some(album_id.clone()),
                    name: song.album.clone().unwrap_or_default(),
                    artist: song
                        .artists
                        .as_ref()
                        .and_then(|a| a.first())
                        .cloned()
                        .unwrap_or_default(),
                    artist_id: song.artist_ids.as_ref().and_then(|a| a.first()).cloned(),
                    album_art_url: song.album_art_url.clone(),
                    song_count,
                    songs: None,
                    image_tags: None,
                    provider_ids: None,
                    date_created: song.date_created.clone(),
                    date_modified: None,
                });
            }
        }

        // Derive recently added albums (sort by newest song date in each album)
        let mut recently_added = albums.clone();
        recently_added.sort_by(|a, b| b.date_created.cmp(&a.date_created));
        recently_added.truncate(20);

        albums.shuffle(&mut rng);
        let random_albums: Vec<Album> = albums.iter().take(20).cloned().collect();

        // Featured albums (same as random for now)
        let featured_albums = random_albums.clone();

        Ok(HomeViewData {
            recently_played,
            recently_added,
            random_albums,
            featured_albums,
        })
    }

    async fn get_recently_played(&self) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_recently_played(creds.server_url, creds.token, creds.user_id).await
    }

    async fn get_image(
        &self,
        item_id: String,
        image_type: String,
        server_url: String,
        token: String,
        width: Option<u32>,
        quality: Option<u32>,
    ) -> ApiResult<Option<String>> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let cache_dir = app_dir.join("image_cache");
        std::fs::create_dir_all(&cache_dir).map_err(|e| AppError::FileSystem(e.to_string()))?;

        // Build cache filename
        let mut cache_name = format!("{}_{}", item_id, image_type);
        if let Some(w) = width {
            cache_name.push_str(&format!("_w{}", w));
        }
        if let Some(q) = quality {
            cache_name.push_str(&format!("_q{}", q));
        }
        let cache_path = cache_dir.join(&cache_name);

        // Return cached file if it exists
        if cache_path.exists() {
            return Ok(Some(cache_path.to_string_lossy().to_string()));
        }

        // Build the Jellyfin image URL
        let mut url = format!(
            "{}/Items/{}/Images/{}",
            server_url.trim_end_matches('/'),
            item_id,
            image_type
        );
        let mut query = Vec::new();
        if let Some(w) = width {
            query.push(format!("width={}", w));
        }
        if let Some(q) = quality {
            query.push(format!("quality={}", q));
        }
        query.push(format!("api_key={}", token));
        if !query.is_empty() {
            url.push('?');
            url.push_str(&query.join("&"));
        }

        // Download and cache
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        std::fs::write(&cache_path, &bytes).map_err(|e| AppError::FileSystem(e.to_string()))?;

        Ok(Some(cache_path.to_string_lossy().to_string()))
    }

    async fn get_audio_stream_url(
        &self,
        item_id: String,
        server_url: String,
        token: String,
        container: Option<String>,
    ) -> ApiResult<String> {
        Ok(aurelia_core::build_stream_url(
            server_url, token, item_id, container,
        ))
    }

    async fn register_client_capabilities(
        &self,
        server_url: String,
        token: String,
        device_id: String,
    ) -> ApiResult<()> {
        use aurelia_core::models::jellyfin::{
            ClientCapabilities, DeviceProfile, DirectPlayProfile, SubtitleProfile,
            TranscodingProfile,
        };

        // Create default device profile for audio
        let device_profile = DeviceProfile {
            name: Some("Aurelia Audio Profile".to_string()),
            id: Some(device_id.clone()),
            max_streaming_bitrate: Some(140000000),
            max_static_bitrate: Some(140000000),
            music_streaming_transcoding_bitrate: Some(384000),
            max_static_music_bitrate: Some(4000000),
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
                    container: "ogg".to_string(),
                    audio_codec: Some("vorbis".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
            ],
            transcoding_profiles: vec![TranscodingProfile {
                container: "mp3".to_string(),
                profile_type: "Audio".to_string(),
                video_codec: None,
                audio_codec: Some("mp3".to_string()),
                protocol: "http".to_string(),
                estimate_content_length: None,
                enable_mpegts_m2_ts_mode: None,
                transcode_seek_info: None,
                copy_timestamps: None,
                context: Some("Streaming".to_string()),
                enable_subtitles_in_manifest: None,
                max_audio_channels: None,
                min_segments: None,
                segment_length: None,
                break_on_non_key_frames: None,
                conditions: vec![],
                enable_audio_vbr_encoding: None,
            }],
            container_profiles: vec![],
            codec_profiles: vec![],
            subtitle_profiles: vec![SubtitleProfile {
                format: "srt".to_string(),
                method: "External".to_string(),
                didl_mode: None,
                language: None,
                container: None,
            }],
        };

        let capabilities = ClientCapabilities {
            playable_media_types: vec!["Audio".to_string()],
            supported_commands: vec![
                "PlayNow".to_string(),
                "PlayNext".to_string(),
                "SetVolume".to_string(),
                "ToggleMute".to_string(),
            ],
            supports_media_control: true,
            supports_persistent_identifier: true,
            device_profile,
            app_store_url: None,
            icon_url: None,
        };

        let client = aurelia_core::services::JellyfinClient::with_auth(server_url, token);
        client
            .register_capabilities(&capabilities)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn clear_image_cache(&self) -> ApiResult<()> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let cache_dir = app_dir.join("image_cache");
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).map_err(|e| AppError::FileSystem(e.to_string()))?;
        }
        Ok(())
    }

    async fn get_image_cache_stats(&self) -> ApiResult<String> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let cache_dir = app_dir.join("image_cache");
        let (mut file_count, mut total_size) = (0u64, 0u64);
        if cache_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&cache_dir)
        {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata()
                    && meta.is_file()
                {
                    file_count += 1;
                    total_size += meta.len();
                }
            }
        }
        Ok(serde_json::json!({
            "cache_dir": cache_dir.to_string_lossy(),
            "file_count": file_count,
            "total_size": total_size,
        })
        .to_string())
    }

    async fn get_lyrics(
        &self,
        _id: String,
        artist: String,
        title: String,
        _path: Option<String>,
    ) -> ApiResult<String> {
        Ok(aurelia_core::get_lyrics(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            artist,
            title,
        )
        .await)
    }

    async fn clear_cache(&self) -> ApiResult<()> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::clear_cache(app_dir.to_string_lossy().to_string())
    }

    // ─── ListenBrainz ────────────────────────────────────────────────

    async fn listenbrainz_set_credentials(
        &self,
        credentials: ListenBrainzCredentials,
    ) -> ApiResult<()> {
        let state: tauri::State<'_, ListenBrainzState> = self.app.state();
        aurelia_core::listenbrainz_core::listenbrainz_set_credentials(credentials, &state)
            .map_err(AppError::General)
    }

    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()> {
        let state: tauri::State<'_, ListenBrainzState> = self.app.state();
        aurelia_core::listenbrainz_core::listenbrainz_clear_credentials(&state)
            .map_err(AppError::General)
    }

    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool> {
        let state: tauri::State<'_, ListenBrainzState> = self.app.state();
        aurelia_core::listenbrainz_core::listenbrainz_is_authenticated(&state)
            .map_err(AppError::General)
    }

    async fn listenbrainz_validate_token(
        &self,
        user_token: String,
    ) -> ApiResult<ListenBrainzCredentials> {
        let state: tauri::State<'_, ListenBrainzState> = self.app.state();
        aurelia_core::listenbrainz_core::listenbrainz_validate_token(user_token, &state)
            .await
            .map_err(AppError::General)
    }

    async fn listenbrainz_submit_listen(
        &self,
        listen: ListenBrainzListen,
        timestamp: i64,
    ) -> ApiResult<()> {
        let state: tauri::State<'_, ListenBrainzState> = self.app.state();
        aurelia_core::listenbrainz_core::listenbrainz_submit_listen(
            listen,
            timestamp as f64,
            &state,
        )
        .await
        .map_err(AppError::General)
    }

    async fn listenbrainz_playing_now(
        &self,
        artist: String,
        track: String,
        album: Option<String>,
    ) -> ApiResult<()> {
        let state: tauri::State<'_, ListenBrainzState> = self.app.state();
        aurelia_core::listenbrainz_core::listenbrainz_playing_now(artist, track, album, &state)
            .await
            .map_err(AppError::General)
    }

    // ─── Audio (desktop-only) ────────────────────────────────────────

    async fn audio_init(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;

        if player_guard.is_none() {
            let player = aurelia_core::audio::AudioPlayer::new()
                .map_err(|e| AppError::General(e.to_string()))?;

            // Get analyzer buffer before moving player
            let analyzer_buffer = player.analyzer_buffer();

            *player_guard = Some(player);

            // Store analyzer buffer for spectrum events
            let mut buffer_guard = audio_state.analyzer_buffer.lock().unwrap();
            *buffer_guard = Some(analyzer_buffer);
        }

        Ok(())
    }

    async fn audio_play(&self, url: String, token: String) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;

        player
            .play_url(&url, &token)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_pause(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player.pause();
        Ok(())
    }

    async fn audio_resume(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player.resume();
        Ok(())
    }

    async fn audio_stop(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player.stop();
        Ok(())
    }

    async fn audio_seek(&self, position_secs: f64) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .seek_with_fallback(position_secs)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_get_position(&self) -> ApiResult<f64> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.get_position())
    }

    async fn audio_is_playing(&self) -> ApiResult<bool> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.is_playing())
    }

    async fn audio_get_volume(&self) -> ApiResult<f64> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.get_volume() as f64)
    }

    async fn audio_set_volume(&self, volume: f64) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player.set_volume(volume as f32);
        Ok(())
    }

    async fn audio_is_eq_enabled(&self) -> ApiResult<bool> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.is_eq_enabled())
    }

    async fn audio_set_eq_enabled(&self, enabled: bool) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .set_eq_enabled(enabled)
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_get_eq_band(&self, band: u32) -> ApiResult<f64> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.get_eq_band(band as usize) as f64)
    }

    async fn audio_set_eq_band(&self, band: u32, gain_db: f64) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .set_eq_band(band as usize, gain_db as f32)
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_get_all_eq_bands(&self) -> ApiResult<Vec<f64>> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player
            .get_all_eq_bands()
            .iter()
            .map(|&x| x as f64)
            .collect())
    }

    async fn audio_reset_eq(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .reset_eq()
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_advance_gapless(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .advance_to_next()
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_prepare_next(&self, url: String, token: String) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .prepare_next(&url, &token)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn audio_is_finished(&self) -> ApiResult<bool> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.is_finished())
    }

    async fn audio_set_analyzer_enabled(&self, enabled: bool) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player.set_analyzer_enabled(enabled);
        Ok(())
    }

    async fn audio_is_analyzer_enabled(&self) -> ApiResult<bool> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_ref()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        Ok(player.is_analyzer_enabled())
    }

    async fn audio_reinit(&self) -> ApiResult<()> {
        let audio_state: tauri::State<'_, AudioState> = self.app.state();
        let mut player_guard = audio_state.player.lock().await;
        let player = player_guard
            .as_mut()
            .ok_or_else(|| AppError::General("Audio player not initialized".to_string()))?;
        player
            .reinit()
            .map_err(|e| AppError::General(e.to_string()))
    }

    // ─── Discord RPC (desktop-only) ──────────────────────────────────

    async fn discord_rpc_start(&self, app_id: String) -> ApiResult<()> {
        let state: tauri::State<'_, DiscordRpcState> = self.app.state();
        state
            .start(app_id)
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn discord_rpc_stop(&self) -> ApiResult<()> {
        let state: tauri::State<'_, DiscordRpcState> = self.app.state();
        state.stop().map_err(|e| AppError::General(e.to_string()))
    }

    async fn discord_rpc_is_running(&self) -> ApiResult<bool> {
        let state: tauri::State<'_, DiscordRpcState> = self.app.state();
        Ok(state.is_running())
    }

    async fn discord_rpc_set_activity(&self, activity: RpcActivity) -> ApiResult<()> {
        let state: tauri::State<'_, DiscordRpcState> = self.app.state();
        state
            .set_activity(activity)
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn discord_rpc_clear_activity(&self) -> ApiResult<()> {
        let state: tauri::State<'_, DiscordRpcState> = self.app.state();
        state
            .clear_activity()
            .map_err(|e| AppError::General(e.to_string()))
    }

    // ─── Media Controls (desktop-only) ───────────────────────────────

    async fn media_update_now_playing(&self, payload: NowPlayingPayload) -> ApiResult<()> {
        let state: tauri::State<'_, MediaControlsState> = self.app.state();
        state.update_now_playing(payload).map_err(AppError::General)
    }

    async fn media_clear_now_playing(&self) -> ApiResult<()> {
        let state: tauri::State<'_, MediaControlsState> = self.app.state();
        state.clear_now_playing().map_err(AppError::General)
    }

    async fn media_set_playback_status(
        &self,
        is_playing: bool,
        position_secs: Option<f64>,
    ) -> ApiResult<()> {
        let state: tauri::State<'_, MediaControlsState> = self.app.state();
        state
            .set_playback_status(is_playing, position_secs)
            .map_err(AppError::General)
    }

    async fn media_set_button_enabled(&self, button: String, enabled: bool) -> ApiResult<()> {
        let state: tauri::State<'_, MediaControlsState> = self.app.state();
        state
            .set_button_enabled(&button, enabled)
            .map_err(AppError::General)
    }

    // ─── Last.fm (stub for now) ──────────────────────────────────────

    async fn lastfm_set_credentials(&self, _credentials: LastFmCredentials) -> ApiResult<()> {
        // TODO: Implement Last.fm in aurelia-core
        Ok(())
    }

    async fn lastfm_clear_credentials(&self) -> ApiResult<()> {
        Ok(())
    }

    async fn lastfm_is_authenticated(&self) -> ApiResult<bool> {
        Ok(false)
    }

    async fn lastfm_start_auth_server(&self) -> ApiResult<()> {
        Ok(())
    }

    async fn lastfm_authenticate(&self) -> ApiResult<LastFmCredentials> {
        Ok(LastFmCredentials {
            api_key: None,
            session_key: String::new(),
            username: String::new(),
        })
    }

    async fn lastfm_scrobble(
        &self,
        _artist: String,
        _track: String,
        _album: Option<String>,
        _timestamp: Option<i64>,
    ) -> ApiResult<()> {
        Ok(())
    }

    async fn lastfm_update_now_playing(
        &self,
        _artist: String,
        _track: String,
        _album: Option<String>,
    ) -> ApiResult<()> {
        Ok(())
    }

    // ─── Window/Tray (desktop-only) ──────────────────────────────────

    async fn show_main_window(&self) -> ApiResult<()> {
        if let Some(window) = self.app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
        Ok(())
    }

    async fn hide_main_window(&self) -> ApiResult<()> {
        if let Some(window) = self.app.get_webview_window("main") {
            let _ = window.hide();
        }
        Ok(())
    }

    async fn quit_application(&self) -> ApiResult<()> {
        self.app.cleanup_before_exit();
        std::process::exit(0);
    }

    async fn set_minimize_to_tray(&self, _minimize_to_tray: bool) -> ApiResult<()> {
        // TODO: Implement via AtomicBool in a state
        Ok(())
    }

    async fn set_close_to_tray(&self, _close_to_tray: bool) -> ApiResult<()> {
        // TODO: Implement via AtomicBool in a state
        Ok(())
    }
}
