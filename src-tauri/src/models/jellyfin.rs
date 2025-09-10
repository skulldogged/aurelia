//! Jellyfin API-specific data models

use serde::{Deserialize, Serialize};
use specta::Type;

/// Jellyfin lyrics response
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct JellyfinLyrics {
    /// List of lyric lines with timestamps
    #[serde(rename = "Lyrics")]
    #[specta(rename = "lyrics")]
    pub lyrics: Vec<JellyfinLyricLine>,
}

/// Individual lyric line with optional timestamp
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct JellyfinLyricLine {
    /// Lyric text
    #[serde(rename = "Text")]
    #[specta(rename = "text")]
    pub text: String,
    /// Timestamp in ticks (100ns intervals from start)
    #[serde(rename = "Start")]
    #[specta(rename = "timestamp")]
    pub timestamp: Option<i64>,
}
