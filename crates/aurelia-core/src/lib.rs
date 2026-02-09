pub mod api;
pub mod cache;
pub mod db;
pub mod domain;
pub mod error;
pub mod lastfm_core;
pub mod listenbrainz_core;
pub mod models;
pub mod services;
pub mod state;
pub mod tray_settings;
pub mod utils;

// Desktop-only modules
#[cfg(feature = "desktop")]
pub mod audio;
#[cfg(feature = "desktop")]
pub mod discord_rpc;
#[cfg(feature = "desktop")]
pub mod media_controls;

#[uniffi::export]
pub fn ping() -> String {
    "pong".to_string()
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn authenticate(
    server_url: String,
    username: String,
    password: String,
    device_id: String,
) -> Result<models::LoginResponse, error::AppError> {
    let client = services::JellyfinClient::new(server_url);
    client.authenticate(&username, &password, &device_id).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn fetch_songs(
    server_url: String,
    token: String,
    user_id: String,
    app_data_dir: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    let songs = client.get_music_library(&user_id).await?;
    if !app_data_dir.is_empty() {
        let app_dir = std::path::PathBuf::from(app_data_dir);
        if let Err(err) = cache::sync_library(app_dir, &songs, &[], &[]) {
            tracing::warn!("Failed to cache songs: {err}");
        }
    }
    Ok(songs)
}

#[uniffi::export]
pub fn load_cached_songs(app_data_dir: String) -> Result<Vec<models::Song>, error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(vec![]);
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::get_songs(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

/// Derive mobile home sections from an in-memory song list.
#[uniffi::export]
pub fn derive_mobile_home_data(
    songs: Vec<models::Song>,
    most_played_limit: i64,
    recently_played_limit: i64,
    album_section_limit: i64,
    featured_albums_limit: i64,
) -> models::MobileHomeData {
    let limits = domain::services::MobileHomeViewLimits {
        most_played: most_played_limit.max(0) as u32,
        recently_played: recently_played_limit.max(0) as u32,
        album_section: album_section_limit.max(0) as u32,
        featured_albums: featured_albums_limit.max(0) as u32,
    };
    let mut rng = rand::rng();
    domain::services::derive_mobile_home_data(&songs, limits, &mut rng)
}

#[uniffi::export]
pub fn cache_songs(app_data_dir: String, songs: Vec<models::Song>) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::sync_library(app_dir, &songs, &[], &[])
        .map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn get_library_sync_state(app_data_dir: String) -> Result<String, error::AppError> {
    if app_data_dir.is_empty() {
        return Ok("".to_string());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::get_sync_state(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn set_library_sync_state(
    app_data_dir: String,
    state_json: String,
) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::set_sync_state(app_dir, &state_json)
        .map_err(|err| error::AppError::Database(err.to_string()))
}

/// Get sync state as a typed struct (better for UI binding)
#[uniffi::export]
pub fn get_sync_state(app_data_dir: String) -> Result<domain::SyncState, error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(domain::SyncState::default());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    let json =
        cache::get_sync_state(app_dir).map_err(|err| error::AppError::Database(err.to_string()))?;

    if json.is_empty() {
        return Ok(domain::SyncState::default());
    }

    serde_json::from_str(&json).map_err(|err| error::AppError::Serialization(err.to_string()))
}

#[uniffi::export]
pub fn build_stream_url(
    server_url: String,
    token: String,
    item_id: String,
    container: Option<String>,
) -> String {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.get_audio_stream_url(&item_id, container.as_deref())
}

/// Build a stream URL optimized for mobile playback.
/// Uses HLS transcoding for non-seekable containers so that Media3/ExoPlayer can seek natively.
#[uniffi::export]
pub fn build_mobile_stream_url(
    server_url: String,
    token: String,
    item_id: String,
    container: Option<String>,
) -> String {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.get_mobile_audio_stream_url(&item_id, container.as_deref())
}

#[cfg(test)]
mod tests {
    use super::{build_mobile_stream_url, build_stream_url};

    #[test]
    fn build_stream_url_uses_static_for_seekable() {
        let url = build_stream_url(
            "http://localhost:8096".to_string(),
            "token".to_string(),
            "song123".to_string(),
            Some("flac".to_string()),
        );
        assert!(url.contains("/Audio/song123/stream"));
        assert!(url.contains("static=true"));
    }

    #[test]
    fn build_stream_url_transcodes_non_seekable() {
        let url = build_stream_url(
            "http://localhost:8096".to_string(),
            "token".to_string(),
            "song123".to_string(),
            Some("alac".to_string()),
        );
        assert!(url.contains("/Audio/song123/stream.aac"));
        assert!(!url.contains("static=true"));
    }

    #[test]
    fn build_mobile_stream_url_uses_universal_for_non_seekable() {
        let url = build_mobile_stream_url(
            "http://localhost:8096".to_string(),
            "token".to_string(),
            "song123".to_string(),
            Some("alac".to_string()),
        );
        assert!(url.contains("/Audio/song123/universal"));
        assert!(url.contains("transcodingProtocol=http"));
    }
}

#[uniffi::export]
pub fn save_credentials(
    app_data_dir: String,
    credentials: models::Credentials,
) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::save_credentials(app_dir, &credentials)
        .map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn load_credentials(
    app_data_dir: String,
) -> Result<Option<models::Credentials>, error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(None);
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::load_credentials(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn clear_credentials(app_data_dir: String) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::clear_credentials(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn save_setting(
    app_data_dir: String,
    key: String,
    value: String,
) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::save_setting(app_dir, &key, &value)
        .map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn load_setting(app_data_dir: String, key: String) -> Result<Option<String>, error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(None);
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::load_setting(app_dir, &key).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn delete_setting(app_data_dir: String, key: String) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::delete_setting(app_dir, &key).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn clear_cache(app_data_dir: String) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::clear_cache(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_lyrics(
    server_url: String,
    token: String,
    item_id: String,
    artist: String,
    title: String,
) -> String {
    // 1. Try Jellyfin server first
    if !server_url.is_empty() && !token.is_empty() && !item_id.is_empty() {
        let client = services::JellyfinClient::with_auth(server_url, token);
        if let Ok(Some(jf_lyrics)) = client.get_lyrics(&item_id).await
            && let Ok(lrc) = utils::lyrics::jellyfin_to_lrc(&jf_lyrics)
                && !lrc.trim().is_empty() {
                    return lrc;
                }
    }

    // 2. Fall back to LrcLib
    let client = services::LrcLibClient::new();
    match client.search_lyrics(&artist, &title).await {
        Ok(results) => services::LrcLibClient::get_best_lyrics(&results).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_parsed_lyrics(
    server_url: String,
    token: String,
    item_id: String,
    artist: String,
    title: String,
    path: Option<String>,
    aurelia_server_url: Option<String>,
) -> models::ParsedLyrics {
    // 1. Try sidecar lyrics first (richest source: TTML with word-sync, sections, agents)

    // 1a. Try local sidecar files (.ttml, .lrc) next to the audio file
    let resolved_path = match path {
        Some(ref p) if !p.is_empty() => Some(p.clone()),
        _ if !server_url.is_empty() && !token.is_empty() && !item_id.is_empty() => {
            tracing::info!("[Lyrics] No path provided, fetching from Jellyfin item metadata");
            let client = services::JellyfinClient::with_auth(server_url.clone(), token.clone());
            match client.get_item_path(&item_id).await {
                Ok(Some(p)) => {
                    tracing::info!("[Lyrics] Got path from Jellyfin: {}", p);
                    Some(p)
                }
                Ok(None) => {
                    tracing::info!("[Lyrics] Jellyfin item has no path");
                    None
                }
                Err(e) => {
                    tracing::warn!("[Lyrics] Failed to fetch item path: {}", e);
                    None
                }
            }
        }
        _ => None,
    };

    if let Some(ref audio_path) = resolved_path {
        tracing::info!("[Lyrics] Trying local sidecar files for: {}", audio_path);
        if let Some(parsed) = try_read_sidecar_lyrics(audio_path) {
            tracing::info!(
                "[Lyrics] Local sidecar found: syncedLines={}, hasSections={}, hasWords={}",
                parsed.synced.len(),
                parsed.sections.is_some(),
                parsed.synced.first().is_some_and(|l| l.words.is_some()),
            );
            if parsed.is_valid() {
                return parsed;
            }
        } else {
            tracing::info!("[Lyrics] No local sidecar files found");
        }
    }

    // 1b. Try fetching sidecar lyrics from the Aurelia web backend (for remote clients)
    if let Some(ref aurelia_url) = aurelia_server_url
        && !aurelia_url.is_empty() {
            tracing::info!(
                "[Lyrics] Trying remote sidecar from Aurelia server: {}",
                aurelia_url
            );
            match fetch_remote_sidecar_lyrics(aurelia_url, &item_id).await {
                Ok(Some(parsed)) if parsed.is_valid() => {
                    tracing::info!(
                        "[Lyrics] Remote sidecar found: syncedLines={}, hasWords={}",
                        parsed.synced.len(),
                        parsed.synced.first().is_some_and(|l| l.words.is_some()),
                    );
                    return parsed;
                }
                Ok(_) => {
                    tracing::info!("[Lyrics] Remote sidecar returned no valid lyrics");
                }
                Err(e) => {
                    tracing::warn!("[Lyrics] Remote sidecar fetch failed: {}", e);
                }
            }
        }

    // 2. Try Jellyfin lyrics API
    if !server_url.is_empty() && !token.is_empty() && !item_id.is_empty() {
        tracing::info!(
            "[Lyrics] Trying Jellyfin: itemId={}, serverUrl={}...",
            item_id,
            &server_url[..server_url.len().min(30)]
        );
        let client = services::JellyfinClient::with_auth(server_url.clone(), token.clone());
        match client.get_lyrics(&item_id).await {
            Ok(Some(jf_lyrics)) => {
                let line_count = jf_lyrics.lyrics.len();
                let lines_with_cues = jf_lyrics
                    .lyrics
                    .iter()
                    .filter(|l| l.cues.as_ref().is_some_and(|c| !c.is_empty()))
                    .count();
                let has_metadata = jf_lyrics.metadata.is_some();
                tracing::info!(
                    "[Lyrics] Jellyfin returned {} lines, {} with cues, hasMetadata={}",
                    line_count,
                    lines_with_cues,
                    has_metadata,
                );

                let parsed = utils::lyrics::jellyfin_to_parsed_lyrics(&jf_lyrics);
                tracing::info!(
                    "[Lyrics] Converted: syncedLines={}, hasWords={}, plainLines={}",
                    parsed.synced.len(),
                    parsed.synced.first().is_some_and(|l| l.words.is_some()),
                    parsed.plain.len(),
                );
                if parsed.is_valid() {
                    return parsed;
                }
                tracing::warn!("[Lyrics] Jellyfin lyrics parsed but not valid");
            }
            Ok(None) => {
                tracing::info!(
                    "[Lyrics] Jellyfin returned no lyrics for itemId={}",
                    item_id
                );
            }
            Err(e) => {
                tracing::warn!("[Lyrics] Jellyfin lyrics fetch error: {}", e);
            }
        }
    } else {
        tracing::info!(
            "[Lyrics] Skipping Jellyfin (serverUrl empty={}, token empty={}, itemId empty={})",
            server_url.is_empty(),
            token.is_empty(),
            item_id.is_empty(),
        );
    }

    // 3. Fall back to LrcLib
    tracing::info!(
        "[Lyrics] Falling back to LrcLib for '{}' by '{}'",
        title,
        artist
    );
    let lrclib_client = services::LrcLibClient::new();
    let raw = match lrclib_client.search_lyrics(&artist, &title).await {
        Ok(results) => {
            let best = services::LrcLibClient::get_best_lyrics(&results).unwrap_or_default();
            tracing::info!("[Lyrics] LrcLib returned {} bytes", best.len());
            best
        }
        Err(e) => {
            tracing::warn!("[Lyrics] LrcLib search error: {}", e);
            String::new()
        }
    };
    utils::lyrics_parser::parse_lyrics(&raw)
}

/// Fetch sidecar lyrics for a Jellyfin item from the local filesystem.
/// Used by the web backend's `/api/lyrics/sidecar/{item_id}` endpoint.
pub async fn get_sidecar_lyrics(
    server_url: String,
    token: String,
    item_id: String,
) -> Result<models::ParsedLyrics, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    let path = client
        .get_item_path(&item_id)
        .await?
        .ok_or_else(|| error::AppError::General("Item has no filesystem path".to_string()))?;

    try_read_sidecar_lyrics(&path).ok_or_else(|| {
        error::AppError::General("No sidecar lyrics found for this item".to_string())
    })
}

/// Fetch sidecar lyrics from a remote Aurelia web backend.
async fn fetch_remote_sidecar_lyrics(
    aurelia_server_url: &str,
    item_id: &str,
) -> Result<Option<models::ParsedLyrics>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "{}/api/lyrics/sidecar/{}",
        aurelia_server_url.trim_end_matches('/'),
        item_id
    );
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    // The web backend wraps responses in ApiResponse { status: "ok", data, error }
    let body: serde_json::Value = resp.json().await?;
    if body.get("status") == Some(&serde_json::Value::String("ok".to_string())) {
        if let Some(data) = body.get("data") {
            let parsed: models::ParsedLyrics = serde_json::from_value(data.clone())?;
            return Ok(Some(parsed));
        }
    } else {
        tracing::warn!("[Lyrics] Remote sidecar response not ok: {:?}", body);
    }

    Ok(None)
}

/// Try to read a sidecar lyrics file next to the given audio file path.
///
/// Checks for `.ttml` first (richest metadata), then `.lrc` / `.elrc`.
fn try_read_sidecar_lyrics(audio_path: &str) -> Option<models::ParsedLyrics> {
    let audio = std::path::Path::new(audio_path);
    let stem = audio.file_stem()?.to_str()?;
    let parent = audio.parent()?;

    // Extensions to try, in priority order (TTML first — richest format)
    let extensions = [".ttml", ".lrc", ".elrc", ".txt"];

    for ext in &extensions {
        let candidate = parent.join(format!("{stem}{ext}"));
        tracing::debug!("[Lyrics] Checking sidecar: {}", candidate.display());
        if candidate.is_file() {
            match std::fs::read_to_string(&candidate) {
                Ok(contents) => {
                    tracing::info!(
                        "[Lyrics] Reading sidecar: {} ({} bytes)",
                        candidate.display(),
                        contents.len()
                    );
                    let parsed = utils::lyrics_parser::parse_lyrics(&contents);
                    if parsed.is_valid() {
                        return Some(parsed);
                    }
                    tracing::warn!(
                        "[Lyrics] Sidecar {} parsed but not valid, trying next",
                        candidate.display()
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "[Lyrics] Failed to read sidecar {}: {}",
                        candidate.display(),
                        e
                    );
                }
            }
        }
    }

    None
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn toggle_favorite(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    is_favorite: bool,
) -> Result<bool, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client
        .toggle_favorite(&user_id, &item_id, is_favorite)
        .await?;
    Ok(is_favorite)
}

// Playlist operations

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_playlists(
    server_url: String,
    token: String,
    user_id: String,
) -> Result<Vec<models::Playlist>, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.get_playlists(&user_id).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn create_playlist(
    server_url: String,
    token: String,
    data: models::PlaylistCreateData,
) -> Result<models::Playlist, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.create_playlist(&data).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn update_playlist(
    server_url: String,
    token: String,
    playlist_id: String,
    updates: models::PlaylistUpdateData,
) -> Result<models::Playlist, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.update_playlist(&playlist_id, &updates).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn delete_playlist(
    server_url: String,
    token: String,
    playlist_id: String,
) -> Result<(), error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.delete_playlist(&playlist_id).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn add_playlist_items(
    server_url: String,
    token: String,
    playlist_id: String,
    item_ids: Vec<String>,
) -> Result<(), error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.add_playlist_items(&playlist_id, &item_ids).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn remove_playlist_items(
    server_url: String,
    token: String,
    playlist_id: String,
    item_ids: Vec<String>,
) -> Result<(), error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.remove_playlist_items(&playlist_id, &item_ids).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_playlist_items(
    server_url: String,
    token: String,
    playlist_id: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.get_playlist_items(&playlist_id).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn mark_item_played(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
) -> Result<(), error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.mark_item_played(&user_id, &item_id).await
}

// Lazy-load functions for hybrid sync

/// Sync only songs (fast startup). Artists/albums are fetched on-demand.
#[uniffi::export(async_runtime = "tokio")]
pub async fn sync_songs_only(
    server_url: String,
    token: String,
    user_id: String,
    app_data_dir: String,
) -> Result<bool, error::AppError> {
    // Initialize database
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    // Fetch songs only
    let client = services::JellyfinClient::with_auth(server_url, token);
    let songs = client.get_music_library(&user_id).await?;

    // Use incremental sync
    db::sync_songs_only(&songs).map_err(|e| error::AppError::Database(e.to_string()))
}

/// Fetch a single artist from server and cache it
#[uniffi::export(async_runtime = "tokio")]
pub async fn fetch_artist(
    server_url: String,
    token: String,
    user_id: String,
    artist_id: String,
    app_data_dir: String,
) -> Result<models::Artist, error::AppError> {
    // Initialize database
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    // Fetch from server
    let client = services::JellyfinClient::with_auth(server_url, token);
    let artist = client.get_artist_details(&user_id, &artist_id).await?;

    // Cache in database
    db::artists::cache(&artist).map_err(|e| error::AppError::Database(e.to_string()))?;

    Ok(artist)
}

/// Fetch a single album from server and cache it
#[uniffi::export(async_runtime = "tokio")]
pub async fn fetch_album(
    server_url: String,
    token: String,
    user_id: String,
    album_id: String,
    app_data_dir: String,
) -> Result<models::Album, error::AppError> {
    // Initialize database
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    // Fetch from server
    let client = services::JellyfinClient::with_auth(server_url, token);
    let album = client.get_album_details(&user_id, &album_id).await?;

    // Cache in database
    db::albums::cache(&album).map_err(|e| error::AppError::Database(e.to_string()))?;

    Ok(album)
}

/// Get a cached artist from local database
#[uniffi::export]
pub fn get_cached_artist(
    app_data_dir: String,
    artist_id: String,
) -> Result<Option<models::Artist>, error::AppError> {
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    db::artists::get_by_id(&artist_id).map_err(|e| error::AppError::Database(e.to_string()))
}

/// Get a cached album from local database
#[uniffi::export]
pub fn get_cached_album(
    app_data_dir: String,
    album_id: String,
) -> Result<Option<models::Album>, error::AppError> {
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    db::albums::get_by_id(&album_id).map_err(|e| error::AppError::Database(e.to_string()))
}

/// Get a cached song from local database
#[uniffi::export]
pub fn get_cached_song(
    app_data_dir: String,
    song_id: String,
) -> Result<Option<models::Song>, error::AppError> {
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    db::songs::get_by_id(&song_id).map_err(|e| error::AppError::Database(e.to_string()))
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_recently_played(
    server_url: String,
    token: String,
    user_id: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.get_recently_played(&user_id).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_instant_mix(
    server_url: String,
    token: String,
    item_id: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.get_instant_mix(&item_id).await
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_song_share_urls(
    song: models::Song,
) -> Result<std::collections::HashMap<String, String>, error::AppError> {
    services::MusicBrainzService::get_song_share_urls(&song)
        .await
        .map_err(error::AppError::UniFfi)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_related_artists(
    app_data_dir: String,
    artist_id: String,
) -> Result<Vec<models::Artist>, error::AppError> {
    let app_data_path = std::path::PathBuf::from(&app_data_dir);
    db::init(&app_data_path).map_err(|e| error::AppError::Database(e.to_string()))?;

    let all_artists =
        db::artists::get_all().map_err(|e| error::AppError::Database(e.to_string()))?;
    let all_songs = db::songs::get_all().map_err(|e| error::AppError::Database(e.to_string()))?;

    let current_artist = all_artists
        .iter()
        .find(|a| a.id == artist_id)
        .ok_or_else(|| error::AppError::UniFfi("Artist not found".to_string()))?;

    const COLLABORATION_SCORE: i32 = 10;
    const SHARED_GENRE_SCORE: i32 = 5;
    const SHARED_ALBUM_SCORE: i32 = 2;

    let current_artist_songs: Vec<&models::Song> = all_songs
        .iter()
        .filter(|s| {
            s.artists
                .as_ref()
                .is_some_and(|a| a.contains(&current_artist.name))
        })
        .collect();

    let current_artist_genres: std::collections::HashSet<&String> = current_artist_songs
        .iter()
        .flat_map(|s| {
            s.genres
                .as_ref()
                .map_or_else(Vec::new, |g| g.iter().collect())
        })
        .collect();

    let current_artist_albums: std::collections::HashSet<&String> = current_artist_songs
        .iter()
        .filter_map(|s| s.album.as_ref())
        .collect();

    let mut artist_scores: std::collections::HashMap<String, i32> =
        std::collections::HashMap::new();

    for other_artist in &all_artists {
        if other_artist.id == current_artist.id {
            continue;
        }

        let mut score = 0;
        let other_artist_songs: Vec<&models::Song> = all_songs
            .iter()
            .filter(|s| {
                s.artists
                    .as_ref()
                    .is_some_and(|a| a.contains(&other_artist.name))
            })
            .collect();

        if other_artist_songs.is_empty() {
            continue;
        }

        let collaborations = current_artist_songs
            .iter()
            .filter(|s| {
                s.artists
                    .as_ref()
                    .is_some_and(|a| a.contains(&other_artist.name))
            })
            .count();
        score += collaborations as i32 * COLLABORATION_SCORE;

        let other_artist_genres: std::collections::HashSet<&String> = other_artist_songs
            .iter()
            .flat_map(|s| {
                s.genres
                    .as_ref()
                    .map_or_else(Vec::new, |g| g.iter().collect())
            })
            .collect();

        for genre in &other_artist_genres {
            if current_artist_genres.contains(genre) {
                score += SHARED_GENRE_SCORE;
            }
        }

        let other_artist_albums: std::collections::HashSet<&String> = other_artist_songs
            .iter()
            .filter_map(|s| s.album.as_ref())
            .collect();

        for album in &other_artist_albums {
            if current_artist_albums.contains(album) && collaborations == 0 {
                score += SHARED_ALBUM_SCORE;
            }
        }

        if score > 0 {
            artist_scores.insert(other_artist.id.clone(), score);
        }
    }

    let mut sorted_artists: Vec<_> = artist_scores.into_iter().collect();
    sorted_artists.sort_by_key(|b| std::cmp::Reverse(b.1));

    let result: Vec<models::Artist> = sorted_artists
        .iter()
        .take(6)
        .filter_map(|(artist_id, _)| all_artists.iter().find(|a| a.id == *artist_id).cloned())
        .collect();

    Ok(result)
}

uniffi::setup_scaffolding!();
