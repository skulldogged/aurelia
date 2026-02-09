//! Parsed lyrics models shared across clients.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Word-level synchronized lyric entry.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyricsWord {
    pub time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub word: String,
}

/// Line-level synchronized lyric entry.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyricsLine {
    pub time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub line: String,
    pub words: Option<Vec<ParsedLyricsWord>>,
    /// Agent/singer identifier (e.g. "v1", "v2000") for multi-singer attribution.
    pub agent_id: Option<String>,
}

/// A named section of the song (e.g. Verse, Chorus, Bridge).
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyricsSection {
    /// Section name (e.g. "Verse", "Chorus", "Bridge", "Intro", "Outro").
    pub name: String,
    /// Start time in milliseconds.
    pub start_time_ms: i64,
    /// End time in milliseconds.
    pub end_time_ms: i64,
    /// Lines belonging to this section.
    pub lines: Vec<ParsedLyricsLine>,
    /// Default agent for this section, if any.
    pub agent_id: Option<String>,
}

/// A singer/performer agent definition.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyricsAgent {
    /// Unique identifier (e.g. "v1", "v2000").
    pub id: String,
    /// Agent type: "person" for a singer, "other" for background/samples.
    pub agent_type: String,
}

/// Parsed lyrics payload returned by shared parser.
#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyrics {
    pub plain: Vec<String>,
    pub synced: Vec<ParsedLyricsLine>,
    pub sections: Option<Vec<ParsedLyricsSection>>,
    pub agents: Option<Vec<ParsedLyricsAgent>>,
    pub songwriters: Option<Vec<String>>,
    pub language: Option<String>,
    pub are_from_remote: bool,
}

impl ParsedLyrics {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.synced.is_empty() || !self.plain.is_empty()
    }
}
