//! Parsed lyrics models shared across clients.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Word-level synchronized lyric entry.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyricsWord {
    pub time_ms: i64,
    pub word: String,
}

/// Line-level synchronized lyric entry.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyricsLine {
    pub time_ms: i64,
    pub line: String,
    pub words: Option<Vec<ParsedLyricsWord>>,
}

/// Parsed lyrics payload returned by shared parser.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyrics {
    pub plain: Vec<String>,
    pub synced: Vec<ParsedLyricsLine>,
    pub are_from_remote: bool,
}

impl ParsedLyrics {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.synced.is_empty() || !self.plain.is_empty()
    }
}
