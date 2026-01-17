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

#[uniffi::export]
pub fn authenticate(
    server_url: String,
    username: String,
    password: String,
) -> Result<models::LoginResponse, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::new(server_url);
        client.authenticate(&username, &password).await
    })
}

#[uniffi::export]
pub fn fetch_songs(
    server_url: String,
    token: String,
    user_id: String,
    app_data_dir: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        let songs = client.get_music_library(&user_id).await?;
        if !app_data_dir.is_empty() {
            let app_dir = std::path::PathBuf::from(app_data_dir);
            if let Err(err) = cache::sync_library(app_dir, &songs, &[], &[]) {
                tracing::warn!("Failed to cache songs: {err}");
            }
        }
        Ok(songs)
    })
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

uniffi::setup_scaffolding!();
