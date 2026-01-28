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

#[uniffi::export]
pub fn clear_cache(app_data_dir: String) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::clear_cache(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub fn get_lyrics(
    _server_url: String, // Kept for API compatibility, though currently unused for LRCLIB
    _token: String,
    _item_id: String,
    artist: String,
    title: String,
) -> String {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let client = services::LrcLibClient::new();
        match client.search_lyrics(&artist, &title).await {
            Ok(results) => services::LrcLibClient::get_best_lyrics(&results).unwrap_or_default(),
            Err(_) => String::new(),
        }
    })
}

#[uniffi::export]
pub fn toggle_favorite(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    is_favorite: bool,
) -> Result<bool, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;

    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        let new_state = !is_favorite;
        client.toggle_favorite(&user_id, &item_id, new_state).await?;
        Ok(new_state)
    })
}

// Playlist operations

#[uniffi::export]
pub fn get_playlists(
    server_url: String,
    token: String,
    user_id: String,
) -> Result<Vec<models::Playlist>, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.get_playlists(&user_id).await
    })
}

#[uniffi::export]
pub fn create_playlist(
    server_url: String,
    token: String,
    data: models::PlaylistCreateData,
) -> Result<models::Playlist, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.create_playlist(&data).await
    })
}

#[uniffi::export]
pub fn update_playlist(
    server_url: String,
    token: String,
    playlist_id: String,
    updates: models::PlaylistUpdateData,
) -> Result<models::Playlist, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.update_playlist(&playlist_id, &updates).await
    })
}

#[uniffi::export]
pub fn delete_playlist(
    server_url: String,
    token: String,
    playlist_id: String,
) -> Result<(), error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.delete_playlist(&playlist_id).await
    })
}

#[uniffi::export]
pub fn add_playlist_items(
    server_url: String,
    token: String,
    playlist_id: String,
    item_ids: Vec<String>,
) -> Result<(), error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.add_playlist_items(&playlist_id, &item_ids).await
    })
}

#[uniffi::export]
pub fn remove_playlist_items(
    server_url: String,
    token: String,
    playlist_id: String,
    item_ids: Vec<String>,
) -> Result<(), error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.remove_playlist_items(&playlist_id, &item_ids).await
    })
}

#[uniffi::export]
pub fn get_playlist_items(
    server_url: String,
    token: String,
    playlist_id: String,
) -> Result<Vec<models::Song>, error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.get_playlist_items(&playlist_id).await
    })
}


#[uniffi::export]
pub fn mark_item_played(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
) -> Result<(), error::AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| error::AppError::UniFfi(error.to_string()))?;
    runtime.block_on(async {
        let client = services::JellyfinClient::with_auth(server_url, token);
        client.mark_item_played(&user_id, &item_id).await
    })
}

uniffi::setup_scaffolding!();
