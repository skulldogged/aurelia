use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyrics {
    pub plain: Vec<String>,
    pub synced: Vec<ParsedLyricsLine>,
    pub sections: Option<Vec<ParsedLyricsSection>>,
    pub agents: Option<Vec<ParsedLyricsAgent>>,
    pub songwriters: Option<Vec<String>>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyricsLine {
    pub time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub line: String,
    pub words: Option<Vec<ParsedLyricsWord>>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyricsWord {
    pub time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub word: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyricsSection {
    pub name: String,
    pub start_line_index: usize,
    pub end_line_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyricsAgent {
    pub id: String,
    pub agent_type: String,
}

impl ParsedLyrics {
    pub fn is_empty(&self) -> bool {
        self.plain.is_empty() && self.synced.is_empty()
    }
}
