//! LrcLib API data models

use serde::{Deserialize, Serialize};
use specta::Type;

/// LrcLib track response containing lyrics
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct LrcLibTrackResponse {
    /// Track ID in LrcLib
    pub id: i32,
    /// Track name
    pub name: String,
    /// Alternative track name
    #[serde(rename = "trackName")]
    pub track_name: String,
    /// Artist name
    #[serde(rename = "artistName")]
    pub artist_name: String,
    /// Album name
    pub album_name: Option<String>,
    /// Track duration in seconds
    pub duration: f64,
    /// Whether this is an instrumental track
    pub instrumental: bool,
    /// Plain text lyrics (no timestamps)
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,
    /// Synchronized lyrics with timestamps
    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
}
