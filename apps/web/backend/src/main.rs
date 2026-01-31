use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

// Re-export types from aurelia_core
use aurelia_core::models::{Album, Artist, Song};

// Library data structure
#[derive(Debug, Serialize)]
struct LibraryData {
    songs: Vec<Song>,
    albums: Vec<Album>,
    artists: Vec<Artist>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HomeViewData {
    recently_played: Vec<Song>,
    recently_added: Vec<Album>,
    random_albums: Vec<Album>,
    featured_albums: Vec<Album>,
}

// Types matching the aurelia-core API
/// API response that serializes as a discriminated union
/// Success: { "status": "ok", "data": T }
/// Error: { "status": "error", "error": E }
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            data: Some(data),
            error: None,
        }
    }

    fn err(error: String) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(error),
        }
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest {
    server_url: String,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FavoriteRequest {
    is_favorite: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistItemsRequest {
    song_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamUrlRequest {
    server_url: String,
    token: String,
    container: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyricsRequest {
    id: String,
    artist: String,
    title: String,
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapabilitiesRequest {
    server_url: String,
    token: String,
    device_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackReportRequest {
    server_url: String,
    token: String,
    item_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    position_ticks: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_paused: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageUrlRequest {
    image_type: String,
    server_url: String,
    token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<u32>,
}

// App state
struct AppState {
    app_data_dir: PathBuf,
    ws_tx: broadcast::Sender<WsMessage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    SyncState(SyncStateInfo),
    PlaybackProgress(PlaybackProgress),
    LibraryUpdate,
}

#[derive(Debug, Clone, Serialize)]
struct SyncStateInfo {
    last_sync_time: String,
    last_full_sync_time: Option<String>,
    last_sync_version: Option<String>,
    song_count: u32,
    artist_count: u32,
    album_count: u32,
}

#[derive(Debug, Clone, Serialize)]
struct PlaybackProgress {
    item_id: String,
    position: f64,
    duration: f64,
    is_playing: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Setup app data directory
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("./data"))
        .join("aurelia-web");
    
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create data directory");
    
    info!("Using data directory: {:?}", app_data_dir);

    // Initialize database
    if let Err(e) = aurelia_core::db::init(&app_data_dir) {
        warn!("Failed to initialize database: {}", e);
    }

    // WebSocket broadcast channel
    let (ws_tx, _ws_rx) = broadcast::channel::<WsMessage>(100);

    let state = Arc::new(AppState {
        app_data_dir,
        ws_tx,
    });

    // CORS for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Auth routes
        .route("/api/auth/credentials", get(get_credentials))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        // Library routes
        .route("/api/library", get(get_library))
        .route("/api/library/sync", post(sync_library))
        .route("/api/library/clear-cache", post(clear_cache))
        .route("/api/library/sync-state", get(get_sync_state))
        // Song routes
        .route("/api/songs/{song_id}", get(get_song))
        .route("/api/songs/{song_id}/favorite", post(toggle_favorite))
        .route("/api/songs/{song_id}/instant-mix", get(get_instant_mix))
        .route("/api/songs/{song_id}/share-urls", get(get_share_urls))
        // Playlist routes
        .route("/api/playlists", get(get_playlists).post(create_playlist))
        .route("/api/playlists/{playlist_id}", delete(delete_playlist).patch(update_playlist))
        .route("/api/playlists/{playlist_id}/items", get(get_playlist_items).post(add_playlist_items).delete(remove_playlist_items))
        // Home routes
        .route("/api/home", get(get_home_view))
        .route("/api/home/recently-played", get(get_recently_played))
        // Audio routes
        .route("/api/audio/{item_id}/stream-url", get(get_stream_url))
        .route("/api/audio/proxy", get(proxy_audio))
        // Lyrics routes
        .route("/api/lyrics", post(get_lyrics))
        // Artist routes
        .route("/api/artists/{artist_id}", get(get_artist))
        .route("/api/artists/{artist_id}/related", get(get_related_artists))
        // Playback reporting routes
        .route("/api/sessions/capabilities", post(register_capabilities))
        .route("/api/sessions/playing", post(report_playback))
        // Image routes
        .route("/api/images/{item_id}", get(get_image_url))
        // WebSocket
        .route("/ws", get(websocket_handler))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Auth handlers
async fn get_credentials(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(creds) => (StatusCode::OK, Json(ApiResponse::ok(creds))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match aurelia_core::authenticate(req.server_url.clone(), req.username.clone(), req.password).await {
        Ok(login_resp) => {
            // Convert LoginResponse to Credentials
            let creds = aurelia_core::models::Credentials {
                server_url: req.server_url,
                username: req.username,
                token: login_resp.token,
                user_id: login_resp.user_id,
            };
            // Save credentials
            let _ = aurelia_core::save_credentials(
                state.app_data_dir.to_string_lossy().to_string(),
                creds.clone(),
            );
            (StatusCode::OK, Json(ApiResponse::ok(creds)))
        }
        Err(e) => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn logout(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = aurelia_core::clear_credentials(state.app_data_dir.to_string_lossy().to_string());
    (StatusCode::OK, Json(ApiResponse::ok(())))
}

// Library handlers
// Library handlers
async fn get_library(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let songs = match aurelia_core::load_cached_songs(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(songs) => songs,
        Err(_) => vec![],
    };

    // Derive albums from songs
    let mut album_map: std::collections::HashMap<String, Vec<Song>> = std::collections::HashMap::new();
    for song in &songs {
        if let Some(album_id) = &song.album_id {
            album_map.entry(album_id.clone()).or_default().push(song.clone());
        }
    }

    let albums: Vec<Album> = album_map
        .iter()
        .filter_map(|(album_id, album_songs)| {
            let first_song = album_songs
                .iter()
                .max_by_key(|s| s.date_created.as_deref().unwrap_or(""))?;
            Some(Album {
                id: Some(album_id.clone()),
                name: first_song
                    .album
                    .clone()
                    .unwrap_or_else(|| "Unknown Album".to_string()),
                artist: first_song
                    .artists
                    .as_ref()
                    .and_then(|a| a.first())
                    .cloned()
                    .unwrap_or_else(|| "Unknown Artist".to_string()),
                artist_id: first_song
                    .artist_ids
                    .as_ref()
                    .and_then(|ids| ids.first())
                    .cloned(),
                album_art_url: first_song.album_art_url.clone(),
                song_count: album_songs.len() as i64,
                songs: None,
                image_tags: None,
                provider_ids: None,
                date_created: first_song.date_created.clone(),
                date_modified: None,
            })
        })
        .collect();

    // Derive artists from songs
    let mut artist_map: std::collections::HashMap<String, (String, Option<String>)> = std::collections::HashMap::new();
    for song in &songs {
        if let Some(artist_ids) = &song.artist_ids {
            let artist_names = song.artists.as_ref();
            for (i, artist_id) in artist_ids.iter().enumerate() {
                if !artist_map.contains_key(artist_id) {
                    let name = artist_names
                        .and_then(|names| names.get(i))
                        .cloned()
                        .unwrap_or_else(|| "Unknown Artist".to_string());
                    let image_url = if i == 0 {
                        song.album_art_url.clone()
                    } else {
                        None
                    };
                    artist_map.insert(artist_id.clone(), (name, image_url));
                }
            }
        }
    }

    let artists: Vec<Artist> = artist_map
        .into_iter()
        .map(|(id, (name, image_url))| Artist {
            id,
            name,
            image_url,
            image_tags: None,
            overview: None,
            provider_ids: None,
            community_rating: None,
            song_count: None,
            date_modified: None,
            songs: None,
        })
        .collect();

    let data = LibraryData {
        songs,
        albums,
        artists,
    };

    (StatusCode::OK, Json(ApiResponse::ok(data)))
}

async fn sync_library(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Get credentials first
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::sync_songs_only(
                creds.server_url,
                creds.token,
                creds.user_id,
                state.app_data_dir.to_string_lossy().to_string(),
            ).await {
                Ok(_) => {
                    let _ = state.ws_tx.send(WsMessage::LibraryUpdate);
                    (StatusCode::OK, Json(ApiResponse::ok(())))
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn clear_cache(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match aurelia_core::clear_cache(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(()))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn get_sync_state(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match aurelia_core::get_sync_state(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(sync_state) => {
            let info = SyncStateInfo {
                last_sync_time: sync_state.last_sync_time,
                last_full_sync_time: sync_state.last_full_sync_time,
                last_sync_version: sync_state.last_sync_version,
                song_count: sync_state.song_count,
                artist_count: sync_state.artist_count,
                album_count: sync_state.album_count,
            };
            (StatusCode::OK, Json(ApiResponse::ok(info)))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

// Song handlers
async fn get_song(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::get_cached_song(state.app_data_dir.to_string_lossy().to_string(), song_id) {
        Ok(Some(song)) => (StatusCode::OK, Json(ApiResponse::ok(song))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::err("Song not found".to_string()))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn toggle_favorite(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(req): Json<FavoriteRequest>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            // Frontend sends the target state (req.is_favorite).
            // But core's toggle_favorite takes CURRENT state and negates it.
            // So we pass the OPPOSITE of target state to core.
            match aurelia_core::toggle_favorite(
                creds.server_url,
                creds.token,
                creds.user_id,
                item_id.clone(),
                !req.is_favorite, 
            ).await {
                Ok(new_state) => {
                    // Update local cache as well to keep it in sync
                    let _ = aurelia_core::db::songs::update_favorite_status(
                        &item_id,
                        new_state
                    );
                    (StatusCode::OK, Json(ApiResponse::ok(new_state)))
                },
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn get_instant_mix(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::get_instant_mix(creds.server_url, creds.token, item_id).await {
                Ok(songs) => (StatusCode::OK, Json(ApiResponse::ok(songs))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn get_share_urls(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::get_cached_song(state.app_data_dir.to_string_lossy().to_string(), song_id) {
        Ok(Some(song)) => {
            match aurelia_core::get_song_share_urls(song).await {
                Ok(urls) => (StatusCode::OK, Json(ApiResponse::ok(urls))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::err("Song not found".to_string()))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn get_artist(
    State(state): State<Arc<AppState>>,
    Path(artist_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::fetch_artist(
                creds.server_url,
                creds.token,
                creds.user_id,
                artist_id,
                state.app_data_dir.to_string_lossy().to_string(),
            ).await {
                Ok(artist) => (StatusCode::OK, Json(ApiResponse::ok(artist))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

// Playlist handlers
async fn get_playlists(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::get_playlists(creds.server_url, creds.token, creds.user_id).await {
                Ok(playlists) => (StatusCode::OK, Json(ApiResponse::ok(playlists))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Json(req): Json<aurelia_core::models::PlaylistCreateData>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::create_playlist(creds.server_url, creds.token, req).await {
                Ok(playlist) => (StatusCode::OK, Json(ApiResponse::ok(playlist))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<String>,
    Json(req): Json<aurelia_core::models::PlaylistUpdateData>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::update_playlist(creds.server_url, creds.token, playlist_id, req).await {
                Ok(playlist) => (StatusCode::OK, Json(ApiResponse::ok(playlist))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::delete_playlist(creds.server_url, creds.token, playlist_id).await {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(()))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn get_playlist_items(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::get_playlist_items(creds.server_url, creds.token, playlist_id).await {
                Ok(items) => (StatusCode::OK, Json(ApiResponse::ok(items))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn add_playlist_items(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<String>,
    Json(req): Json<PlaylistItemsRequest>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::add_playlist_items(creds.server_url, creds.token, playlist_id, req.song_ids).await {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(()))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}

async fn remove_playlist_items(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<String>,
    Json(req): Json<PlaylistItemsRequest>,
) -> impl IntoResponse {
    match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::remove_playlist_items(creds.server_url, creds.token, playlist_id, req.song_ids).await {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(()))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
            }
        }
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string()))),
    }
}


async fn get_home_view(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let songs = match aurelia_core::load_cached_songs(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(songs) => songs,
        Err(_) => vec![],
    };

    if songs.is_empty() {
        return (StatusCode::OK, Json(ApiResponse::ok(HomeViewData {
            recently_played: vec![],
            recently_added: vec![],
            random_albums: vec![],
            featured_albums: vec![],
        })));
    }

    // Fetch recently played from Jellyfin if credentials exist
    let recently_played = match aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string()) {
        Ok(Some(creds)) => {
            match aurelia_core::get_recently_played(creds.server_url, creds.token, creds.user_id).await {
                Ok(songs) => songs,
                Err(e) => {
                    let error_str = e.to_string();
                    warn!("Failed to fetch recently played from Jellyfin: {}", error_str);
                    // Check if it's an auth error and return 401 to trigger re-login
                    if error_str.contains("401") || error_str.to_lowercase().contains("unauthorized") {
                        return (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Unauthorized - Please log in again".to_string())));
                    }
                    vec![]
                }
            }
        }
        Ok(None) => {
            warn!("No credentials found, cannot fetch recently played");
            return (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Not authenticated".to_string())));
        }
        Err(e) => {
            warn!("Failed to load credentials: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("Failed to load credentials".to_string())));
        }
    };

    // Derive albums from songs
    let mut album_map: std::collections::HashMap<String, Vec<Song>> = std::collections::HashMap::new();
    for song in &songs {
        if let Some(album_id) = &song.album_id {
            album_map.entry(album_id.clone()).or_default().push(song.clone());
        }
    }

    let derived_albums: Vec<Album> = album_map
        .iter()
        .filter_map(|(album_id, album_songs)| {
            let first_song = album_songs
                .iter()
                .max_by_key(|s| s.date_created.as_deref().unwrap_or(""))?;
            Some(Album {
                id: Some(album_id.clone()),
                name: first_song
                    .album
                    .clone()
                    .unwrap_or_else(|| "Unknown Album".to_string()),
                artist: first_song
                    .artists
                    .as_ref()
                    .and_then(|a| a.first())
                    .cloned()
                    .unwrap_or_else(|| "Unknown Artist".to_string()),
                artist_id: first_song
                    .artist_ids
                    .as_ref()
                    .and_then(|ids| ids.first())
                    .cloned(),
                album_art_url: first_song.album_art_url.clone(),
                song_count: album_songs.len() as i64,
                songs: Some(album_songs.clone()),
                image_tags: None,
                provider_ids: None,
                date_created: first_song.date_created.clone(),
                date_modified: None,
            })
        })
        .collect();

    // Sort by date_created for recently added
    let mut recently_added = derived_albums.clone();
    recently_added.sort_by(|a, b| {
        let date_a = a.date_created.as_deref().unwrap_or("");
        let date_b = b.date_created.as_deref().unwrap_or("");
        date_b.cmp(date_a)
    });

    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    
    let mut random_albums = derived_albums.clone();
    random_albums.shuffle(&mut rng);

    let mut featured_albums = derived_albums;
    featured_albums.shuffle(&mut rng);

    let data = HomeViewData {
        recently_played,
        recently_added: recently_added.into_iter().take(20).collect(),
        random_albums: random_albums.into_iter().take(20).collect(),
        featured_albums: featured_albums.into_iter().take(20).collect(),
    };

    (StatusCode::OK, Json(ApiResponse::ok(data)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentlyPlayedQuery {
    server_url: String,
    token: String,
    user_id: String,
}

async fn get_recently_played(
    Query(params): Query<RecentlyPlayedQuery>,
) -> impl IntoResponse {
    match aurelia_core::get_recently_played(params.server_url, params.token, params.user_id).await {
        Ok(songs) => (StatusCode::OK, Json(ApiResponse::ok(songs))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

#[derive(Debug, Deserialize)]
struct ProxyQuery {
    url: String,
}

async fn proxy_audio(
    headers: HeaderMap,
    Query(params): Query<ProxyQuery>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();
    
    let mut request = client.get(&params.url);
    
    // Forward Range header if present
    if let Some(range) = headers.get(header::RANGE) {
        request = request.header(header::RANGE, range);
    }
    
    let response = match request.send().await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let status = response.status();
    let mut response_headers = HeaderMap::new();
    
    // Copy relevant headers from original response
    if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        response_headers.insert(header::CONTENT_TYPE, content_type.clone());
    }
    if let Some(content_length) = response.headers().get(header::CONTENT_LENGTH) {
        response_headers.insert(header::CONTENT_LENGTH, content_length.clone());
    }
    if let Some(accept_ranges) = response.headers().get(header::ACCEPT_RANGES) {
        response_headers.insert(header::ACCEPT_RANGES, accept_ranges.clone());
    }
    if let Some(content_range) = response.headers().get(header::CONTENT_RANGE) {
        response_headers.insert(header::CONTENT_RANGE, content_range.clone());
    }

    let stream = response.bytes_stream();
    let body = Body::from_stream(stream);

    (status, response_headers, body).into_response()
}

// Audio handlers
async fn get_stream_url(
    Path(item_id): Path<String>,
    Query(params): Query<StreamUrlRequest>,
) -> impl IntoResponse {
    info!("get_stream_url request for item: {}", item_id);
    let url = aurelia_core::build_stream_url(
        params.server_url,
        params.token,
        item_id,
        params.container,
    );
    info!("Generated stream URL: {}", url);
    (StatusCode::OK, Json(ApiResponse::ok(url)))
}

// Lyrics handlers
async fn get_lyrics(Json(req): Json<LyricsRequest>) -> impl IntoResponse {
    // For LRCLIB, we don't need server/token/item_id from Jellyfin
    // but we still pass them for API compatibility
    let lyrics = aurelia_core::get_lyrics(
        "".to_string(),
        "".to_string(),
        req.id,
        req.artist,
        req.title,
    ).await;
    // Return empty string as null for consistency with frontend expectations
    let result = if lyrics.is_empty() { None } else { Some(lyrics) };
    (StatusCode::OK, Json(ApiResponse::ok(result)))
}

// Artist handlers
async fn get_related_artists(
    State(state): State<Arc<AppState>>,
    Path(artist_id): Path<String>,
) -> impl IntoResponse {
    match aurelia_core::get_related_artists(state.app_data_dir.to_string_lossy().to_string(), artist_id).await {
        Ok(artists) => (StatusCode::OK, Json(ApiResponse::ok(artists))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string()))),
    }
}

// Playback reporting handlers
async fn register_capabilities(Json(req): Json<CapabilitiesRequest>) -> impl IntoResponse {
    // For web, we don't need to register capabilities with Jellyfin in the same way
    // The frontend handles this directly. Just return success.
    info!("Register capabilities request for device: {}", req.device_id);
    (StatusCode::OK, Json(ApiResponse::ok(())))
}

async fn report_playback(Json(req): Json<PlaybackReportRequest>) -> impl IntoResponse {
    info!(
        "Playback report - item: {}, position: {:?}, event: {:?}, paused: {:?}",
        req.item_id, req.position_ticks, req.event_name, req.is_paused
    );
    
    // For web, playback reporting is handled by the frontend directly with Jellyfin
    // This endpoint is here for API compatibility with desktop
    (StatusCode::OK, Json(ApiResponse::ok(())))
}

// Image handler - generates Jellyfin image URL
async fn get_image_url(
    Path(item_id): Path<String>,
    Query(params): Query<ImageUrlRequest>,
) -> impl IntoResponse {
    let mut url = format!("{}/Items/{}/Images/{}", params.server_url, item_id, params.image_type);
    
    let query_params: Vec<String> = [
        Some(format!("api_key={}", params.token)),
        params.width.map(|w| format!("width={}", w)),
        params.quality.map(|q| format!("quality={}", q)),
    ]
    .into_iter()
    .flatten()
    .collect();
    
    if !query_params.is_empty() {
        url.push_str("?");
        url.push_str(&query_params.join("&"));
    }
    
    (StatusCode::OK, Json(ApiResponse::ok(url)))
}

// WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Send initial sync state
    if let Ok(sync_state) = aurelia_core::get_sync_state(state.app_data_dir.to_string_lossy().to_string()) {
        let msg = WsMessage::SyncState(SyncStateInfo {
            last_sync_time: sync_state.last_sync_time,
            last_full_sync_time: sync_state.last_full_sync_time,
            last_sync_version: sync_state.last_sync_version,
            song_count: sync_state.song_count,
            artist_count: sync_state.artist_count,
            album_count: sync_state.album_count,
        });
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    // Handle incoming messages and broadcast updates
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(msg) => {
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            recv = receiver.next() => {
                match recv {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
