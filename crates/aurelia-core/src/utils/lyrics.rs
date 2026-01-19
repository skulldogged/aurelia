//! Lyrics format conversion utilities

use crate::models::JellyfinLyrics;
use std::fmt::Write;

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
