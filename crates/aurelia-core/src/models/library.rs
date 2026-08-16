use crate::models::{Album, Artist, Song};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct LibraryData {
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub songs: Vec<Song>,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct HomeViewData {
    pub recently_played: Vec<Song>,
    pub recently_added: Vec<Album>,
    pub random_albums: Vec<Album>,
    pub featured_albums: Vec<Album>,
}

/// Home view sections used by mobile clients.
#[derive(Serialize, Deserialize, Type, Clone, Debug, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct MobileHomeData {
    pub most_played: Vec<Song>,
    pub recently_played: Vec<Song>,
    pub recently_added: Vec<Album>,
    pub random_albums: Vec<Album>,
    pub featured_albums: Vec<Album>,
}

/// Sync state information for UI display
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct SyncStateInfo {
    pub last_sync_time: Option<String>,
    pub song_count: u32,
    pub artist_count: u32,
    pub album_count: u32,
}

/// Discord Rich Presence activity data
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct RpcActivity {
    pub state: Option<String>,
    pub details: Option<String>,
    pub large_image_key: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_key: Option<String>,
    pub small_image_text: Option<String>,
    pub start_timestamp: Option<i64>,
    pub end_timestamp: Option<i64>,
}

/// OS Now Playing metadata (SMTC / MPRIS / MPNowPlayingInfoCenter)
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingPayload {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub cover_url: Option<String>,
}

/// Last.fm credentials
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct LastFmCredentials {
    pub session_key: String,
    pub username: String,
    pub api_key: Option<String>,
}
