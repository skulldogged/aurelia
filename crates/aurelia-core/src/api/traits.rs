//! API trait definitions - single source of truth for all API operations
//! 
//! This trait defines the complete API surface that both Tauri commands
//! and HTTP endpoints implement.

use crate::error::AppError;
use crate::listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen};
use crate::models::*;
use std::collections::HashMap;

/// Result type for API operations
pub type ApiResult<T> = Result<T, AppError>;

/// Core API operations trait
/// 
/// This trait defines the complete API surface that both Tauri commands
/// and HTTP endpoints implement.
#[allow(async_fn_in_trait)]
pub trait Api {
    // Auth operations
    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>>;
    async fn authenticate(&self, server_url: String, username: String, password: String) -> ApiResult<Credentials>;
    async fn logout(&self) -> ApiResult<()>;
    
    // Library operations  
    async fn get_library(&self) -> ApiResult<LibraryData>;
    async fn sync_library(&self) -> ApiResult<()>;
    async fn get_sync_state(&self) -> ApiResult<crate::domain::SyncState>;
    
    // Song operations
    async fn get_song(&self, song_id: String) -> ApiResult<Song>;
    async fn toggle_favorite(&self, item_id: String, is_favorite: bool) -> ApiResult<bool>;
    async fn get_instant_mix(&self, item_id: String) -> ApiResult<Vec<Song>>;
    async fn get_song_share_urls(&self, song: Song) -> ApiResult<HashMap<String, String>>;
    
    // Artist operations
    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist>;
    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>>;
    
    // Album operations
    async fn get_album(&self, album_id: String) -> ApiResult<Album>;
    
    // Playlist operations
    async fn get_playlists(&self) -> ApiResult<Vec<Playlist>>;
    async fn get_playlist_items(&self, playlist_id: String) -> ApiResult<Vec<Song>>;
    async fn create_playlist(&self, data: PlaylistCreateData) -> ApiResult<Playlist>;
    async fn update_playlist(&self, playlist_id: String, updates: PlaylistUpdateData) -> ApiResult<Playlist>;
    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()>;
    async fn add_playlist_items(&self, playlist_id: String, item_ids: Vec<String>) -> ApiResult<()>;
    async fn remove_playlist_items(&self, playlist_id: String, item_ids: Vec<String>) -> ApiResult<()>;
    
    // Home operations
    async fn get_home_view_data(&self) -> ApiResult<HomeViewData>;
    async fn get_recently_played(&self) -> ApiResult<Vec<Song>>;
    
    // Audio operations
    async fn get_audio_stream_url(&self, item_id: String, container: Option<String>) -> ApiResult<String>;
    async fn get_saved_volume(&self) -> ApiResult<Option<f64>>;
    async fn save_volume(&self, volume: f64) -> ApiResult<()>;
    
    // Lyrics operations
    async fn get_lyrics(&self, id: String, artist: String, title: String, path: Option<String>) -> ApiResult<String>;
    
    // Cache operations
    async fn clear_cache(&self) -> ApiResult<()>;
    async fn get_image_cache_stats(&self) -> ApiResult<String>;
    
    // ListenBrainz operations
    async fn listenbrainz_set_credentials(&self, credentials: ListenBrainzCredentials) -> ApiResult<()>;
    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()>;
    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool>;
    async fn listenbrainz_validate_token(&self, user_token: String) -> ApiResult<ListenBrainzCredentials>;
    async fn listenbrainz_submit_listen(&self, listen: ListenBrainzListen, timestamp: i64) -> ApiResult<()>;
    async fn listenbrainz_playing_now(&self, artist: String, track: String, album: Option<String>) -> ApiResult<()>;
}
