//! Lyrics-related command handlers

use crate::models::JellyfinLyrics;
use crate::services::{JellyfinClient, LrcLibClient};

/// Get lyrics for a track
#[tauri::command]
pub async fn get_lyrics(
    id: String,
    artist: String,
    title: String,
    _path: Option<String>,
) -> Result<String, String> {
    // First, try to get lyrics from Jellyfin API
    if let Ok(Some(creds)) = crate::handlers::auth::get_saved_credentials() {
        let client = JellyfinClient::with_auth(creds.server_url, creds.token);

        if let Ok(Some(jellyfin_lyrics)) = client.get_lyrics(&id).await {
            if !jellyfin_lyrics.lyrics.is_empty() {
                return Ok(convert_jellyfin_lyrics_to_lrc(jellyfin_lyrics));
            }
        }
    }

    // Fallback to lrclib.net
    println!("[get_lyrics] No lyrics found on Jellyfin server. Fetching from lrclib.net...");
    let lrclib_client = LrcLibClient::new();

    let search_results = lrclib_client
        .search_lyrics(&artist, &title)
        .await
        .map_err(|e| e.to_string())?;

    println!(
        "[get_lyrics] Found {} search results.",
        search_results.len()
    );

    // Get the best lyrics match
    if let Some(lyrics) = LrcLibClient::get_best_lyrics(&search_results) {
        println!(
            "[get_lyrics] Returning lyrics from the first result for '{}'",
            title
        );
        Ok(lyrics)
    } else {
        println!(
            "[get_lyrics] No lyrics found in any results for '{}'",
            title
        );
        Err("No lyrics found".to_string())
    }
}

/// Convert Jellyfin lyrics format to LRC format
fn convert_jellyfin_lyrics_to_lrc(lyrics: JellyfinLyrics) -> String {
    let mut lrc_content = String::new();

    for line in lyrics.lyrics {
        if let Some(timestamp) = line.timestamp {
            // Convert Jellyfin timestamp (100ns ticks from start) to LRC format (MM:SS.mm)
            let total_seconds = timestamp / 10_000_000;
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            let milliseconds = (timestamp % 10_000_000) / 10_000;

            lrc_content.push_str(&format!(
                "[{:02}:{:02}.{:03}] {}\n",
                minutes, seconds, milliseconds, line.text
            ));
        } else {
            lrc_content.push_str(&format!("{}\n", line.text));
        }
    }

    lrc_content
}
