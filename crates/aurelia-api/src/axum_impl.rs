//! Axum API implementation
//!
//! This module provides the Axum-specific implementation of the Api trait
//! for the web backend.

use crate::shared::{lastfm_secret, session_reporting};
use crate::{
    Album, Api, ApiResult, AppError, Artist, Credentials, HomeViewData, LastFmCredentials,
    LibraryData, NowPlayingPayload, Playlist, PlaylistCreateData, PlaylistUpdateData, RpcActivity,
    Song, SyncStateInfo,
};
use aurelia_core::lastfm_core::{
    LastFmState, lastfm_authenticate, lastfm_clear_credentials, lastfm_is_authenticated,
    lastfm_scrobble, lastfm_set_api_secret, lastfm_set_credentials, lastfm_update_now_playing,
};
use aurelia_core::listenbrainz_core::ListenBrainzState;
use aurelia_core::listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Application state for Axum
#[derive(Clone)]
pub struct AppState {
    pub app_data_dir: PathBuf,
    pub listenbrainz_state: Arc<ListenBrainzState>,
    pub lastfm_state: Arc<LastFmState>,
}

/// Helper to get cached credentials
fn get_credentials(state: &AppState) -> ApiResult<Option<Credentials>> {
    aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string())
}

/// Axum API implementation
pub struct AxumApiImpl {
    state: Arc<AppState>,
}

impl AxumApiImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl Api for AxumApiImpl {
    // ─── Auth ────────────────────────────────────────────────────

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
        serde_json::to_value(login_resp).map_err(|e| AppError::General(e.to_string()))
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
        aurelia_core::save_credentials(self.state.app_data_dir.to_string_lossy().to_string(), creds)
    }

    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>> {
        get_credentials(&self.state)
    }

    async fn clear_saved_credentials(&self) -> ApiResult<()> {
        aurelia_core::clear_credentials(self.state.app_data_dir.to_string_lossy().to_string())
    }

    async fn save_volume(&self, _volume: f64) -> ApiResult<()> {
        Ok(())
    }

    async fn get_saved_volume(&self) -> ApiResult<Option<f64>> {
        Ok(None)
    }

    // ─── Library ─────────────────────────────────────────────────

    async fn get_library(&self) -> ApiResult<LibraryData> {
        let songs =
            aurelia_core::load_cached_songs(self.state.app_data_dir.to_string_lossy().to_string())?;
        Ok(aurelia_core::domain::services::derive_library_data(&songs))
    }

    async fn sync_library(&self) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;

        // Fetch songs from Jellyfin (result is saved to disk by fetch_songs)
        let _songs = aurelia_core::fetch_songs(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
            self.state.app_data_dir.to_string_lossy().to_string(),
        )
        .await?;

        Ok(())
    }

    async fn get_sync_state(&self) -> ApiResult<SyncStateInfo> {
        let state =
            aurelia_core::get_sync_state(self.state.app_data_dir.to_string_lossy().to_string())?;
        Ok(SyncStateInfo {
            last_sync_time: Some(state.last_sync_time),
            song_count: state.song_count,
            artist_count: state.artist_count,
            album_count: state.album_count,
        })
    }

    // ─── Songs ───────────────────────────────────────────────────

    async fn get_song(&self, song_id: String) -> ApiResult<Song> {
        // Try cache first
        if let Ok(Some(song)) = aurelia_core::get_cached_song(
            self.state.app_data_dir.to_string_lossy().to_string(),
            song_id.clone(),
        ) {
            return Ok(song);
        }

        Err(AppError::General("Song not found".to_string()))
    }

    async fn toggle_favorite_status(&self, item_id: String, is_favorite: bool) -> ApiResult<bool> {
        let creds = get_credentials(&self.state)?
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
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_instant_mix(creds.server_url, creds.token, item_id).await
    }

    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>> {
        // Get song from cache
        let song = aurelia_core::get_cached_song(
            self.state.app_data_dir.to_string_lossy().to_string(),
            item_id,
        )?
        .ok_or_else(|| AppError::General("Song not found".to_string()))?;

        aurelia_core::get_song_share_urls(song).await
    }

    // ─── Artists ─────────────────────────────────────────────────

    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist> {
        // Try cache first
        if let Ok(Some(artist)) = aurelia_core::get_cached_artist(
            self.state.app_data_dir.to_string_lossy().to_string(),
            artist_id.clone(),
        ) {
            return Ok(artist);
        }

        // Fetch from server
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_artist(
            creds.server_url,
            creds.token,
            creds.user_id,
            artist_id,
            self.state.app_data_dir.to_string_lossy().to_string(),
        )
        .await
    }

    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>> {
        aurelia_core::get_related_artists(
            self.state.app_data_dir.to_string_lossy().to_string(),
            artist_id,
        )
        .await
    }

    async fn get_artist_share_urls(&self, artist_id: String) -> ApiResult<HashMap<String, String>> {
        let artist = aurelia_core::get_cached_artist(
            self.state.app_data_dir.to_string_lossy().to_string(),
            artist_id,
        )?
        .ok_or_else(|| AppError::General("Artist not found".to_string()))?;

        aurelia_core::services::MusicBrainzService::get_artist_share_urls(&artist)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    // ─── Albums ──────────────────────────────────────────────────

    async fn get_album(&self, album_id: String) -> ApiResult<Album> {
        // Try cache first
        if let Ok(Some(album)) = aurelia_core::get_cached_album(
            self.state.app_data_dir.to_string_lossy().to_string(),
            album_id.clone(),
        ) {
            return Ok(album);
        }

        // Fetch from server
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_album(
            creds.server_url,
            creds.token,
            creds.user_id,
            album_id,
            self.state.app_data_dir.to_string_lossy().to_string(),
        )
        .await
    }

    async fn get_album_share_urls(&self, album_id: String) -> ApiResult<HashMap<String, String>> {
        // Get album from cache
        let album = aurelia_core::get_cached_album(
            self.state.app_data_dir.to_string_lossy().to_string(),
            album_id,
        )?
        .ok_or_else(|| AppError::General("Album not found".to_string()))?;

        // Use MusicBrainz to get share URLs
        aurelia_core::services::MusicBrainzService::get_album_share_urls(&album)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    // ─── Playlists ───────────────────────────────────────────────

    async fn get_playlists(&self) -> ApiResult<Vec<Playlist>> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlists(creds.server_url, creds.token, creds.user_id).await
    }

    async fn get_playlist_items(&self, playlist_id: String) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlist_items(creds.server_url, creds.token, playlist_id).await
    }

    async fn create_playlist(&self, data: PlaylistCreateData) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        // Build the proper aurelia_core PlaylistCreateData
        let core_data = aurelia_core::models::PlaylistCreateData {
            name: data.name,
            ids: data.ids,
            user_id: creds.user_id.clone(),
            is_public: data.is_public,
        };
        aurelia_core::create_playlist(creds.server_url, creds.token, core_data).await
    }

    async fn update_playlist(
        &self,
        playlist_id: String,
        updates: PlaylistUpdateData,
    ) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        // Build the proper aurelia_core PlaylistUpdateData
        let core_data = aurelia_core::models::PlaylistUpdateData {
            name: updates.name,
            ids: updates.ids,
            user_id: Some(creds.user_id.clone()),
            is_public: updates.is_public,
            songs: None,
            is_favorite: None,
        };
        aurelia_core::update_playlist(creds.server_url, creds.token, playlist_id, core_data).await
    }

    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::delete_playlist(creds.server_url, creds.token, playlist_id).await
    }

    async fn add_playlist_items(
        &self,
        playlist_id: String,
        song_ids: Vec<String>,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::add_playlist_items(creds.server_url, creds.token, playlist_id, song_ids).await
    }

    async fn remove_playlist_items(
        &self,
        playlist_id: String,
        song_ids: Vec<String>,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::remove_playlist_items(creds.server_url, creds.token, playlist_id, song_ids)
            .await
    }

    // ─── Home ────────────────────────────────────────────────────

    async fn get_home_view_data(&self) -> ApiResult<HomeViewData> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let all_songs =
            aurelia_core::load_cached_songs(self.state.app_data_dir.to_string_lossy().to_string())?;

        // Get recently played from server
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
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_recently_played(creds.server_url, creds.token, creds.user_id).await
    }

    // ─── Images ──────────────────────────────────────────────────

    async fn get_image(
        &self,
        item_id: String,
        image_type: String,
        server_url: String,
        token: String,
        width: Option<u32>,
        quality: Option<u32>,
    ) -> ApiResult<Option<String>> {
        // Build the Jellyfin image URL with provided credentials
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

        Ok(Some(url))
    }

    async fn clear_image_cache(&self) -> ApiResult<()> {
        // Web doesn't cache images locally
        Ok(())
    }

    async fn get_image_cache_stats(&self) -> ApiResult<String> {
        Ok("{}".to_string())
    }

    async fn clear_image_from_cache(&self, _item_id: String, _image_type: String) -> ApiResult<()> {
        Ok(())
    }

    // ─── Audio ───────────────────────────────────────────────────

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

    // ─── Lyrics ──────────────────────────────────────────────────

    async fn get_lyrics(
        &self,
        id: String,
        artist: String,
        title: String,
        _path: Option<String>,
    ) -> ApiResult<String> {
        let (server_url, token) = match get_credentials(&self.state)? {
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
        let (server_url, token) = match get_credentials(&self.state)? {
            Some(creds) => (creds.server_url, creds.token),
            None => (String::new(), String::new()),
        };
        // Web backend runs co-located with media files, no need for remote sidecar fetch
        Ok(aurelia_core::get_parsed_lyrics(server_url, token, id, artist, title, path, None).await)
    }

    async fn get_sidecar_lyrics(
        &self,
        item_id: String,
    ) -> ApiResult<aurelia_core::models::ParsedLyrics> {
        let (server_url, token) = match get_credentials(&self.state)? {
            Some(creds) => (creds.server_url, creds.token),
            None => {
                return Err(AppError::Auth("Not authenticated".to_string()));
            }
        };
        aurelia_core::get_sidecar_lyrics(server_url, token, item_id).await
    }

    async fn get_setting(&self, key: String) -> ApiResult<Option<String>> {
        aurelia_core::load_setting(self.state.app_data_dir.to_string_lossy().to_string(), key)
    }

    async fn save_setting(&self, key: String, value: String) -> ApiResult<()> {
        aurelia_core::save_setting(
            self.state.app_data_dir.to_string_lossy().to_string(),
            key,
            value,
        )
    }

    async fn delete_setting(&self, key: String) -> ApiResult<()> {
        aurelia_core::delete_setting(self.state.app_data_dir.to_string_lossy().to_string(), key)
    }

    // ─── Cache ───────────────────────────────────────────────────

    async fn clear_cache(&self) -> ApiResult<()> {
        aurelia_core::clear_cache(self.state.app_data_dir.to_string_lossy().to_string())
    }

    // ─── Session / Playback Reporting ─────────────────────────────

    async fn register_client_capabilities(
        &self,
        server_url: String,
        token: String,
        device_id: String,
    ) -> ApiResult<()> {
        session_reporting::register_client_capabilities(server_url, token, device_id).await
    }

    async fn report_playback_start(
        &self,
        item_id: String,
        position_ticks: Option<i64>,
    ) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        session_reporting::report_playback_start(
            creds.server_url,
            creds.token,
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
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        session_reporting::report_playback_progress(
            creds.server_url,
            creds.token,
            item_id,
            position_ticks,
            is_paused,
        )
        .await
    }

    async fn report_playback_stop(&self, item_id: String, position_ticks: i64) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        session_reporting::report_playback_stop(
            creds.server_url,
            creds.token,
            item_id,
            position_ticks,
        )
        .await
    }

    async fn mark_item_played(&self, item_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?
            .ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;

        use reqwest::Client;

        let url = format!(
            "{}/Users/{}/PlayedItems/{}",
            creds.server_url.trim_end_matches('/'),
            creds.user_id,
            item_id
        );
        let client = Client::new();
        let response = client
            .post(&url)
            .header(
                "Authorization",
                format!("MediaBrowser Token=\"{}\"", creds.token),
            )
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to mark item played: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    // ─── ListenBrainz ────────────────────────────────────────────

    async fn listenbrainz_set_credentials(
        &self,
        credentials: ListenBrainzCredentials,
    ) -> ApiResult<()> {
        aurelia_core::listenbrainz_core::listenbrainz_set_credentials(
            credentials,
            self.state.listenbrainz_state.as_ref(),
        )
        .map_err(AppError::General)
    }

    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()> {
        aurelia_core::listenbrainz_core::listenbrainz_clear_credentials(
            self.state.listenbrainz_state.as_ref(),
        )
        .map_err(AppError::General)
    }

    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool> {
        aurelia_core::listenbrainz_core::listenbrainz_is_authenticated(
            self.state.listenbrainz_state.as_ref(),
        )
        .map_err(AppError::General)
    }

    async fn listenbrainz_validate_token(
        &self,
        user_token: String,
    ) -> ApiResult<ListenBrainzCredentials> {
        aurelia_core::listenbrainz_core::listenbrainz_validate_token(
            user_token,
            self.state.listenbrainz_state.as_ref(),
        )
        .await
        .map_err(AppError::General)
    }

    async fn listenbrainz_submit_listen(
        &self,
        listen: ListenBrainzListen,
        timestamp: i64,
    ) -> ApiResult<()> {
        aurelia_core::listenbrainz_core::listenbrainz_submit_listen(
            listen,
            timestamp as f64,
            self.state.listenbrainz_state.as_ref(),
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
        aurelia_core::listenbrainz_core::listenbrainz_playing_now(
            artist,
            track,
            album,
            self.state.listenbrainz_state.as_ref(),
        )
        .await
        .map_err(AppError::General)
    }

    // ─── Desktop-only operations ─────────────────────────────────

    async fn audio_init(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_play(&self, _url: String, _token: String) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_pause(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_resume(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_stop(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_get_volume(&self) -> ApiResult<f64> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_set_volume(&self, _volume: f64) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_seek(&self, _position_secs: f64) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_get_position(&self) -> ApiResult<f64> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_is_playing(&self) -> ApiResult<bool> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_start(&self, _app_id: String) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_stop(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_is_running(&self) -> ApiResult<bool> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_set_activity(&self, _activity: RpcActivity) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_clear_activity(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_is_eq_enabled(&self) -> ApiResult<bool> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_set_eq_enabled(&self, _enabled: bool) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_get_eq_band(&self, _band: u32) -> ApiResult<f64> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_set_eq_band(&self, _band: u32, _gain: f64) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_get_all_eq_bands(&self) -> ApiResult<Vec<f64>> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_reset_eq(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_advance_gapless(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_prepare_next(&self, _url: String, _token: String) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_is_finished(&self) -> ApiResult<bool> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_set_analyzer_enabled(&self, _enabled: bool) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_is_analyzer_enabled(&self) -> ApiResult<bool> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_reinit(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn media_update_now_playing(&self, _payload: NowPlayingPayload) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn media_clear_now_playing(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn media_set_playback_status(
        &self,
        _is_playing: bool,
        _position_secs: Option<f64>,
    ) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn media_set_button_enabled(&self, _button: String, _enabled: bool) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn lastfm_set_credentials(&self, credentials: LastFmCredentials) -> ApiResult<()> {
        let state = self.state.lastfm_state.as_ref();
        lastfm_set_credentials(credentials, state).map_err(AppError::General)?;

        if let Some(api_secret) = lastfm_secret::load(&self.state.app_data_dir) {
            let _ = lastfm_set_api_secret(api_secret, state);
        }

        Ok(())
    }

    async fn lastfm_clear_credentials(&self) -> ApiResult<()> {
        let state = self.state.lastfm_state.as_ref();
        lastfm_clear_credentials(state).map_err(AppError::General)?;
        let _ = lastfm_secret::clear(&self.state.app_data_dir);
        Ok(())
    }

    async fn lastfm_is_authenticated(&self) -> ApiResult<bool> {
        let state = self.state.lastfm_state.as_ref();
        lastfm_is_authenticated(state).map_err(AppError::General)
    }

    async fn lastfm_start_auth_server(&self) -> ApiResult<()> {
        Err(AppError::General(
            "Last.fm auth server is only supported on desktop".to_string(),
        ))
    }

    async fn lastfm_authenticate(
        &self,
        api_key: String,
        api_secret: String,
        token: String,
    ) -> ApiResult<LastFmCredentials> {
        let state = self.state.lastfm_state.as_ref();
        let credentials = lastfm_authenticate(api_key, api_secret.clone(), token, state)
            .await
            .map_err(AppError::General)?;

        let _ = lastfm_secret::save(&self.state.app_data_dir, &api_secret);

        Ok(credentials)
    }

    async fn lastfm_scrobble(
        &self,
        artist: String,
        track: String,
        album: Option<String>,
        timestamp: Option<i64>,
    ) -> ApiResult<()> {
        let state = self.state.lastfm_state.as_ref();
        lastfm_scrobble(artist, track, album, timestamp, state)
            .await
            .map_err(AppError::General)
    }

    async fn lastfm_update_now_playing(
        &self,
        artist: String,
        track: String,
        album: Option<String>,
    ) -> ApiResult<()> {
        let state = self.state.lastfm_state.as_ref();
        lastfm_update_now_playing(artist, track, album, state)
            .await
            .map_err(AppError::General)
    }

    async fn show_main_window(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn hide_main_window(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn quit_application(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn set_minimize_to_tray(&self, _minimize_to_tray: bool) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn set_close_to_tray(&self, _close_to_tray: bool) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }
}
