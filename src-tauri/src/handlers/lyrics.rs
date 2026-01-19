//! Lyrics-related command handlers

use aurelia_core::services::{JellyfinClient, LrcLibClient};
use aurelia_core::utils::lyrics::jellyfin_to_lrc;
use tracing::{debug, info, warn};

/// Get lyrics for a track
#[tauri::command]
#[specta::specta]
pub async fn get_lyrics(
    app: tauri::AppHandle,
    id: String,
    artist: String,
    title: String,
    _path: Option<String>,
) -> Result<String, String> {
    if let Ok(Some(creds)) = super::auth::get_credentials_cached(&app).await {
        let client = JellyfinClient::with_auth(creds.server_url, creds.token);

        if let Ok(Some(jellyfin_lyrics)) = client.get_lyrics(&id).await
            && !jellyfin_lyrics.lyrics.is_empty()
        {
            match jellyfin_to_lrc(&jellyfin_lyrics) {
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
