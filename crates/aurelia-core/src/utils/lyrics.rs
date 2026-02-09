//! Lyrics format conversion utilities

use crate::models::{JellyfinLyrics, ParsedLyrics, ParsedLyricsLine, ParsedLyricsWord};
use std::fmt::Write;

/// Convert 100ns ticks to milliseconds.
fn ticks_to_ms(ticks: f64) -> i64 {
    (ticks / 10_000.0).round() as i64
}

/// Convert 100ns ticks (i64) to milliseconds.
fn ticks_i64_to_ms(ticks: i64) -> i64 {
    (ticks as f64 / 10_000.0).round() as i64
}

/// Convert Jellyfin lyrics format to LRC format
///
/// Jellyfin uses 100ns ticks for timestamps, this converts to standard LRC format [MM:SS.mmm]
pub fn jellyfin_to_lrc(lyrics: &JellyfinLyrics) -> Result<String, std::fmt::Error> {
    let mut lrc_content = String::new();

    for line in &lyrics.lyrics {
        if let Some(timestamp) = line.timestamp {
            // Convert Jellyfin timestamp (100ns ticks) to LRC format (MM:SS.mmm)
            let total_seconds = timestamp / 10_000_000.0;
            let total_seconds_floor = total_seconds.floor();
            let minutes = (total_seconds_floor / 60.0).floor();
            let seconds = (total_seconds_floor % 60.0).floor();
            let milliseconds = ((timestamp % 10_000_000.0) / 10_000.0).floor();

            writeln!(
                lrc_content,
                "[{:02}:{:02}.{:03}] {}",
                minutes, seconds, milliseconds, line.text
            )?;
        } else {
            writeln!(lrc_content, "{}", line.text)?;
        }
    }

    Ok(lrc_content)
}

/// Extract word-level [`ParsedLyricsWord`] entries from Jellyfin `LyricLineCue` data.
///
/// Each cue provides character-position indices (`Position`..`EndPosition`) into
/// the line's `Text` together with start/end tick timestamps.
fn extract_words_from_cues(
    text: &str,
    cues: &[crate::models::JellyfinLyricLineCue],
) -> Option<Vec<ParsedLyricsWord>> {
    if cues.is_empty() {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut words = Vec::with_capacity(cues.len());

    for cue in cues {
        let start = cue.position.max(0) as usize;
        let end = (cue.end_position.max(0) as usize).min(chars.len());
        let word: String = if start < end {
            chars[start..end].iter().collect()
        } else {
            String::new()
        };

        // Skip empty cues (pure whitespace separators)
        let trimmed = word.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        words.push(ParsedLyricsWord {
            time_ms: ticks_i64_to_ms(cue.start),
            end_time_ms: cue.end.map(ticks_i64_to_ms),
            word: trimmed,
        });
    }

    if words.is_empty() { None } else { Some(words) }
}

/// Convert Jellyfin lyrics directly to [`ParsedLyrics`] without an intermediate LRC string.
///
/// Jellyfin timestamps are in 100ns tick units.  If the server parsed a TTML
/// file the response will include `Cues` on each line with word-level timing
/// data — these are converted to [`ParsedLyricsWord`] entries.
#[must_use]
pub fn jellyfin_to_parsed_lyrics(lyrics: &JellyfinLyrics) -> ParsedLyrics {
    let mut synced = Vec::new();
    let mut plain = Vec::new();
    let mut has_timestamps = false;

    for (i, line) in lyrics.lyrics.iter().enumerate() {
        let text = line.text.trim().to_string();
        if let Some(ticks) = line.timestamp {
            has_timestamps = true;
            let time_ms = ticks_to_ms(ticks);

            // Try to extract word-level cues
            let words = line
                .cues
                .as_deref()
                .and_then(|cues| extract_words_from_cues(&line.text, cues));

            // Derive line end_time_ms:
            //   1. Last word's end_time if we have word cues
            //   2. Otherwise next line's start time
            //   3. Otherwise None
            let end_from_words = words
                .as_ref()
                .and_then(|w| w.last())
                .and_then(|w| w.end_time_ms);

            let end_from_next = lyrics
                .lyrics
                .get(i + 1)
                .and_then(|next| next.timestamp)
                .map(ticks_to_ms);

            let end_time_ms = end_from_words.or(end_from_next);

            synced.push(ParsedLyricsLine {
                time_ms,
                end_time_ms,
                line: text,
                words,
                agent_id: None,
            });
        } else {
            plain.push(text);
        }
    }

    if has_timestamps && !synced.is_empty() {
        synced.sort_by_key(|l| l.time_ms);
        let plain_from_synced = synced.iter().map(|l| l.line.clone()).collect();
        ParsedLyrics {
            plain: plain_from_synced,
            synced,
            sections: None,
            agents: None,
            songwriters: None,
            language: None,
            are_from_remote: true,
        }
    } else {
        ParsedLyrics {
            plain: plain.into_iter().filter(|l| !l.is_empty()).collect(),
            synced: vec![],
            sections: None,
            agents: None,
            songwriters: None,
            language: None,
            are_from_remote: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{jellyfin_to_lrc, jellyfin_to_parsed_lyrics};
    use crate::models::{JellyfinLyricLine, JellyfinLyricLineCue, JellyfinLyrics};

    fn make_line(text: &str, ticks: Option<f64>) -> JellyfinLyricLine {
        JellyfinLyricLine {
            text: text.to_string(),
            timestamp: ticks,
            cues: None,
        }
    }

    #[test]
    fn converts_jellyfin_timestamps_to_lrc() {
        let lyrics = JellyfinLyrics {
            metadata: None,
            lyrics: vec![
                make_line("Intro", Some(0.0)),
                make_line("Verse", Some(12_340_000.0)),
                make_line("No timestamp", None),
            ],
        };

        let rendered = jellyfin_to_lrc(&lyrics).expect("lrc render");
        let lines: Vec<&str> = rendered.lines().collect();

        assert_eq!(lines[0], "[00:00.000] Intro");
        assert_eq!(lines[1], "[00:01.234] Verse");
        assert_eq!(lines[2], "No timestamp");
    }

    #[test]
    fn parsed_lyrics_without_cues_have_no_words() {
        let lyrics = JellyfinLyrics {
            metadata: None,
            lyrics: vec![
                make_line("Hello world", Some(0.0)),
                make_line("Goodbye world", Some(50_000_000.0)),
            ],
        };

        let parsed = jellyfin_to_parsed_lyrics(&lyrics);
        assert_eq!(parsed.synced.len(), 2);
        assert!(parsed.synced[0].words.is_none());
        assert!(parsed.synced[1].words.is_none());
        // end_time_ms should be derived from next line
        assert_eq!(parsed.synced[0].end_time_ms, Some(5000));
        assert!(parsed.synced[1].end_time_ms.is_none());
    }

    #[test]
    fn parsed_lyrics_extracts_word_cues() {
        // "Hello world" — 'Hello' at chars 0..5, 'world' at chars 6..11
        let lyrics = JellyfinLyrics {
            metadata: None,
            lyrics: vec![JellyfinLyricLine {
                text: "Hello world".to_string(),
                timestamp: Some(0.0),
                cues: Some(vec![
                    JellyfinLyricLineCue {
                        position: 0,
                        end_position: 5,
                        start: 0,
                        end: Some(5_000_000), // 500ms
                    },
                    JellyfinLyricLineCue {
                        position: 6,
                        end_position: 11,
                        start: 5_000_000,
                        end: Some(10_000_000), // 1000ms
                    },
                ]),
            }],
        };

        let parsed = jellyfin_to_parsed_lyrics(&lyrics);
        assert_eq!(parsed.synced.len(), 1);

        let words = parsed.synced[0].words.as_ref().expect("should have words");
        assert_eq!(words.len(), 2);

        assert_eq!(words[0].word, "Hello");
        assert_eq!(words[0].time_ms, 0);
        assert_eq!(words[0].end_time_ms, Some(500));

        assert_eq!(words[1].word, "world");
        assert_eq!(words[1].time_ms, 500);
        assert_eq!(words[1].end_time_ms, Some(1000));

        // Line end_time_ms should come from last word's end
        assert_eq!(parsed.synced[0].end_time_ms, Some(1000));
    }

    #[test]
    fn whitespace_only_cues_are_skipped() {
        let lyrics = JellyfinLyrics {
            metadata: None,
            lyrics: vec![JellyfinLyricLine {
                text: "Hi there".to_string(),
                timestamp: Some(0.0),
                cues: Some(vec![
                    JellyfinLyricLineCue {
                        position: 0,
                        end_position: 2,
                        start: 0,
                        end: Some(2_000_000),
                    },
                    // Whitespace-only cue (the space between words)
                    JellyfinLyricLineCue {
                        position: 2,
                        end_position: 3,
                        start: 2_000_000,
                        end: Some(3_000_000),
                    },
                    JellyfinLyricLineCue {
                        position: 3,
                        end_position: 8,
                        start: 3_000_000,
                        end: Some(6_000_000),
                    },
                ]),
            }],
        };

        let parsed = jellyfin_to_parsed_lyrics(&lyrics);
        let words = parsed.synced[0].words.as_ref().expect("should have words");
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].word, "Hi");
        assert_eq!(words[1].word, "there");
    }

    #[test]
    fn empty_cues_array_produces_no_words() {
        let lyrics = JellyfinLyrics {
            metadata: None,
            lyrics: vec![JellyfinLyricLine {
                text: "No cues here".to_string(),
                timestamp: Some(0.0),
                cues: Some(vec![]),
            }],
        };

        let parsed = jellyfin_to_parsed_lyrics(&lyrics);
        assert!(parsed.synced[0].words.is_none());
    }
}
