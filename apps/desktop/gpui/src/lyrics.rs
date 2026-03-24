use aurelia_core::models::{ParsedLyrics, ParsedLyricsLine, ParsedLyricsSection, ParsedLyricsWord};
use std::time::Instant;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LyricsUiState {
    pub request_nonce: u64,
    pub loaded_song_id: Option<String>,
    pub active_line_index: Option<usize>,
    pub active_word_index: Option<usize>,
    pub active_word_progress: f32,
    pub last_auto_scrolled_line: Option<usize>,
    pub sampled_position_secs: f64,
    pub sampled_at: Option<Instant>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LyricsWordRender {
    pub text: String,
    pub trailing_whitespace: String,
    pub state: LyricsWordState,
    pub progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LyricsWordState {
    Upcoming,
    Filling,
    Sung,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LyricsLineRender<'a> {
    pub line: &'a ParsedLyricsLine,
    pub section_label: Option<&'a str>,
    pub is_active: bool,
    pub is_background_vocal: bool,
    pub translation: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveLyricsState {
    pub line_index: Option<usize>,
    pub word_index: Option<usize>,
    pub word_progress: f32,
}

pub fn next_request_nonce(state: &mut LyricsUiState) -> u64 {
    state.request_nonce = state.request_nonce.wrapping_add(1);
    state.request_nonce
}

pub fn reset_for_song(state: &mut LyricsUiState, song_id: Option<String>) {
    state.loaded_song_id = song_id;
    state.active_line_index = None;
    state.active_word_index = None;
    state.active_word_progress = 0.0;
    state.last_auto_scrolled_line = None;
    state.sampled_position_secs = 0.0;
    state.sampled_at = None;
}

pub fn effective_position_secs(state: &LyricsUiState, is_playing: bool) -> f64 {
    if !is_playing {
        return state.sampled_position_secs;
    }

    let elapsed = state
        .sampled_at
        .map(|sampled_at| sampled_at.elapsed().as_secs_f64())
        .unwrap_or(0.0);

    (state.sampled_position_secs + elapsed).max(0.0)
}

pub fn compute_active_state(lyrics: &ParsedLyrics, current_time_secs: f64) -> ActiveLyricsState {
    let line_index = active_line_index(&lyrics.synced, current_time_secs);
    let (word_index, word_progress) = line_index
        .and_then(|idx| lyrics.synced.get(idx))
        .map(|line| active_word_state(line, current_time_secs))
        .unwrap_or((None, 0.0));

    ActiveLyricsState {
        line_index,
        word_index,
        word_progress,
    }
}

pub fn section_label_for_line<'a>(
    sections: Option<&'a [ParsedLyricsSection]>,
    line: &ParsedLyricsLine,
) -> Option<&'a str> {
    sections?.iter().find_map(|section| {
        section
            .lines
            .first()
            .filter(|first| first.time_ms == line.time_ms)
            .map(|_| section.name.as_str())
    })
}

pub fn is_background_vocal(lyrics: &ParsedLyrics, line: &ParsedLyricsLine) -> bool {
    let Some(agent_id) = line.agent_id.as_deref() else {
        return false;
    };

    lyrics
        .agents
        .as_ref()
        .and_then(|agents| agents.iter().find(|agent| agent.id == agent_id))
        .is_some_and(|agent| agent.agent_type == "other")
}

pub fn render_line<'a>(
    lyrics: &'a ParsedLyrics,
    index: usize,
    active: &ActiveLyricsState,
) -> Option<LyricsLineRender<'a>> {
    let line = lyrics.synced.get(index)?;
    Some(LyricsLineRender {
        line,
        section_label: section_label_for_line(lyrics.sections.as_deref(), line),
        is_active: active.line_index == Some(index),
        is_background_vocal: is_background_vocal(lyrics, line),
        translation: line.translation.as_deref().filter(|text| !text.trim().is_empty()),
    })
}

pub fn render_words(line: &ParsedLyricsLine, active: &ActiveLyricsState, line_index: usize) -> Vec<LyricsWordRender> {
    let Some(words) = line.words.as_ref() else {
        return Vec::new();
    };

    words
        .iter()
        .enumerate()
        .map(|(word_index, word)| {
            let (state, progress) = if active.line_index == Some(line_index) {
                match active.word_index {
                    Some(active_word) if word_index < active_word => (LyricsWordState::Sung, 1.0),
                    Some(active_word) if word_index == active_word => {
                        (LyricsWordState::Filling, active.word_progress)
                    }
                    _ => (LyricsWordState::Upcoming, 0.0),
                }
            } else {
                (LyricsWordState::Upcoming, 0.0)
            };

            LyricsWordRender {
                text: word.word.clone(),
                trailing_whitespace: trailing_whitespace(words, word_index),
                state,
                progress,
            }
        })
        .collect()
}

fn trailing_whitespace(words: &[ParsedLyricsWord], index: usize) -> String {
    let current = words[index].word.as_str();
    let next = words.get(index + 1).map(|word| word.word.as_str());

    let keep_attached = matches!(current, "(" | "[" | "{" | "\u{201c}" | "\u{2018}" | "/")
        || next.is_some_and(|next| matches!(next, ")" | "]" | "}" | "," | "." | "!" | "?" | ":" | ";" | "\u{201d}" | "\u{2019}"));

    if keep_attached || index + 1 >= words.len() {
        String::new()
    } else {
        " ".to_string()
    }
}

fn active_line_index(lines: &[ParsedLyricsLine], current_time_secs: f64) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }

    let time_ms = seconds_to_ms(current_time_secs);

    for (index, line) in lines.iter().enumerate().rev() {
        if line.time_ms <= time_ms {
            if let Some(end_time_ms) = line.end_time_ms
                && time_ms > end_time_ms
                && index + 1 < lines.len()
                && lines[index + 1].time_ms <= time_ms
            {
                continue;
            }
            return Some(index);
        }
    }

    None
}

fn active_word_state(line: &ParsedLyricsLine, current_time_secs: f64) -> (Option<usize>, f32) {
    let Some(words) = line.words.as_ref() else {
        return (None, 0.0);
    };
    if words.is_empty() {
        return (None, 0.0);
    }

    let time_ms = seconds_to_ms(current_time_secs);

    for index in (0..words.len()).rev() {
        if words[index].time_ms <= time_ms {
            let progress = word_progress(words, line, index, time_ms);
            return (Some(index), progress);
        }
    }

    (None, 0.0)
}

fn word_progress(words: &[ParsedLyricsWord], line: &ParsedLyricsLine, index: usize, time_ms: i64) -> f32 {
    let word = &words[index];
    let end_time_ms = word.end_time_ms.or_else(|| words.get(index + 1).map(|next| next.time_ms)).or(line.end_time_ms).unwrap_or(word.time_ms + 500);
    let duration_ms = (end_time_ms - word.time_ms).max(1) as f32;
    let elapsed_ms = (time_ms - word.time_ms).max(0) as f32;
    (elapsed_ms / duration_ms).clamp(0.0, 1.0)
}

fn seconds_to_ms(seconds: f64) -> i64 {
    ((seconds + 0.01) * 1000.0).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(time_ms: i64, end_time_ms: Option<i64>, text: &str) -> ParsedLyricsLine {
        ParsedLyricsLine {
            time_ms,
            end_time_ms,
            line: text.to_string(),
            words: None,
            agent_id: None,
            translation: None,
        }
    }

    fn word(time_ms: i64, end_time_ms: Option<i64>, text: &str) -> ParsedLyricsWord {
        ParsedLyricsWord {
            time_ms,
            end_time_ms,
            word: text.to_string(),
        }
    }

    fn lyrics_with_lines(lines: Vec<ParsedLyricsLine>) -> ParsedLyrics {
        ParsedLyrics {
            plain: vec![],
            synced: lines,
            sections: None,
            agents: None,
            songwriters: None,
            language: None,
            are_from_remote: false,
        }
    }

    #[test]
    fn picks_active_line_from_start_time() {
        let lyrics = lyrics_with_lines(vec![line(1_000, Some(2_000), "one"), line(3_000, Some(4_000), "two")]);

        let active = compute_active_state(&lyrics, 3.1);

        assert_eq!(active.line_index, Some(1));
    }

    #[test]
    fn keeps_previous_line_active_during_gap() {
        let lyrics = lyrics_with_lines(vec![line(1_000, Some(2_000), "one"), line(4_000, Some(5_000), "two")]);

        let active = compute_active_state(&lyrics, 2.6);

        assert_eq!(active.line_index, Some(0));
    }

    #[test]
    fn computes_active_word_and_progress() {
        let mut active_line = line(1_000, Some(3_000), "hello world");
        active_line.words = Some(vec![word(1_000, Some(1_500), "hello"), word(1_500, Some(2_000), "world")]);
        let lyrics = lyrics_with_lines(vec![active_line]);

        let active = compute_active_state(&lyrics, 1.75);

        assert_eq!(active.line_index, Some(0));
        assert_eq!(active.word_index, Some(1));
        assert!(active.word_progress > 0.45 && active.word_progress < 0.55);
    }

    #[test]
    fn preserves_spacing_between_rendered_words() {
        let mut active_line = line(1_000, Some(3_000), "hello world!");
        active_line.words = Some(vec![
            word(1_000, Some(1_500), "hello"),
            word(1_500, Some(2_000), "world"),
            word(2_000, Some(2_300), "!"),
        ]);

        let active = ActiveLyricsState {
            line_index: Some(0),
            word_index: Some(1),
            word_progress: 0.5,
        };

        let rendered = render_words(&active_line, &active, 0);

        assert_eq!(rendered[0].trailing_whitespace, " ");
        assert_eq!(rendered[1].trailing_whitespace, "");
        assert_eq!(rendered[2].trailing_whitespace, "");
    }

    #[test]
    fn exposes_section_label_for_first_line() {
        let line = line(10_000, Some(12_000), "chorus");
        let sections = [ParsedLyricsSection {
            name: "Chorus".to_string(),
            start_time_ms: 10_000,
            end_time_ms: 12_000,
            lines: vec![line.clone()],
            agent_id: None,
        }];
        let label = section_label_for_line(
            Some(&sections),
            &line,
        );

        assert_eq!(label, Some("Chorus"));
    }

    #[test]
    fn marks_background_vocals_from_agent_map() {
        let mut line = line(0, Some(1_000), "adlib");
        line.agent_id = Some("bg".to_string());
        let lyrics = ParsedLyrics {
            plain: vec![],
            synced: vec![line.clone()],
            sections: None,
            agents: Some(vec![aurelia_core::models::ParsedLyricsAgent {
                id: "bg".to_string(),
                agent_type: "other".to_string(),
            }]),
            songwriters: None,
            language: None,
            are_from_remote: false,
        };

        assert!(is_background_vocal(&lyrics, &line));
    }
}
