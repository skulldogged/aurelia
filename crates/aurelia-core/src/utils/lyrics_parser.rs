//! Shared lyrics parser used by web/desktop/mobile clients.
//!
//! Supports LRC and TTML formats with automatic detection via [`parse_lyrics`].

use crate::models::{ParsedLyrics, ParsedLyricsLine, ParsedLyricsWord};
use crate::utils::ttml_parser;
use once_cell::sync::Lazy;
use regex::Regex;

/// Parse lyrics text, auto-detecting the format (TTML or LRC/plain).
#[must_use]
pub fn parse_lyrics(lyrics_text: &str) -> ParsedLyrics {
    if ttml_parser::is_ttml(lyrics_text) {
        ttml_parser::parse_ttml_lyrics(lyrics_text)
    } else {
        parse_lrc_lyrics(lyrics_text)
    }
}

static LRC_LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\[(\d{1,3}):(\d{2})\.(\d{2,3})\](.*)$").expect("valid line regex"));
static LRC_WORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<(\d{1,3}):(\d{2})\.(\d{2,3})>([^<]*)").expect("valid word regex"));
static LRC_WORD_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<\d{1,3}:\d{2}\.\d{2,3}>").expect("valid word tag regex"));
static LRC_TIMESTAMP_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[\d{1,3}:\d{2}(?:\.\d{1,3})?]").expect("valid timestamp regex"));

fn parse_timestamp_to_ms(minutes: &str, seconds: &str, fraction: &str) -> i64 {
    let minutes = minutes.parse::<i64>().unwrap_or(0);
    let seconds = seconds.parse::<i64>().unwrap_or(0);
    let fraction_len = fraction.len();
    let fraction = fraction.parse::<i64>().unwrap_or(0);
    let millis = if fraction_len == 2 {
        fraction * 10
    } else {
        fraction
    };
    minutes * 60 * 1000 + seconds * 1000 + millis
}

fn strip_lrc_timestamps(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    let without_tags = LRC_TIMESTAMP_TAG_REGEX.replace_all(value, "");
    without_tags.trim_start().to_string()
}

fn sanitize_lrc_line(raw_line: &str) -> String {
    if raw_line.is_empty() {
        return raw_line.to_string();
    }

    let without_controls: String = raw_line
        .trim_end_matches(&['\r', '\n'][..])
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect();

    let without_bom = without_controls.trim_end_matches('\u{FEFF}');
    let trimmed_prefix = without_bom.trim_start();

    if let Some(first_bracket) = trimmed_prefix.find('[')
        && first_bracket > 0
    {
        return trimmed_prefix[first_bracket..].to_string();
    }

    trimmed_prefix.to_string()
}

fn strip_format_characters(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect();

    match cleaned.as_str() {
        "\"" | "'" => String::new(),
        _ => cleaned,
    }
}

/// Parse LRC or plain lyrics text into structured lyrics.
#[must_use]
pub fn parse_lrc_lyrics(lyrics_text: &str) -> ParsedLyrics {
    if lyrics_text.trim().is_empty() {
        return ParsedLyrics {
            plain: vec![],
            synced: vec![],
            sections: None,
            agents: None,
            songwriters: None,
            language: None,
            are_from_remote: true,
        };
    }

    let mut synced_lines: Vec<ParsedLyricsLine> = Vec::new();
    let mut plain_lines: Vec<String> = Vec::new();
    let mut is_synced = false;

    for raw_line in lyrics_text.lines() {
        let line = sanitize_lrc_line(raw_line);
        if line.is_empty() {
            continue;
        }

        if let Some(caps) = LRC_LINE_REGEX.captures(&line) {
            is_synced = true;
            let line_timestamp = parse_timestamp_to_ms(&caps[1], &caps[2], &caps[3]);
            let text_with_tags =
                strip_format_characters(caps.get(4).map_or("", |m| m.as_str()).trim());
            let text = strip_lrc_timestamps(&text_with_tags);

            if LRC_WORD_TAG_REGEX.is_match(&text) {
                let mut words: Vec<ParsedLyricsWord> = Vec::new();
                let mut cursor = 0usize;

                for captures in LRC_WORD_REGEX.captures_iter(&text) {
                    let Some(matched) = captures.get(0) else {
                        continue;
                    };

                    if matched.start() > cursor {
                        let untagged = &text[cursor..matched.start()];
                        if !untagged.is_empty() {
                            let fallback_time = words
                                .last()
                                .map(|word| word.time_ms)
                                .unwrap_or(line_timestamp);
                            words.push(ParsedLyricsWord {
                                time_ms: fallback_time,
                                end_time_ms: None,
                                word: untagged.to_string(),
                            });
                        }
                    }

                    let word_timestamp =
                        parse_timestamp_to_ms(&captures[1], &captures[2], &captures[3]);
                    let word_text =
                        strip_format_characters(captures.get(4).map_or("", |m| m.as_str()));

                    words.push(ParsedLyricsWord {
                        time_ms: word_timestamp,
                        end_time_ms: None,
                        word: word_text,
                    });

                    cursor = matched.end();
                }

                if cursor < text.len() {
                    let trailing = &text[cursor..];
                    if !trailing.is_empty() {
                        let fallback_time = words
                            .last()
                            .map(|word| word.time_ms)
                            .unwrap_or(line_timestamp);
                        words.push(ParsedLyricsWord {
                            time_ms: fallback_time,
                            end_time_ms: None,
                            word: trailing.to_string(),
                        });
                    }
                }

                if words.is_empty() {
                    synced_lines.push(ParsedLyricsLine {
                        time_ms: line_timestamp,
                        end_time_ms: None,
                        line: text,
                        words: None,
                        agent_id: None,
                    });
                } else {
                    let full_line_text = words.iter().map(|word| word.word.as_str()).collect();
                    synced_lines.push(ParsedLyricsLine {
                        time_ms: line_timestamp,
                        end_time_ms: None,
                        line: full_line_text,
                        words: Some(words),
                        agent_id: None,
                    });
                }
            } else {
                synced_lines.push(ParsedLyricsLine {
                    time_ms: line_timestamp,
                    end_time_ms: None,
                    line: text,
                    words: None,
                    agent_id: None,
                });
            }
        } else {
            let stripped = strip_lrc_timestamps(&strip_format_characters(&line));

            if is_synced && !synced_lines.is_empty() {
                let last_index = synced_lines.len() - 1;
                let existing = synced_lines[last_index].line.clone();
                synced_lines[last_index].line = if existing.is_empty() {
                    stripped
                } else {
                    format!("{existing}\n{stripped}")
                };
            } else {
                plain_lines.push(stripped);
            }
        }
    }

    if is_synced && !synced_lines.is_empty() {
        synced_lines.sort_by_key(|line| line.time_ms);
        let plain = synced_lines.iter().map(|line| line.line.clone()).collect();
        ParsedLyrics {
            plain,
            synced: synced_lines,
            sections: None,
            agents: None,
            songwriters: None,
            language: None,
            are_from_remote: true,
        }
    } else {
        ParsedLyrics {
            plain: plain_lines
                .into_iter()
                .filter(|line| !line.is_empty())
                .collect(),
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
    use super::{parse_lrc_lyrics, parse_lyrics};

    #[test]
    fn parse_lyrics_dispatches_to_ttml() {
        let ttml = r#"<?xml version="1.0" ?>
<tt xmlns="http://www.w3.org/ns/ttml" itunes:timing="Line" xml:lang="en">
  <head><metadata/></head>
  <body dur="1:00.000">
    <div begin="0.0" end="5.0">
      <p begin="0.0" end="2.0">Hello TTML</p>
    </div>
  </body>
</tt>"#;
        let parsed = parse_lyrics(ttml);
        assert_eq!(parsed.synced.len(), 1);
        assert_eq!(parsed.synced[0].line, "Hello TTML");
        assert!(parsed.synced[0].end_time_ms.is_some());
    }

    #[test]
    fn parse_lyrics_dispatches_to_lrc() {
        let parsed = parse_lyrics("[00:01.50]Hello LRC");
        assert_eq!(parsed.synced.len(), 1);
        assert_eq!(parsed.synced[0].line, "Hello LRC");
        assert!(parsed.synced[0].end_time_ms.is_none());
    }

    #[test]
    fn parse_lyrics_dispatches_plain() {
        let parsed = parse_lyrics("just plain text");
        assert!(parsed.synced.is_empty());
        assert_eq!(parsed.plain, vec!["just plain text"]);
    }

    #[test]
    fn parses_plain_lyrics() {
        let parsed = parse_lrc_lyrics("line one\nline two");
        assert!(parsed.synced.is_empty());
        assert_eq!(
            parsed.plain,
            vec!["line one".to_string(), "line two".to_string()]
        );
    }

    #[test]
    fn parses_line_synced_lyrics() {
        let parsed = parse_lrc_lyrics("[00:01.50]Hello\n[00:03.000]World");
        assert_eq!(parsed.synced.len(), 2);
        assert_eq!(parsed.synced[0].time_ms, 1500);
        assert_eq!(parsed.synced[1].time_ms, 3000);
        assert_eq!(parsed.synced[1].line, "World");
    }

    #[test]
    fn parses_word_synced_lyrics() {
        let parsed = parse_lrc_lyrics("[00:10.00]<00:10.00>Hello <00:10.30>world");
        assert_eq!(parsed.synced.len(), 1);
        let words = parsed.synced[0].words.clone().expect("word synced lyrics");
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].time_ms, 10000);
        assert_eq!(words[0].word, "Hello ");
        assert_eq!(words[1].time_ms, 10300);
        assert_eq!(words[1].word, "world");
    }

    #[test]
    fn tolerates_invalid_lines() {
        let parsed = parse_lrc_lyrics("junk\n[xx:yy.zz]bad\n[00:05.00]good");
        assert_eq!(parsed.synced.len(), 1);
        assert_eq!(parsed.synced[0].line, "good");
    }
}
