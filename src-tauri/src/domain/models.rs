use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncState {
    pub last_sync_time: String,
    pub last_sync_version: Option<String>,
    pub song_count: u32,
    pub artist_count: u32,
    pub album_count: u32,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync_time: "1970-01-01T00:00:00Z".to_string(),
            last_sync_version: None,
            song_count: 0,
            artist_count: 0,
            album_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncReport {
    pub full_sync: bool,
    pub songs_updated: u32,
    pub artists_updated: u32,
    pub albums_updated: u32,
    pub duration_ms: u64,
}
