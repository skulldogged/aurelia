//! Aurelia Sidecar Lyrics Daemon
//!
//! A lightweight HTTP server that serves word-synced lyrics from sidecar files
//! for Jellyfin music libraries.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use clap::Parser;
use dashmap::DashMap;
use serde::Serialize;
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, warn};

use aurelia_lyrics::{parse_auto, parse_ttml, ParsedLyrics};

mod config;
mod jellyfin;
mod lrclib;

use config::Config;
use jellyfin::JellyfinClient;
use lrclib::LrcLibClient;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "aurelia-sidecar-daemon")]
#[command(about = "Lightweight sidecar lyrics daemon for Jellyfin")]
struct Args {
    /// Path to config file
    #[arg(short, long, env = "AURELIA_CONFIG")]
    config: Option<PathBuf>,

    /// Jellyfin server URL
    #[arg(short, long, env = "JELLYFIN_URL")]
    jellyfin_url: Option<String>,

    /// Jellyfin API key
    #[arg(short = 'k', long, env = "JELLYFIN_API_KEY")]
    jellyfin_api_key: Option<String>,

    /// Music library paths (comma-separated)
    #[arg(short, long, env = "MUSIC_PATHS")]
    music_paths: Option<String>,

    /// HTTP server port
    #[arg(short, long, default_value = "8080", env = "PORT")]
    port: u16,

    /// Bind address
    #[arg(long, default_value = "0.0.0.0", env = "BIND")]
    bind: String,
}

/// Application state shared across handlers
struct AppState {
    config: Config,
    jellyfin: Option<JellyfinClient>,
    lrclib: LrcLibClient,
    cache: Arc<DashMap<String, CachedLyrics>>,
}

#[derive(Clone)]
struct CachedLyrics {
    lyrics: ParsedLyrics,
    source: LyricsSource,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum LyricsSource {
    Sidecar(String),
    Jellyfin,
    LrcLib,
}

/// API response for lyrics
#[derive(Serialize)]
struct LyricsResponse {
    item_id: String,
    found: bool,
    source: Option<String>,
    lyrics: Option<ParsedLyrics>,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: &'static str,
    jellyfin_connected: bool,
    cache_size: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Load or create config
    let config = if let Some(config_path) = args.config.clone() {
        Config::from_file_and_args(config_path, &args).await?
    } else {
        Config::from_args(&args).await?
    };

    info!("Starting Aurelia Sidecar Lyrics Daemon");
    info!("Jellyfin URL: {:?}", config.jellyfin_url);
    info!("Music paths: {:?}", config.music_paths);
    info!("Listening on {}:{}", config.bind, config.port);

    // Initialize Jellyfin client
    let jellyfin =
        if let (Some(url), Some(api_key)) = (&config.jellyfin_url, &config.jellyfin_api_key) {
            match JellyfinClient::new(url.clone(), api_key.clone()).await {
                Ok(client) => {
                    info!("Connected to Jellyfin server");
                    Some(client)
                }
                Err(e) => {
                    warn!("Failed to connect to Jellyfin: {}", e);
                    None
                }
            }
        } else {
            warn!("Jellyfin URL or API key not provided; will use file path mapping only");
            None
        };

    // Create shared state
    let state = Arc::new(AppState {
        config,
        jellyfin,
        lrclib: LrcLibClient::new(),
        cache: Arc::new(DashMap::new()),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/lyrics/{item_id}", get(get_lyrics_handler))
        .route("/lyrics/{item_id}/raw", get(get_raw_lyrics_handler))
        .route("/cache/clear", get(clear_cache_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state.clone());

    // Start server
    let addr: SocketAddr = format!("{}:{}", state.config.bind, state.config.port).parse()?;

    info!("Server ready at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION"),
        jellyfin_connected: state.jellyfin.is_some(),
        cache_size: state.cache.len(),
    })
}

/// Get parsed lyrics for a Jellyfin item
async fn get_lyrics_handler(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> Result<Json<LyricsResponse>, StatusCode> {
    // Check cache first
    if let Some(cached) = state.cache.get(&item_id) {
        debug!("Cache hit for {}", item_id);
        return Ok(Json(LyricsResponse {
            item_id,
            found: true,
            source: Some(format!("{:?}", cached.source)),
            lyrics: Some(cached.lyrics.clone()),
        }));
    }

    // Get item info from Jellyfin (path, artist, album, etc.)
    let item_info = if let Some(ref jellyfin) = state.jellyfin {
        match jellyfin.get_item_info(&item_id).await {
            Ok(info) => Some(info),
            Err(e) => {
                warn!("Failed to get item info from Jellyfin: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Try to find sidecar files
    if let Some(ref info) = item_info {
        if let Some(ref path) = info.path {
            match try_read_sidecar_lyrics(path).await {
                Ok(lyrics) => {
                    debug!("Found sidecar lyrics for {} at {:?}", item_id, path);

                    let source = LyricsSource::Sidecar(path.to_string_lossy().to_string());
                    state.cache.insert(
                        item_id.clone(),
                        CachedLyrics {
                            lyrics: lyrics.clone(),
                            source: source.clone(),
                        },
                    );

                    return Ok(Json(LyricsResponse {
                        item_id,
                        found: true,
                        source: Some(format!("{:?}", source)),
                        lyrics: Some(lyrics),
                    }));
                }
                Err(e) => {
                    debug!("No sidecar lyrics found for {}: {}", item_id, e);
                }
            }
        }
    }

    // Fallback: Try Jellyfin's built-in lyrics API
    if let Some(ref jellyfin) = state.jellyfin {
        match jellyfin.get_lyrics(&item_id).await {
            Ok(lyrics) => {
                debug!("Found lyrics from Jellyfin API for {}", item_id);

                state.cache.insert(
                    item_id.clone(),
                    CachedLyrics {
                        lyrics: lyrics.clone(),
                        source: LyricsSource::Jellyfin,
                    },
                );

                return Ok(Json(LyricsResponse {
                    item_id,
                    found: true,
                    source: Some("Jellyfin".to_string()),
                    lyrics: Some(lyrics),
                }));
            }
            Err(e) => {
                debug!("No lyrics from Jellyfin API for {}: {}", item_id, e);
            }
        }
    }

    // Fallback: Try LrcLib if we have artist/title info
    if let Some(ref info) = item_info {
        if let Some(artist) = &info.artist {
            let duration_ms = info.run_time_ticks.map(|t| t / 10_000);

            match state
                .lrclib
                .get(
                    artist.as_str(),
                    &info.name,
                    info.album.as_deref(),
                    duration_ms,
                )
                .await
            {
                Ok(Some(lyrics)) => {
                    debug!("Found lyrics from LrcLib for {}", item_id);

                    state.cache.insert(
                        item_id.clone(),
                        CachedLyrics {
                            lyrics: lyrics.clone(),
                            source: LyricsSource::LrcLib,
                        },
                    );

                    return Ok(Json(LyricsResponse {
                        item_id,
                        found: true,
                        source: Some("LrcLib".to_string()),
                        lyrics: Some(lyrics),
                    }));
                }
                Ok(None) => {
                    debug!("No lyrics found on LrcLib for {}", item_id);
                }
                Err(e) => {
                    debug!("LrcLib error for {}: {}", item_id, e);
                }
            }
        }
    }

    // No lyrics found
    Ok(Json(LyricsResponse {
        item_id,
        found: false,
        source: None,
        lyrics: None,
    }))
}

/// Get raw lyrics content (for debugging)
async fn get_raw_lyrics_handler(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> Result<String, StatusCode> {
    if let Some(ref jellyfin) = state.jellyfin {
        match jellyfin.get_item_info(&item_id).await {
            Ok(info) => {
                if let Some(path) = info.path {
                    if let Some(sidecar_path) = find_sidecar_file(&path).await {
                        match fs::read_to_string(&sidecar_path).await {
                            Ok(content) => Ok(content),
                            Err(_) => Err(StatusCode::NOT_FOUND),
                        }
                    } else {
                        Err(StatusCode::NOT_FOUND)
                    }
                } else {
                    Err(StatusCode::NOT_FOUND)
                }
            }
            Err(_) => Err(StatusCode::NOT_FOUND),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Clear the lyrics cache
async fn clear_cache_handler(State(state): State<Arc<AppState>>) -> String {
    let size = state.cache.len();
    state.cache.clear();
    format!("Cleared {} cached entries", size)
}

/// Try to read lyrics from sidecar files
async fn try_read_sidecar_lyrics(audio_path: &FsPath) -> anyhow::Result<ParsedLyrics> {
    if let Some(sidecar_path) = find_sidecar_file(audio_path).await {
        let content = fs::read_to_string(&sidecar_path).await?;
        let extension = sidecar_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");

        let lyrics = if extension == "ttml" {
            parse_ttml(&content)?
        } else {
            parse_auto(&content, extension)?
        };

        if lyrics.is_empty() {
            anyhow::bail!("Parsed lyrics are empty");
        }

        Ok(lyrics)
    } else {
        anyhow::bail!("No sidecar file found")
    }
}

/// Find the first available sidecar file for an audio file
async fn find_sidecar_file(audio_path: &FsPath) -> Option<PathBuf> {
    let parent = audio_path.parent()?;
    let stem = audio_path.file_stem()?.to_str()?;

    // Priority order: TTML > LRC > ELRC > TXT
    let extensions = ["ttml", "lrc", "elrc", "txt"];

    for ext in &extensions {
        let sidecar_path = parent.join(format!("{}.{}", stem, ext));
        if sidecar_path.exists() {
            return Some(sidecar_path);
        }
    }

    None
}
