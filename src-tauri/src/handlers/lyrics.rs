//! Lyrics-related command handlers

use crate::models::JellyfinLyrics;
use crate::services::{JellyfinClient, LrcLibClient};
use std::fmt::Write;
use tracing::{debug, info, warn};

/// Get lyrics for a track
#[tauri::command]
#[specta::specta]
pub async fn get_lyrics(
    id: String,
    artist: String,
    title: String,
    _path: Option<String>,
) -> Result<String, String> {
    if let Ok(Some(creds)) = crate::handlers::auth::get_saved_credentials().await {
        let client = JellyfinClient::with_auth(creds.server_url, creds.token);

        if let Ok(Some(jellyfin_lyrics)) = client.get_lyrics(&id).await
            && !jellyfin_lyrics.lyrics.is_empty()
        {
            match convert_jellyfin_lyrics_to_lrc(jellyfin_lyrics) {
                Ok(lrc_content) => return Ok(lrc_content),
                Err(e) => {
                    warn!("Failed to convert Jellyfin lyrics to LRC format: {}", e);
                }
            }
        }
    }

    info!("No lyrics found on Jellyfin server. Fetching from lrclib.net...");
    let lrclib_client = LrcLibClient::new();

    let search_results = match lrclib_client.search_lyrics(&artist, &title).await {
        Ok(results) => results,
        Err(e) => return Err(e.to_string()),
    };

    debug!(
        "Found {} search results from lrclib.net",
        search_results.len()
    );

    LrcLibClient::get_best_lyrics(&search_results).map_or_else(
        || {
            warn!("No lyrics found for '{}'", title);
            Err(format!("No lyrics found for '{title}'"))
        },
        |lyrics| {
            debug!("Returning lyrics for '{}'", title);
            Ok(lyrics)
        },
    )
}

/// Convert Jellyfin lyrics format to LRC format
fn convert_jellyfin_lyrics_to_lrc(lyrics: JellyfinLyrics) -> Result<String, std::fmt::Error> {
    let mut lrc_content = String::new();

    for line in lyrics.lyrics {
        if let Some(timestamp) = line.timestamp {
            // Convert Jellyfin timestamp (100ns ticks from start) to LRC format (MM:SS.mm)
            let total_seconds = timestamp / 10_000_000.0;
            // Use floor to ensure we don't exceed valid time bounds
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
