//! LRC and plain text lyrics parsing

use crate::error::Result;
use crate::models::{ParsedLyrics, ParsedLyricsLine};
use regex::Regex;

/// Parse LRC format lyrics
pub fn parse_lrc(content: &str) -> Result<ParsedLyrics> {
    let mut lines = Vec::new();
    let mut plain_lines = Vec::new();

    // Regex for LRC timestamps [mm:ss.xx] or [mm:ss.xxx]
    let time_re = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\]").unwrap();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Extract all timestamps from the line
        let timestamps: Vec<_> = time_re.find_iter(line).collect();

        if timestamps.is_empty() {
            // Plain text line without timestamp
            plain_lines.push(line.to_string());
            continue;
        }

        // Get the text content (everything after the last timestamp)
        let last_ts_end = timestamps.last().map(|m| m.end()).unwrap_or(0);
        let text = line[last_ts_end..].trim().to_string();

        if text.is_empty() {
            continue;
        }

        plain_lines.push(text.clone());

        // Create a synced line for each timestamp
        for ts_match in &timestamps {
            let ts_str = &line[ts_match.start()..ts_match.end()];
            if let Some(caps) = time_re.captures(ts_str) {
                let minutes: i64 = caps[1].parse().unwrap_or(0);
                let seconds: i64 = caps[2].parse().unwrap_or(0);
                let millis: i64 = caps[3].parse().unwrap_or(0);
                let millis = if caps[3].len() == 2 {
                    millis * 10
                } else {
                    millis
                };

                let time_ms = minutes * 60_000 + seconds * 1_000 + millis;

                lines.push(ParsedLyricsLine {
                    time_ms,
                    end_time_ms: None,
                    line: text.clone(),
                    words: None,
                    agent_id: None,
                });
            }
        }
    }

    // Sort by timestamp
    lines.sort_by_key(|l| l.time_ms);

    Ok(ParsedLyrics {
        plain: plain_lines,
        synced: lines,
        sections: None,
        agents: None,
        songwriters: None,
        language: None,
    })
}

/// Parse plain text lyrics (no timestamps)
pub fn parse_plain_text(content: &str) -> ParsedLyrics {
    let plain: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    ParsedLyrics {
        plain: plain.clone(),
        synced: plain
            .into_iter()
            .enumerate()
            .map(|(i, line)| ParsedLyricsLine {
                time_ms: i as i64 * 5_000, // Dummy 5s spacing
                end_time_ms: None,
                line,
                words: None,
                agent_id: None,
            })
            .collect(),
        sections: None,
        agents: None,
        songwriters: None,
        language: None,
    }
}

/// Parse enhanced LRC format with word-level timing
/// Format: [mm:ss.xx]word [mm:ss.xx]word
pub fn parse_elrc(content: &str) -> Result<ParsedLyrics> {
    // For now, fall back to regular LRC parsing
    // Word-level parsing would require more complex regex
    parse_lrc(content)
}

/// Auto-detect format and parse
pub fn parse_auto(content: &str, extension: &str) -> Result<ParsedLyrics> {
    match extension.to_lowercase().as_str() {
        "lrc" => parse_lrc(content),
        "elrc" => parse_elrc(content),
        "txt" => Ok(parse_plain_text(content)),
        _ => {
            // Try to detect by content
            if content.contains('[') && content.contains(']') {
                parse_lrc(content)
            } else {
                Ok(parse_plain_text(content))
            }
        }
    }
}
