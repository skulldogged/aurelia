//! TTML (Timed Text Markup Language) parser
//! Supports Apple Music-style TTML with word-level timing

use crate::error::{LyricsError, Result};
use crate::models::{
    ParsedLyrics, ParsedLyricsAgent, ParsedLyricsLine, ParsedLyricsSection, ParsedLyricsWord,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// Parse TTML XML content
pub fn parse_ttml(content: &str) -> Result<ParsedLyrics> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut lines = Vec::new();
    let mut sections = Vec::new();
    let mut agents = HashMap::new();
    let mut songwriters = Vec::new();
    let mut language = None;

    let mut current_line = Option::<LineBuilder>::None;
    let mut current_words = Vec::new();
    let mut in_lyrics = false;
    let mut line_index = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|e| LyricsError::Xml(format!("Invalid UTF-8: {e}")))?;

                match name {
                    "tt" => {
                        // Extract xml:lang attribute
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = std::str::from_utf8(&attr.key.as_ref())
                                    .map_err(|e| LyricsError::Xml(format!("Invalid UTF-8: {e}")))?;
                                if key == "xml:lang" {
                                    language = attr.unescape_value().ok().map(|v| v.to_string());
                                }
                            }
                        }
                    }
                    "p" => {
                        // Start of a line
                        let mut begin = None;
                        let mut end = None;
                        let mut agent = None;

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = std::str::from_utf8(&attr.key.as_ref())
                                    .map_err(|e| LyricsError::Xml(format!("Invalid UTF-8: {e}")))?;
                                let value = attr.unescape_value().map_err(|e| {
                                    LyricsError::Xml(format!("Invalid attribute value: {e}"))
                                })?;

                                match key {
                                    "begin" => begin = parse_time(&value),
                                    "end" => end = parse_time(&value),
                                    "ttm:agent" => agent = Some(value.to_string()),
                                    _ => {}
                                }
                            }
                        }

                        if let Some(begin) = begin {
                            current_line = Some(LineBuilder {
                                begin,
                                end,
                                text: String::new(),
                                words: Vec::new(),
                                agent,
                            });
                            in_lyrics = true;
                        }
                    }
                    "span" => {
                        // Word with timing
                        if in_lyrics {
                            let mut begin = None;
                            let mut end = None;

                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let key =
                                        std::str::from_utf8(&attr.key.as_ref()).map_err(|e| {
                                            LyricsError::Xml(format!("Invalid UTF-8: {e}"))
                                        })?;
                                    let value = attr.unescape_value().map_err(|e| {
                                        LyricsError::Xml(format!("Invalid attribute value: {e}"))
                                    })?;

                                    match key {
                                        "begin" => begin = parse_time(&value),
                                        "end" => end = parse_time(&value),
                                        _ => {}
                                    }
                                }
                            }

                            current_words.push(WordBuilder {
                                begin,
                                end,
                                text: String::new(),
                            });
                        }
                    }
                    "div" => {
                        // Check for songPart (sections)
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = std::str::from_utf8(&attr.key.as_ref())
                                    .map_err(|e| LyricsError::Xml(format!("Invalid UTF-8: {e}")))?;
                                if key == "itunes:songPart" {
                                    let value = attr.unescape_value().map_err(|e| {
                                        LyricsError::Xml(format!("Invalid attribute value: {e}"))
                                    })?;
                                    sections.push(ParsedLyricsSection {
                                        name: value.to_string(),
                                        start_line_index: line_index,
                                        end_line_index: line_index, // Will update later
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| LyricsError::Xml(format!("Invalid text: {e}")))?
                    .to_string();

                if !text.trim().is_empty() {
                    if let Some(ref mut word) = current_words.last_mut() {
                        word.text.push_str(&text);
                    } else if let Some(ref mut line) = current_line {
                        line.text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|e| LyricsError::Xml(format!("Invalid UTF-8: {e}")))?;

                match name {
                    "p" => {
                        if let Some(line_builder) = current_line.take() {
                            let words = if current_words.is_empty() {
                                None
                            } else {
                                Some(
                                    current_words
                                        .drain(..)
                                        .filter(|w| !w.text.is_empty())
                                        .map(|w| ParsedLyricsWord {
                                            time_ms: w.begin.unwrap_or(0),
                                            end_time_ms: w.end,
                                            word: w.text.trim().to_string(),
                                        })
                                        .collect(),
                                )
                            };

                            lines.push(ParsedLyricsLine {
                                time_ms: line_builder.begin,
                                end_time_ms: line_builder.end,
                                line: line_builder.text.trim().to_string(),
                                words,
                                agent_id: line_builder.agent,
                            });
                            line_index += 1;
                            in_lyrics = false;
                        }
                    }
                    "span" => {
                        // Word ended, add to current line
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(LyricsError::Xml(format!("XML parsing error: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    // Update section end indices
    for i in 0..sections.len() {
        if i + 1 < sections.len() {
            sections[i].end_line_index = sections[i + 1].start_line_index;
        } else {
            sections[i].end_line_index = line_index;
        }
    }

    let plain: Vec<String> = lines.iter().map(|l| l.line.clone()).collect();

    Ok(ParsedLyrics {
        plain,
        synced: lines,
        sections: if sections.is_empty() {
            None
        } else {
            Some(sections)
        },
        agents: if agents.is_empty() {
            None
        } else {
            Some(
                agents
                    .into_iter()
                    .map(|(id, name)| ParsedLyricsAgent { id, name })
                    .collect(),
            )
        },
        songwriters: if songwriters.is_empty() {
            None
        } else {
            Some(songwriters)
        },
        language,
    })
}

fn parse_time(time_str: &str) -> Option<i64> {
    // Parse formats like "30.348s" or "00:30.348"
    if time_str.ends_with('s') {
        time_str[..time_str.len() - 1]
            .parse::<f64>()
            .ok()
            .map(|s| (s * 1000.0) as i64)
    } else if time_str.contains(':') {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 2 {
            let mins: i64 = parts[0].parse().ok()?;
            let secs: f64 = parts[1].parse().ok()?;
            Some(mins * 60_000 + (secs * 1000.0) as i64)
        } else {
            None
        }
    } else {
        time_str.parse::<f64>().ok().map(|s| (s * 1000.0) as i64)
    }
}

struct LineBuilder {
    begin: i64,
    end: Option<i64>,
    text: String,
    words: Vec<ParsedLyricsWord>,
    agent: Option<String>,
}

struct WordBuilder {
    begin: Option<i64>,
    end: Option<i64>,
    text: String,
}
