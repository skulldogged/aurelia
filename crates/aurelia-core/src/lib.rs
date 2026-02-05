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
pub fn clear_cache(app_data_dir: String) -> Result<(), error::AppError> {
    if app_data_dir.is_empty() {
        return Ok(());
    }
    let app_dir = std::path::PathBuf::from(app_data_dir);
    cache::clear_cache(app_dir).map_err(|err| error::AppError::Database(err.to_string()))
}

#[uniffi::export]
pub async fn get_lyrics(
    _server_url: String, // Kept for API compatibility, though currently unused for LRCLIB
    _token: String,
    _item_id: String,
    artist: String,
    title: String,
) -> String {
    let client = services::LrcLibClient::new();
    match client.search_lyrics(&artist, &title).await {
        Ok(results) => services::LrcLibClient::get_best_lyrics(&results).unwrap_or_default(),
        Err(_) => String::new(),
    }
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
