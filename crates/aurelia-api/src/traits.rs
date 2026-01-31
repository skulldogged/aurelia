//! API trait with macro annotations
//!
//! This module provides the Api trait annotated with #[aurelia_api]
//! which generates Tauri commands, Axum routes, and TypeScript client.

use aurelia_api_macros::aurelia_api;
use aurelia_core::{
    error::AppError,
    models::*,
    listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen},
};
use std::collections::HashMap;

pub type ApiResult<T> = Result<T, AppError>;

#[aurelia_api]
#[allow(async_fn_in_trait)]
pub trait Api {
    // ─── Auth ────────────────────────────────────────────────────
    
    #[api(GET "/auth/credentials")]
    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>>;
    
    #[api(POST "/auth/login")]
    async fn authenticate(&self, server_url: String, username: String, password: String) -> ApiResult<Credentials>;
    
    #[api(POST "/auth/logout")]
    async fn logout(&self) -> ApiResult<()>;
    
    // ─── Library ─────────────────────────────────────────────────
    
    #[api(GET "/library")]
    async fn get_library(&self) -> ApiResult<LibraryData>;
    
    #[api(POST "/library/sync")]
    async fn sync_library(&self) -> ApiResult<()>;
    
    #[api(GET "/library/sync-state")]
    async fn get_sync_state(&self) -> ApiResult<aurelia_core::domain::SyncState>;
    
    // ─── Songs ───────────────────────────────────────────────────
    
    #[api(GET "/songs/{song_id}")]
    async fn get_song(&self, song_id: String) -> ApiResult<Song>;
    
    #[api(POST "/songs/{item_id}/favorite")]
    async fn toggle_favorite_status(&self, item_id: String, is_favorite: bool) -> ApiResult<bool>;
    
    #[api(GET "/songs/{item_id}/instant-mix")]
    async fn get_instant_mix(&self, item_id: String, limit: Option<u32>) -> ApiResult<Vec<Song>>;
    
    #[api(GET "/songs/{item_id}/share-urls")]
    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>>;
    
    // ─── Artists ─────────────────────────────────────────────────
    
    #[api(GET "/artists/{artist_id}")]
    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist>;
    
    #[api(GET "/artists/{artist_id}/related")]
    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>>;
    
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
    async fn update_playlist(&self, playlist_id: String, updates: PlaylistUpdateData) -> ApiResult<Playlist>;
    
    #[api(DELETE "/playlists/{playlist_id}")]
    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()>;
    
    #[api(POST "/playlists/{playlist_id}/items")]
    async fn add_playlist_items(&self, playlist_id: String, song_ids: Vec<String>) -> ApiResult<()>;
    
    #[api(DELETE "/playlists/{playlist_id}/items")]
    async fn remove_playlist_items(&self, playlist_id: String, song_ids: Vec<String>) -> ApiResult<()>;
    
    // ─── Home ────────────────────────────────────────────────────
    
    #[api(GET "/home")]
    async fn get_home_view_data(&self) -> ApiResult<HomeViewData>;
    
    #[api(GET "/home/recently-played")]
    async fn get_recently_played(&self, limit: Option<u32>) -> ApiResult<Vec<Song>>;
    
    // ─── Images ──────────────────────────────────────────────────
    
    #[api(GET "/images/{item_id}")]
    async fn get_image(&self, item_id: String, image_type: String, server_url: String, token: String, width: Option<u32>, quality: Option<u32>) -> ApiResult<Option<String>>;
    
    #[api(POST "/cache/image-clear")]
    async fn clear_image_cache(&self) -> ApiResult<()>;
    
    #[api(GET "/cache/image-stats")]
    async fn get_image_cache_stats(&self) -> ApiResult<String>;
    
    // ─── Audio ───────────────────────────────────────────────────
    
    #[api(GET "/audio/{item_id}/stream-url")]
    async fn get_audio_stream_url(&self, item_id: String, server_url: String, token: String, container: Option<String>) -> ApiResult<String>;
    
    // ─── Lyrics ──────────────────────────────────────────────────
    
    #[api(POST "/lyrics")]
    async fn get_lyrics(&self, id: String, artist: String, title: String, path: Option<String>) -> ApiResult<String>;
    
    // ─── Cache ───────────────────────────────────────────────────
    
    #[api(POST "/cache/clear")]
    async fn clear_cache(&self) -> ApiResult<()>;
    
    // ─── ListenBrainz ────────────────────────────────────────────
    
    #[api(POST "/listenbrainz/credentials")]
    async fn listenbrainz_set_credentials(&self, credentials: ListenBrainzCredentials) -> ApiResult<()>;
    
    #[api(DELETE "/listenbrainz/credentials")]
    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()>;
    
    #[api(GET "/listenbrainz/auth-status")]
    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool>;
    
    #[api(POST "/listenbrainz/validate")]
    async fn listenbrainz_validate_token(&self, user_token: String) -> ApiResult<ListenBrainzCredentials>;
    
    #[api(POST "/listenbrainz/submit-listen")]
    async fn listenbrainz_submit_listen(&self, listen: ListenBrainzListen, timestamp: i64) -> ApiResult<()>;
    
    #[api(POST "/listenbrainz/playing-now")]
    async fn listenbrainz_playing_now(&self, artist: String, track: String, album: Option<String>) -> ApiResult<()>;
    
    // ─── Session / Playback Reporting ─────────────────────────────
    
    #[api(POST "/sessions/capabilities")]
    async fn register_client_capabilities(&self, server_url: String, token: String, device_id: String) -> ApiResult<()>;
    
    #[api(POST "/sessions/playing")]
    async fn report_playback(&self, server_url: String, token: String, item_id: String, position_ticks: Option<i64>, event_name: Option<String>, is_paused: Option<bool>) -> ApiResult<()>;
    
    // ─── Desktop-only operations ─────────────────────────────────
    // These are only available on desktop via Tauri
    
    #[api(POST "/audio/play", desktop_only)]
    async fn audio_play(&self) -> ApiResult<()>;
    
    #[api(POST "/audio/pause", desktop_only)]
    async fn audio_pause(&self) -> ApiResult<()>;
    
    #[api(POST "/audio/stop", desktop_only)]
    async fn audio_stop(&self) -> ApiResult<()>;
    
    #[api(GET "/audio/volume", desktop_only)]
    async fn audio_get_volume(&self) -> ApiResult<f64>;
    
    #[api(POST "/audio/volume", desktop_only)]
    async fn audio_set_volume(&self, volume: f64) -> ApiResult<()>;
    
    #[api(POST "/discord/start", desktop_only)]
    async fn discord_rpc_start(&self, app_id: String) -> ApiResult<()>;
    
    #[api(POST "/discord/stop", desktop_only)]
    async fn discord_rpc_stop(&self) -> ApiResult<()>;
}
