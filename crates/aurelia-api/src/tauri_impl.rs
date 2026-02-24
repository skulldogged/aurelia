//! Tauri API implementation
//!
//! This module provides the Tauri-specific implementation of the Api trait.

use crate::shared::{lastfm_secret, profile_storage, session_reporting};
use crate::{
    Album, Api, ApiResult, AppError, Artist, AuthRequest, BackendProvider, Credentials,
    HomeViewData, LastFmCredentials, LibraryData, NowPlayingPayload, Playlist, PlaylistCreateData,
    PlaylistUpdateData, ProviderCapabilities, RpcActivity, Song, SyncStateInfo,
};
use aurelia_core::audio::AudioState;
use aurelia_core::discord_rpc::DiscordRpcState;
use aurelia_core::lastfm_core::{
    LastFmState, lastfm_authenticate, lastfm_clear_credentials, lastfm_is_authenticated,
    lastfm_scrobble, lastfm_set_api_secret, lastfm_set_credentials, lastfm_update_now_playing,
};
use aurelia_core::listenbrainz_core::{
    ListenBrainzCredentials, ListenBrainzListen, ListenBrainzState,
};
use aurelia_core::media_controls::MediaControlsState;
use aurelia_core::tray_settings;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use tauri::Emitter;
use tauri::{AppHandle, Manager};

fn extract_lastfm_token(request: &str) -> Option<String> {
    let line = request.lines().next()?;
    let token_pos = line.find("token=")?;
    let token_start = token_pos + "token=".len();
    let remainder = &line[token_start..];
    let end = remainder
        .find(|c| ['&', ' '].contains(&c))
        .unwrap_or(remainder.len());
    Some(remainder[..end].to_string())
}

fn lock_std_mutex<'a, T>(mutex: &'a Mutex<T>, resource: &str) -> ApiResult<MutexGuard<'a, T>> {
    mutex
        .lock()
        .map_err(|_| AppError::General(format!("Failed to lock {resource}")))
}

fn get_credentials(app: &AppHandle) -> ApiResult<Option<Credentials>> {
    let app_state: tauri::State<'_, aurelia_core::state::AppState> = app.state();

    if let Some(creds) = app_state.get_credentials() {
        return Ok(Some(creds));
    }

    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))?;

    match aurelia_core::load_credentials(app_dir.to_string_lossy().to_string())? {
        Some(creds) => {
            app_state.set_credentials(Some(creds.clone()));
            Ok(Some(creds))
        }
        None => Ok(None),
    }
}

fn get_base_app_data_dir(app: &AppHandle) -> ApiResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))
}

fn get_profile_data_dir_for_credentials(
    app: &AppHandle,
    credentials: &Credentials,
) -> ApiResult<PathBuf> {
    let base_dir = get_base_app_data_dir(app)?;
    profile_storage::profile_data_dir(&base_dir, credentials)
}

fn get_active_app_data_dir(app: &AppHandle) -> ApiResult<PathBuf> {
    let base_dir = get_base_app_data_dir(app)?;
    let credentials = get_credentials(app)?;
    profile_storage::resolve_active_data_dir(&base_dir, credentials.as_ref())
}

fn image_extension_from_content_type(content_type: &str) -> &'static str {
    let mime = content_type.split(';').next().unwrap_or(content_type).trim();
    match mime {
        "image/avif" => "avif",
        "image/gif" => "gif",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/svg+xml" => "svg",
        "image/webp" => "webp",
        _ => "img",
    }
}

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

    async fn detect_provider(&self, server_url: String) -> ApiResult<BackendProvider> {
        aurelia_core::detect_provider(server_url).await
    }

    async fn get_provider_capabilities(
        &self,
        provider: BackendProvider,
        _server_url: String,
    ) -> ApiResult<ProviderCapabilities> {
        Ok(aurelia_core::get_provider_capabilities(provider))
    }

    async fn authenticate(&self, request: AuthRequest) -> ApiResult<Credentials> {
        let login_resp = aurelia_core::authenticate(request.clone()).await?;
        Ok(Credentials {
            provider: request.provider,
            server_url: request.server_url,
            username: request.username,
            token: login_resp.token,
            user_id: login_resp.user_id,
        })
    }

    async fn save_credentials(&self, creds: Credentials) -> ApiResult<()> {
        let base_dir = get_base_app_data_dir(&self.app)?;
        aurelia_core::save_credentials(base_dir.to_string_lossy().to_string(), creds.clone())?;

        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let profile_dir = get_profile_data_dir_for_credentials(&self.app, &creds)?;
        let cached_songs = aurelia_core::load_cached_songs(profile_dir.to_string_lossy().to_string())
            .unwrap_or_default();

        *lock_std_mutex(&app_state.songs, "song cache")? = cached_songs;
        *lock_std_mutex(&app_state.artists, "artist cache")? = Vec::new();
        *lock_std_mutex(&app_state.albums, "album cache")? = Vec::new();
        app_state.set_credentials(Some(creds));

        Ok(())
    }

    async fn clear_saved_credentials(&self) -> ApiResult<()> {
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        app_state.set_credentials(None);

        let base_dir = get_base_app_data_dir(&self.app)?;
        aurelia_core::clear_credentials(base_dir.to_string_lossy().to_string())
    }

    async fn save_volume(&self, volume: f64) -> ApiResult<()> {
        let app_dir = get_active_app_data_dir(&self.app)?;
        let volume_path = app_dir.join("volume.json");

        let json =
            serde_json::to_string(&volume).map_err(|e| AppError::Serialization(e.to_string()))?;

        std::fs::write(&volume_path, json).map_err(|e| AppError::FileSystem(e.to_string()))?;

        Ok(())
    }

    async fn get_saved_volume(&self) -> ApiResult<Option<f64>> {
        let app_dir = get_active_app_data_dir(&self.app)?;
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
        let songs = lock_std_mutex(&app_state.songs, "song cache")?.clone();
        Ok(aurelia_core::domain::services::derive_library_data(&songs))
    }

    async fn sync_library(&self) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let app_dir = get_active_app_data_dir(&self.app)?;

        let _report = aurelia_core::sync_library_smart(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
            app_dir.to_string_lossy().to_string(),
        )
        .await?;

        let songs = aurelia_core::load_cached_songs(app_dir.to_string_lossy().to_string())?;
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        *lock_std_mutex(&app_state.songs, "song cache")? = songs;

        Ok(())
    }

    async fn get_sync_state(&self) -> ApiResult<SyncStateInfo> {
        let app_dir = get_active_app_data_dir(&self.app)?;
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
        let songs = lock_std_mutex(&app_state.songs, "song cache")?.clone();

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
        let app_dir = get_active_app_data_dir(&self.app)?;

        let artist =
            aurelia_core::get_cached_artist(app_dir.to_string_lossy().to_string(), artist_id)?
                .ok_or_else(|| AppError::General("Artist not found".to_string()))?;

        aurelia_core::services::MusicBrainzService::get_artist_share_urls(&artist)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn clear_image_from_cache(&self, item_id: String, image_type: String) -> ApiResult<()> {
        let app_dir = get_active_app_data_dir(&self.app)?;
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
        session_reporting::report_playback_start(
            creds.server_url,
            creds.token,
            creds.user_id,
            item_id,
            position_ticks,
        )
        .await
    }

    async fn report_playback_progress(
        &self,
        item_id: String,
        position_ticks: i64,
        is_paused: bool,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        session_reporting::report_playback_progress(
            creds.server_url,
            creds.token,
            creds.user_id,
            item_id,
            position_ticks,
            is_paused,
        )
        .await
    }

    async fn report_playback_stop(&self, item_id: String, position_ticks: i64) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        session_reporting::report_playback_stop(
            creds.server_url,
            creds.token,
            creds.user_id,
            item_id,
            position_ticks,
        )
        .await
    }

    async fn mark_item_played(&self, item_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::mark_item_played(creds.server_url, creds.token, creds.user_id, item_id).await
    }

    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>> {
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = lock_std_mutex(&app_state.songs, "song cache")?.clone();

        let song = songs
            .into_iter()
            .find(|s| s.id == item_id)
            .ok_or_else(|| AppError::General("Song not found".to_string()))?;

        aurelia_core::get_song_share_urls(song).await
    }

    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist> {
        let app_dir = get_active_app_data_dir(&self.app)?;

        if let Ok(Some(artist)) = aurelia_core::get_cached_artist(
            app_dir.to_string_lossy().to_string(),
            artist_id.clone(),
        ) {
            return Ok(artist);
        }

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
        let app_dir = get_active_app_data_dir(&self.app)?;
        aurelia_core::get_related_artists(app_dir.to_string_lossy().to_string(), artist_id).await
    }

    async fn get_album(&self, album_id: String) -> ApiResult<Album> {
        let app_dir = get_active_app_data_dir(&self.app)?;

        if let Ok(Some(album)) =
            aurelia_core::get_cached_album(app_dir.to_string_lossy().to_string(), album_id.clone())
        {
            return Ok(album);
        }

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
        let app_dir = get_active_app_data_dir(&self.app)?;

        let album =
            aurelia_core::get_cached_album(app_dir.to_string_lossy().to_string(), album_id)?
                .ok_or_else(|| AppError::General("Album not found".to_string()))?;

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
        let all_songs = lock_std_mutex(&app_state.songs, "song cache")?.clone();

        let recently_played = aurelia_core::get_recently_played(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
        )
        .await
        .unwrap_or_default();

        let mut rng = rand::rng();
        Ok(aurelia_core::domain::services::derive_home_view_data(
            &all_songs,
            recently_played,
            aurelia_core::domain::services::HomeViewLimits::default(),
            &mut rng,
        ))
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
        let app_dir = get_active_app_data_dir(&self.app)?;
        let cache_dir = app_dir.join("image_cache");
        std::fs::create_dir_all(&cache_dir).map_err(|e| AppError::FileSystem(e.to_string()))?;

        let mut cache_name = format!("{}_{}", item_id, image_type);
        if let Some(w) = width {
            cache_name.push_str(&format!("_w{}", w));
        }
        if let Some(q) = quality {
            cache_name.push_str(&format!("_q{}", q));
        }

        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if file_name == cache_name || file_name.starts_with(&format!("{cache_name}.")) {
                    return Ok(Some(entry.path().to_string_lossy().to_string()));
                }
            }
        }

        let Some(url) = aurelia_core::build_image_url(
            server_url,
            token,
            item_id,
            image_type,
            width,
            quality,
        )? else {
            return Ok(None);
        };

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let extension = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(image_extension_from_content_type)
            .unwrap_or("img");
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        let cache_path = cache_dir.join(format!("{cache_name}.{extension}"));
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
        session_reporting::register_client_capabilities(server_url, token, device_id).await
    }

    async fn clear_image_cache(&self) -> ApiResult<()> {
        let app_dir = get_active_app_data_dir(&self.app)?;
        let cache_dir = app_dir.join("image_cache");
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).map_err(|e| AppError::FileSystem(e.to_string()))?;
        }
        Ok(())
    }

    async fn get_image_cache_stats(&self) -> ApiResult<String> {
        let app_dir = get_active_app_data_dir(&self.app)?;
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
        id: String,
        artist: String,
        title: String,
        _path: Option<String>,
    ) -> ApiResult<String> {
        let (server_url, token) = match get_credentials(&self.app)? {
            Some(creds) => (creds.server_url, creds.token),
            None => (String::new(), String::new()),
        };
        Ok(aurelia_core::get_lyrics(server_url, token, id, artist, title).await)
    }

    async fn get_parsed_lyrics(
        &self,
        id: String,
        artist: String,
        title: String,
        path: Option<String>,
    ) -> ApiResult<aurelia_core::models::ParsedLyrics> {
        let (server_url, token) = match get_credentials(&self.app)? {
            Some(creds) => (creds.server_url, creds.token),
            None => (String::new(), String::new()),
        };

        let app_dir = get_active_app_data_dir(&self.app)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let lyrics_server_url = if !app_dir.is_empty() {
            aurelia_core::load_setting(app_dir, "lyrics_server_url".to_string())
                .ok()
                .flatten()
        } else {
            None
        };

        Ok(aurelia_core::get_parsed_lyrics(
            server_url,
            token,
            id,
            artist,
            title,
            path,
            lyrics_server_url,
        )
        .await)
    }

    async fn get_sidecar_lyrics(
        &self,
        _item_id: String,
    ) -> ApiResult<aurelia_core::models::ParsedLyrics> {
        // Desktop doesn't serve sidecar files — this endpoint is only for the web backend
        Err(AppError::General(
            "Sidecar lyrics endpoint is only available on the web backend".to_string(),
        ))
    }

    async fn get_setting(&self, key: String) -> ApiResult<Option<String>> {
        let app_dir = get_active_app_data_dir(&self.app)?;
        aurelia_core::load_setting(app_dir.to_string_lossy().to_string(), key)
    }

    async fn save_setting(&self, key: String, value: String) -> ApiResult<()> {
        let app_dir = get_active_app_data_dir(&self.app)?;
        aurelia_core::save_setting(app_dir.to_string_lossy().to_string(), key, value)
    }

    async fn delete_setting(&self, key: String) -> ApiResult<()> {
        let app_dir = get_active_app_data_dir(&self.app)?;
        aurelia_core::delete_setting(app_dir.to_string_lossy().to_string(), key)
    }

    async fn clear_cache(&self) -> ApiResult<()> {
        let app_dir = get_active_app_data_dir(&self.app)?;
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

            let mut buffer_guard =
                lock_std_mutex(&audio_state.analyzer_buffer, "audio analyzer buffer")?;
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

    async fn lastfm_set_credentials(&self, credentials: LastFmCredentials) -> ApiResult<()> {
        let state: tauri::State<'_, LastFmState> = self.app.state();
        lastfm_set_credentials(credentials, &state).map_err(AppError::General)?;

        if let Ok(app_dir) = self.app.path().app_data_dir()
            && let Some(api_secret) = lastfm_secret::load(&app_dir)
        {
            let _ = lastfm_set_api_secret(api_secret, &state);
        }

        Ok(())
    }

    async fn lastfm_clear_credentials(&self) -> ApiResult<()> {
        let state: tauri::State<'_, LastFmState> = self.app.state();
        lastfm_clear_credentials(&state).map_err(AppError::General)?;

        if let Ok(app_dir) = self.app.path().app_data_dir() {
            let _ = lastfm_secret::clear(&app_dir);
        }

        Ok(())
    }

    async fn lastfm_is_authenticated(&self) -> ApiResult<bool> {
        let state: tauri::State<'_, LastFmState> = self.app.state();
        lastfm_is_authenticated(&state).map_err(AppError::General)
    }

    async fn lastfm_start_auth_server(&self) -> ApiResult<()> {
        let listener = TcpListener::bind("127.0.0.1:3000")
            .map_err(|e| AppError::General(format!("Failed to bind callback server: {e}")))?;

        let app = self.app.clone();
        thread::spawn(move || {
            if let Some(Ok(mut stream)) = listener.incoming().next() {
                let mut buffer = [0u8; 4096];
                let bytes_read = stream.read(&mut buffer).unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);

                if let Some(token) = extract_lastfm_token(&request) {
                    let _ = app.emit("lastfm://token-received", token);
                }

                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\nLast.fm authorization received. You can close this window.";
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        Ok(())
    }

    async fn lastfm_authenticate(
        &self,
        api_key: String,
        api_secret: String,
        token: String,
    ) -> ApiResult<LastFmCredentials> {
        let state: tauri::State<'_, LastFmState> = self.app.state();
        let credentials = lastfm_authenticate(api_key, api_secret.clone(), token, &state)
            .await
            .map_err(AppError::General)?;

        if let Ok(app_dir) = self.app.path().app_data_dir() {
            let _ = lastfm_secret::save(&app_dir, &api_secret);
        }

        Ok(credentials)
    }

    async fn lastfm_scrobble(
        &self,
        artist: String,
        track: String,
        album: Option<String>,
        timestamp: Option<i64>,
    ) -> ApiResult<()> {
        let state: tauri::State<'_, LastFmState> = self.app.state();
        lastfm_scrobble(artist, track, album, timestamp, &state)
            .await
            .map_err(AppError::General)
    }

    async fn lastfm_update_now_playing(
        &self,
        artist: String,
        track: String,
        album: Option<String>,
    ) -> ApiResult<()> {
        let state: tauri::State<'_, LastFmState> = self.app.state();
        lastfm_update_now_playing(artist, track, album, &state)
            .await
            .map_err(AppError::General)
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
        tray_settings::set_minimize_to_tray(_minimize_to_tray);
        Ok(())
    }

    async fn set_close_to_tray(&self, _close_to_tray: bool) -> ApiResult<()> {
        tray_settings::set_close_to_tray(_close_to_tray);
        Ok(())
    }
}
