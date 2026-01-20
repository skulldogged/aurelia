pub mod cache;
pub mod database;
pub mod db;
pub mod domain;
pub mod error;
pub mod listenbrainz_core;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

#[uniffi::export]
pub fn ping() -> String {
    "pong".to_string()
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn authenticate(
    server_url: String,
    username: String,
    password: String,
) -> Result<models::LoginResponse, error::AppError> {
    let client = services::JellyfinClient::new(server_url);
    client.authenticate(&username, &password).await
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
pub fn clear_cache(app_data_dir: String) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::clear_cache(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
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

#[uniffi::export(async_runtime = "tokio")]
pub async fn toggle_favorite(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    is_favorite: bool,
) -> Result<bool, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    client.toggle_favorite(&user_id, &item_id, is_favorite).await?;
    Ok(!is_favorite)
}

/// Get lyrics for a track. Tries Jellyfin first, falls back to LrcLib.
/// Returns LRC-formatted lyrics string, or error if not found.
#[uniffi::export(async_runtime = "tokio")]
pub async fn get_lyrics(
    server_url: String,
    token: String,
    item_id: String,
    artist: String,
    title: String,
) -> Result<String, error::AppError> {
    // Try Jellyfin first
    if !server_url.is_empty() && !token.is_empty() {
        let client = services::JellyfinClient::with_auth(server_url, token);
        if let Ok(Some(jellyfin_lyrics)) = client.get_lyrics(&item_id).await {
            if !jellyfin_lyrics.lyrics.is_empty() {
                if let Ok(lrc) = utils::lyrics::jellyfin_to_lrc(&jellyfin_lyrics) {
                    return Ok(lrc);
                }
            }
        }
    }

    // Fall back to LrcLib
    let lrclib = services::LrcLibClient::new();
    let results = lrclib.search_lyrics(&artist, &title).await?;

    services::LrcLibClient::get_best_lyrics(&results)
        .ok_or_else(|| error::AppError::NotFound(format!("No lyrics found for '{title}'")))
}

#[uniffi::export]
pub fn save_credentials(
    app_data_dir: String,
    credentials: models::Credentials,
) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Err(error::AppError::Database("App data dir not set".to_string()));
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::save_credentials(app_dir, &credentials)
        .map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn load_credentials(app_data_dir: String) -> Result<Option<models::Credentials>, error::AppError> {
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

uniffi::setup_scaffolding!();
