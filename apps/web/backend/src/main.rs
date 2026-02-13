use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::{
    Router,
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{info, warn};

// Re-export types from aurelia_core
use aurelia_api::Api;
use aurelia_api::axum_impl::{AppState, AxumApiImpl};
use aurelia_api::traits::axum_routes::build_router;
use aurelia_core::lastfm_core::LastFmState;
use aurelia_core::listenbrainz_core::ListenBrainzState;

/// Server state wrapper that includes WebSocket broadcaster
#[derive(Clone)]
struct ServerState {
    app_state: Arc<AppState>,
    ws_tx: broadcast::Sender<WsMessage>,
}

// WebSocket message types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    SyncState(aurelia_core::models::SyncStateInfo),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Parse CLI arguments / environment
    let host = std::env::var("AURELIA_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("AURELIA_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let static_dir = std::env::var("AURELIA_STATIC_DIR").ok().map(PathBuf::from);

    // Setup app data directory
    let app_data_dir = std::env::var("AURELIA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("./data"))
                .join("aurelia-web")
        });

    std::fs::create_dir_all(&app_data_dir).context("failed to create data directory")?;

    info!("Using data directory: {:?}", app_data_dir);

    // Initialize database
    if let Err(e) = aurelia_core::db::init(&app_data_dir) {
        warn!("Failed to initialize database: {}", e);
    }

    // WebSocket broadcast channel
    let (ws_tx, _ws_rx) = broadcast::channel::<WsMessage>(100);

    let app_state = Arc::new(AppState {
        app_data_dir: app_data_dir.clone(),
        listenbrainz_state: Arc::new(ListenBrainzState::new()),
        lastfm_state: Arc::new(LastFmState::new()),
    });

    let server_state = Arc::new(ServerState {
        app_state: app_state.clone(),
        ws_tx: ws_tx.clone(),
    });

    // CORS for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router from generated axum routes (uses Arc<AppState>)
    let api_router = build_router().with_state(app_state.clone());

    let mut app = Router::new()
        // Mount the generated API routes (already has its own state)
        .nest("/api", api_router)
        // Add WebSocket endpoint
        .route("/ws", get(websocket_handler))
        .layer(cors)
        .with_state(server_state.clone());

    // Optionally serve the frontend static files (SPA with index.html fallback)
    if let Some(ref dir) = static_dir {
        if dir.is_dir() {
            let index = dir.join("index.html");
            info!("Serving static files from: {:?}", dir);
            app = app.fallback_service(ServeDir::new(dir).not_found_service(ServeFile::new(index)));
        } else {
            warn!(
                "Static directory {:?} does not exist, not serving frontend",
                dir
            );
        }
    }

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .context("invalid host/port configuration")?;
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind TCP listener")?;
    axum::serve(listener, app)
        .await
        .context("axum server terminated with error")?;

    Ok(())
}

// WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: Arc<ServerState>) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Send initial sync state
    let api = AxumApiImpl::new(state.app_state.clone());
    if let Ok(sync_state) = api.get_sync_state().await {
        let msg = WsMessage::SyncState(sync_state);
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
                        if let Ok(json) = serde_json::to_string(&msg)
                            && sender.send(Message::Text(json.into())).await.is_err() {
                                break;
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
