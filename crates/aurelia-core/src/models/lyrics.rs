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

impl From<aurelia_lyrics::models::ParsedLyricsWord> for ParsedLyricsWord {
    fn from(other: aurelia_lyrics::models::ParsedLyricsWord) -> Self {
        Self {
            time_ms: other.time_ms,
            end_time_ms: other.end_time_ms,
            word: other.word,
        }
    }
}

impl From<aurelia_lyrics::models::ParsedLyricsLine> for ParsedLyricsLine {
    fn from(other: aurelia_lyrics::models::ParsedLyricsLine) -> Self {
        Self {
            time_ms: other.time_ms,
            end_time_ms: other.end_time_ms,
            line: other.line,
            words: other.words.map(|w| w.into_iter().map(Into::into).collect()),
            agent_id: other.agent_id,
        }
    }
}

impl From<aurelia_lyrics::models::ParsedLyricsAgent> for ParsedLyricsAgent {
    fn from(other: aurelia_lyrics::models::ParsedLyricsAgent) -> Self {
        Self {
            id: other.id,
            agent_type: other.agent_type,
        }
    }
}

impl From<aurelia_lyrics::models::ParsedLyrics> for ParsedLyrics {
    fn from(other: aurelia_lyrics::models::ParsedLyrics) -> Self {
        let synced: Vec<ParsedLyricsLine> = other.synced.into_iter().map(Into::into).collect();

        // Convert sections by mapping indices to lines
        let sections = other.sections.map(|secs| {
            secs.into_iter()
                .map(|s| {
                    // Safe slicing with clamping
                    let start = s.start_line_index.min(synced.len());
                    let end = s.end_line_index.min(synced.len());
                    let lines_in_section = if start <= end {
                        synced[start..end].to_vec()
                    } else {
                        vec![]
                    };
                    
                    let start_time_ms = lines_in_section
                        .first()
                        .map(|l| l.time_ms)
                        .unwrap_or(0);
                        
                    let end_time_ms = lines_in_section
                        .last()
                        .and_then(|l| l.end_time_ms)
                        .or_else(|| lines_in_section.last().map(|l| l.time_ms))
                        .unwrap_or(start_time_ms);

                    ParsedLyricsSection {
                        name: s.name,
                        start_time_ms,
                        end_time_ms,
                        lines: lines_in_section,
                        agent_id: None, // Missing in source
                    }
                })
                .collect()
        });

        Self {
            plain: other.plain,
            synced,
            sections,
            agents: other.agents.map(|a| a.into_iter().map(Into::into).collect()),
            songwriters: other.songwriters,
            language: other.language,
            are_from_remote: true,
        }
    }
}
