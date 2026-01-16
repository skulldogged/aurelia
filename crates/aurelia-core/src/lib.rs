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
    let client = services::JellyfinClient::new(server_url);
    tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?
        .block_on(client.authenticate(&username, &password))
}

#[uniffi::export]
pub fn fetch_songs(
    server_url: String,
    token: String,
    user_id: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let client = services::JellyfinClient::with_auth(server_url, token);
    tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?
        .block_on(client.get_music_library(&user_id))
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

