//! Jellyfin API-specific data models

use serde::{Deserialize, Serialize};

/// Jellyfin lyrics response
#[derive(Serialize, Deserialize, Debug)]
pub struct JellyfinLyrics {
    /// List of lyric lines with timestamps
    #[serde(rename = "Lyrics")]
    pub lyrics: Vec<JellyfinLyricLine>,
}

/// Individual lyric line with optional timestamp
#[derive(Serialize, Deserialize, Debug)]
pub struct JellyfinLyricLine {
    /// Lyric text
    #[serde(rename = "Text")]
    pub text: String,
    /// Timestamp in ticks (100ns intervals from start)
    #[serde(rename = "Start")]
    pub timestamp: Option<i64>,
}
