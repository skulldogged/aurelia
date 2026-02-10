use aurelia_lyrics::models::ParsedLyrics;
use serde::Deserialize;

/// Response from the Sidecar Lyrics Daemon
#[derive(Debug, Clone, Deserialize)]
pub struct LyricsDaemonResponse {
    pub item_id: String,
    pub found: bool,
    pub source: Option<String>,
    pub lyrics: Option<ParsedLyrics>,
}
