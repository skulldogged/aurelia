//! API trait with macro annotations
//!
//! This module provides the Api trait annotated with #[aurelia_api]
//! which generates Tauri commands, Axum routes, and TypeScript client.
//! 
//! IMPORTANT: This trait is the single source of truth for the API.
//! All Tauri commands and TypeScript types are generated from it.

use aurelia_api_macros::aurelia_api;
use aurelia_core::{
    error::AppError,
    models::*,
    listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen},
};

// Re-export types from models for the Api trait
pub use aurelia_core::models::{LastFmCredentials, NowPlayingPayload, RpcActivity, SyncStateInfo};
use std::collections::HashMap;

pub type ApiResult<T> = Result<T, AppError>;

#[aurelia_api]
#[allow(async_fn_in_trait)]
pub trait Api {
    // ─── Auth ────────────────────────────────────────────────────

    #[api(POST "/auth/login")]
    async fn login_to_jellyfin(
        &self, 
        server_url: String, 
        username: String, 
        password: String,
        device_id: String
    ) -> ApiResult<serde_json::Value>; // Returns LoginResponse
    
    #[api(POST "/auth/credentials")]
    async fn save_credentials(
        &self, 
        server_url: String, 
        username: String, 
        token: String, 
        user_id: String
    ) -> ApiResult<()>;
    
    #[api(GET "/auth/credentials")]
    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>>;
    
    #[api(POST "/auth/credentials/clear")]
    async fn clear_saved_credentials(&self) -> ApiResult<()>;
    
    #[api(POST "/auth/volume")]
    async fn save_volume(&self, volume: f64) -> ApiResult<()>;
    
    #[api(GET "/auth/volume")]
    async fn get_saved_volume(&self) -> ApiResult<Option<f64>>;
    
    // ─── Library ─────────────────────────────────────────────────
    
    #[api(GET "/library")]
    async fn get_library(&self) -> ApiResult<LibraryData>;
    
    #[api(POST "/library/sync")]
    async fn sync_library(&self) -> ApiResult<()>;
    
    #[api(GET "/library/sync-state")]
    async fn get_sync_state(&self) -> ApiResult<SyncStateInfo>;
    
    // ─── Songs ───────────────────────────────────────────────────
    
    #[api(GET "/songs/{song_id}")]
    async fn get_song(&self, song_id: String) -> ApiResult<Song>;
    
    #[api(POST "/songs/{item_id}/favorite")]
    async fn toggle_favorite_status(&self, item_id: String, is_favorite: bool) -> ApiResult<bool>;
    
    #[api(GET "/songs/{item_id}/instant-mix")]
    async fn get_instant_mix(&self, item_id: String) -> ApiResult<Vec<Song>>;
    
    #[api(GET "/songs/{item_id}/share-urls")]
    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>>;
    
    // ─── Artists ─────────────────────────────────────────────────
    
    #[api(GET "/artists/{artist_id}")]
    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist>;
    
    #[api(GET "/artists/{artist_id}/related")]
    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>>;
    
    #[api(GET "/artists/{artist_id}/share-urls")]
    async fn get_artist_share_urls(&self, artist_id: String) -> ApiResult<HashMap<String, String>>;
    
    // ─── Albums ──────────────────────────────────────────────────
    
    #[api(GET "/albums/{album_id}")]
    async fn get_album(&self, album_id: String) -> ApiResult<Album>;
    
    #[api(GET "/albums/{album_id}/share-urls")]
    async fn get_album_share_urls(&self, album_id: String) -> ApiResult<HashMap<String, String>>;
    
    // ─── Playlists ───────────────────────────────────────────────
    
    #[api(GET "/playlists")]
    async fn get_playlists(&self) -> ApiResult<Vec<Playlist>>;
    
    #[api(GET "/playlists/{playlist_id}/items")]
    async fn get_playlist_items(&self, playlist_id: String) -> ApiResult<Vec<Song>>;
    
    #[api(POST "/playlists")]
    async fn create_playlist(&self, data: PlaylistCreateData) -> ApiResult<Playlist>;
    
    #[api(PATCH "/playlists/{playlist_id}")]
    async fn update_playlist(
        &self, 
        playlist_id: String, 
        updates: PlaylistUpdateData
    ) -> ApiResult<Playlist>;
    
    #[api(DELETE "/playlists/{playlist_id}")]
    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()>;
    
    #[api(POST "/playlists/{playlist_id}/items")]
    async fn add_playlist_items(
        &self, 
        playlist_id: String, 
        song_ids: Vec<String>
    ) -> ApiResult<()>;
    
    #[api(DELETE "/playlists/{playlist_id}/items")]
    async fn remove_playlist_items(
        &self, 
        playlist_id: String, 
        song_ids: Vec<String>
    ) -> ApiResult<()>;
    
    // ─── Home ────────────────────────────────────────────────────
    
    #[api(GET "/home")]
    async fn get_home_view_data(&self) -> ApiResult<HomeViewData>;
    
    #[api(GET "/home/recently-played")]
    async fn get_recently_played(&self) -> ApiResult<Vec<Song>>;
    
    // ─── Images ──────────────────────────────────────────────────
    
    #[api(GET "/images/{item_id}")]
    async fn get_image(
        &self, 
        item_id: String, 
        image_type: String, 
        server_url: String, 
        token: String, 
        width: Option<u32>, 
        quality: Option<u32>
    ) -> ApiResult<Option<String>>;
    
    #[api(POST "/cache/image-clear")]
    async fn clear_image_cache(&self) -> ApiResult<()>;
    
    #[api(GET "/cache/image-stats")]
    async fn get_image_cache_stats(&self) -> ApiResult<String>;
    
    #[api(POST "/cache/image-clear/{item_id}")]
    async fn clear_image_from_cache(&self, item_id: String, image_type: String) -> ApiResult<()>;
    
    // ─── Audio ───────────────────────────────────────────────────
    
    #[api(GET "/audio/{item_id}/stream-url")]
    async fn get_audio_stream_url(
        &self, 
        item_id: String, 
        server_url: String, 
        token: String, 
        container: Option<String>
    ) -> ApiResult<String>;
    
    // ─── Lyrics ──────────────────────────────────────────────────
    
    #[api(POST "/lyrics")]
    async fn get_lyrics(
        &self, 
        id: String, 
        artist: String, 
        title: String, 
        path: Option<String>
    ) -> ApiResult<String>;
    
    // ─── Cache ───────────────────────────────────────────────────
    
    #[api(POST "/cache/clear")]
    async fn clear_cache(&self) -> ApiResult<()>;
    
    // ─── Session / Playback Reporting ─────────────────────────────
    
    #[api(POST "/sessions/capabilities")]
    async fn register_client_capabilities(
        &self, 
        server_url: String, 
        token: String, 
        device_id: String
    ) -> ApiResult<()>;
    
    #[api(POST "/sessions/playing/start")]
    async fn report_playback_start(
        &self, 
        item_id: String, 
        position_ticks: Option<i64>
    ) -> ApiResult<()>;
    
    #[api(POST "/sessions/playing/progress")]
    async fn report_playback_progress(
        &self, 
        item_id: String, 
        position_ticks: i64, 
        is_paused: bool
    ) -> ApiResult<()>;
    
    #[api(POST "/sessions/playing/stop")]
    async fn report_playback_stop(&self, item_id: String, position_ticks: i64) -> ApiResult<()>;
    
    #[api(POST "/sessions/mark-played")]
    async fn mark_item_played(&self, item_id: String) -> ApiResult<()>;
    
    // ─── ListenBrainz ────────────────────────────────────────────
    
    #[api(POST "/listenbrainz/credentials")]
    async fn listenbrainz_set_credentials(
        &self, 
        credentials: ListenBrainzCredentials
    ) -> ApiResult<()>;
    
    #[api(DELETE "/listenbrainz/credentials")]
    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()>;
    
    #[api(GET "/listenbrainz/auth-status")]
    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool>;
    
    #[api(POST "/listenbrainz/validate")]
    async fn listenbrainz_validate_token(&self, user_token: String) -> ApiResult<ListenBrainzCredentials>;
    
    #[api(POST "/listenbrainz/submit-listen")]
    async fn listenbrainz_submit_listen(
        &self, 
        listen: ListenBrainzListen, 
        timestamp: i64
    ) -> ApiResult<()>;
    
    #[api(POST "/listenbrainz/playing-now")]
    async fn listenbrainz_playing_now(
        &self, 
        artist: String, 
        track: String, 
        album: Option<String>
    ) -> ApiResult<()>;
    
    // ─── Desktop-only operations ─────────────────────────────────
    // These are only available on desktop via Tauri
    
    #[api(POST "/audio/init", desktop_only)]
    async fn audio_init(&self) -> ApiResult<()>;
    
    #[api(POST "/audio/play", desktop_only)]
    async fn audio_play(&self, url: String, token: String) -> ApiResult<()>;
    
    #[api(POST "/audio/pause", desktop_only)]
    async fn audio_pause(&self) -> ApiResult<()>;
    
    #[api(POST "/audio/resume", desktop_only)]
    async fn audio_resume(&self) -> ApiResult<()>;
    
    #[api(POST "/audio/stop", desktop_only)]
    async fn audio_stop(&self) -> ApiResult<()>;
    
    #[api(GET "/audio/volume", desktop_only)]
    async fn audio_get_volume(&self) -> ApiResult<f64>;
    
    #[api(POST "/audio/volume", desktop_only)]
    async fn audio_set_volume(&self, volume: f64) -> ApiResult<()>;
    
    #[api(POST "/audio/seek", desktop_only)]
    async fn audio_seek(&self, position_secs: f64) -> ApiResult<()>;
    
    #[api(GET "/audio/position", desktop_only)]
    async fn audio_get_position(&self) -> ApiResult<f64>;
    
    #[api(GET "/audio/is-playing", desktop_only)]
    async fn audio_is_playing(&self) -> ApiResult<bool>;
    
    #[api(POST "/discord/start", desktop_only)]
    async fn discord_rpc_start(&self, app_id: String) -> ApiResult<()>;
    
    #[api(POST "/discord/stop", desktop_only)]
    async fn discord_rpc_stop(&self) -> ApiResult<()>;
    
    #[api(GET "/discord/is-running", desktop_only)]
    async fn discord_rpc_is_running(&self) -> ApiResult<bool>;
    
    #[api(POST "/discord/activity", desktop_only)]
    async fn discord_rpc_set_activity(&self, activity: RpcActivity) -> ApiResult<()>;
    
    #[api(POST "/discord/clear-activity", desktop_only)]
    async fn discord_rpc_clear_activity(&self) -> ApiResult<()>;
    
    // ─── Audio - Equalizer (desktop only) ────────────────────────
    
    #[api(GET "/audio/eq/enabled", desktop_only)]
    async fn audio_is_eq_enabled(&self) -> ApiResult<bool>;
    
    #[api(POST "/audio/eq/enabled", desktop_only)]
    async fn audio_set_eq_enabled(&self, enabled: bool) -> ApiResult<()>;
    
    #[api(GET "/audio/eq/band", desktop_only)]
    async fn audio_get_eq_band(&self, band: u32) -> ApiResult<f64>;
    
    #[api(POST "/audio/eq/band", desktop_only)]
    async fn audio_set_eq_band(&self, band: u32, gain_db: f64) -> ApiResult<()>;
    
    #[api(GET "/audio/eq/all-bands", desktop_only)]
    async fn audio_get_all_eq_bands(&self) -> ApiResult<Vec<f64>>;
    
    #[api(POST "/audio/eq/reset", desktop_only)]
    async fn audio_reset_eq(&self) -> ApiResult<()>;
    
    // ─── Audio - Gapless/Advanced (desktop only) ─────────────────
    
    #[api(POST "/audio/advance-gapless", desktop_only)]
    async fn audio_advance_gapless(&self) -> ApiResult<()>;
    
    #[api(POST "/audio/prepare-next", desktop_only)]
    async fn audio_prepare_next(&self, url: String, token: String) -> ApiResult<()>;
    
    #[api(GET "/audio/is-finished", desktop_only)]
    async fn audio_is_finished(&self) -> ApiResult<bool>;
    
    // ─── Audio - Analyzer (desktop only) ─────────────────────────
    
    #[api(POST "/audio/analyzer", desktop_only)]
    async fn audio_set_analyzer_enabled(&self, enabled: bool) -> ApiResult<()>;
    
    #[api(GET "/audio/analyzer", desktop_only)]
    async fn audio_is_analyzer_enabled(&self) -> ApiResult<bool>;
    
    #[api(POST "/audio/reinit", desktop_only)]
    async fn audio_reinit(&self) -> ApiResult<()>;
    
    // ─── Media Controls (desktop only) ───────────────────────────
    
    #[api(POST "/media/update-now-playing", desktop_only)]
    async fn media_update_now_playing(&self, payload: NowPlayingPayload) -> ApiResult<()>;
    
    #[api(POST "/media/clear-now-playing", desktop_only)]
    async fn media_clear_now_playing(&self) -> ApiResult<()>;
    
    #[api(POST "/media/playback-status", desktop_only)]
    async fn media_set_playback_status(&self, is_playing: bool, position_secs: Option<f64>) -> ApiResult<()>;
    
    #[api(POST "/media/button-enabled", desktop_only)]
    async fn media_set_button_enabled(&self, button: String, enabled: bool) -> ApiResult<()>;
    
    // ─── Last.fm (desktop only) ──────────────────────────────────
    
    #[api(POST "/lastfm/credentials", desktop_only)]
    async fn lastfm_set_credentials(&self, credentials: LastFmCredentials) -> ApiResult<()>;
    
    #[api(DELETE "/lastfm/credentials", desktop_only)]
    async fn lastfm_clear_credentials(&self) -> ApiResult<()>;
    
    #[api(GET "/lastfm/auth-status", desktop_only)]
    async fn lastfm_is_authenticated(&self) -> ApiResult<bool>;
    
    #[api(POST "/lastfm/start-auth-server", desktop_only)]
    async fn lastfm_start_auth_server(&self) -> ApiResult<()>;
    
    #[api(POST "/lastfm/authenticate", desktop_only)]
    async fn lastfm_authenticate(&self) -> ApiResult<LastFmCredentials>;
    
    #[api(POST "/lastfm/scrobble", desktop_only)]
    async fn lastfm_scrobble(&self, artist: String, track: String, album: Option<String>, timestamp: Option<i64>) -> ApiResult<()>;
    
    #[api(POST "/lastfm/playing-now", desktop_only)]
    async fn lastfm_update_now_playing(&self, artist: String, track: String, album: Option<String>) -> ApiResult<()>;
    
    // ─── System Tray / Window Management (desktop only) ───────────
    
    #[api(POST "/window/show", desktop_only)]
    async fn show_main_window(&self) -> ApiResult<()>;
    
    #[api(POST "/window/hide", desktop_only)]
    async fn hide_main_window(&self) -> ApiResult<()>;
    
    #[api(POST "/app/quit", desktop_only)]
    async fn quit_application(&self) -> ApiResult<()>;
    
    #[api(POST "/settings/minimize-to-tray", desktop_only)]
    async fn set_minimize_to_tray(&self, minimize_to_tray: bool) -> ApiResult<()>;
    
    #[api(POST "/settings/close-to-tray", desktop_only)]
    async fn set_close_to_tray(&self, close_to_tray: bool) -> ApiResult<()>;
}
