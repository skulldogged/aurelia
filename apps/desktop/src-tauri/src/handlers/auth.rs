//! Authentication-related command handlers

use aurelia_core::cache;
use aurelia_core::models::{Credentials, LoginResponse};
use aurelia_core::services::JellyfinClient;
use aurelia_core::state::AppState;
use aurelia_core::utils::error_handling;
use tauri::{Manager, State};

use tracing::{error, info, warn};

/// Login to Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn login_to_jellyfin(
    server_url: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let client = JellyfinClient::new(server_url.clone());
    match client.authenticate(&username, &password).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!(
                "Authentication failed for user '{}' on server '{}': {}",
                username, server_url, e
            );
            Err(error_handling::error_to_user_message(&e))
        }
    }
}

/// Save user credentials to redb and cache in memory
#[tauri::command]
#[specta::specta]
pub async fn save_credentials(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
    server_url: String,
    username: String,
    token: String,
    user_id: String,
) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Application data directory not accessible: {e}"))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app directory: {e}"))?;

    let credentials = Credentials {
        server_url,
        username,
        token,
        user_id,
    };

    // Cache in memory for fast access
    app_state.set_credentials(Some(credentials.clone()));

    // Save to redb
    cache::save_credentials(app_dir, &credentials)
        .map_err(|e| format!("Failed to save credentials: {e}"))
}

/// Load saved credentials - checks memory cache first, then redb (with migration from legacy file)
#[tauri::command]
#[specta::specta]
pub async fn get_saved_credentials(
    app: tauri::AppHandle,
    _app_state: State<'_, AppState>,
) -> Result<Option<Credentials>, String> {
    get_credentials_cached(&app).await
}

/// Internal helper for getting cached credentials
/// Can be called from other handlers that have access to AppHandle
pub async fn get_credentials_cached(app: &tauri::AppHandle) -> Result<Option<Credentials>, String> {
    let app_state: tauri::State<'_, AppState> = app.state();

    // Check memory cache first (fast path)
    if let Some(creds) = app_state.get_credentials() {
        return Ok(Some(creds));
    }

    // Cache miss - load from redb
    let app_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(format!("Application data directory not accessible: {e}")),
    };

    // Try to load from redb first
    match cache::load_credentials(app_dir.clone()) {
        Ok(Some(credentials)) => {
            // Cache for future requests
            app_state.set_credentials(Some(credentials.clone()));
            return Ok(Some(credentials));
        }
        Ok(None) => {
            // Not found in redb, try migration from legacy file
        }
        Err(e) => {
            warn!("Failed to load credentials from redb: {e}");
        }
    }

    // Try to migrate from legacy credentials.json
    let credentials_path = app_dir.join("credentials.json");
    if credentials_path.exists() {
        match tokio::fs::read_to_string(&credentials_path).await {
            Ok(json) => match serde_json::from_str::<Credentials>(&json) {
                Ok(credentials) => {
                    info!("Migrating credentials from legacy credentials.json to redb");
                    // Save to redb
                    if let Err(e) = cache::save_credentials(app_dir, &credentials) {
                        warn!("Failed to migrate credentials to redb: {e}");
                    } else {
                        // Delete old file after successful migration
                        if let Err(e) = tokio::fs::remove_file(&credentials_path).await {
                            warn!("Failed to remove legacy credentials.json: {e}");
                        } else {
                            info!("Successfully migrated credentials and removed legacy file");
                        }
                    }
                    // Cache for future requests
                    app_state.set_credentials(Some(credentials.clone()));
                    return Ok(Some(credentials));
                }
                Err(e) => {
                    warn!("Legacy credentials.json is corrupted: {e}");
                }
            },
            Err(e) => {
                warn!("Failed to read legacy credentials.json: {e}");
            }
        }
    }

    Ok(None)
}

/// Clear saved credentials from redb and memory cache
#[tauri::command]
#[specta::specta]
pub async fn clear_saved_credentials(
    app: tauri::AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Clear memory cache
    app_state.set_credentials(None);

    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Application data directory not accessible: {e}"))?;

    // Clear from redb
    cache::clear_credentials(app_dir)
        .map_err(|e| format!("Failed to clear credentials: {e}"))
}

/// Save user volume preference
#[tauri::command]
#[specta::specta]
pub async fn save_volume(app: tauri::AppHandle, volume: f64) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Application data directory not accessible: {e}"))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app directory: {e}"))?;
    let volume_path = app_dir.join("volume.json");

    let json = match serde_json::to_string(&volume) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize volume: {e}")),
    };

    tokio::fs::write(&volume_path, json)
        .await
        .map_err(|e| format!("Failed to save volume: {e}"))
}

/// Load saved volume preference
#[tauri::command]
#[specta::specta]
pub async fn get_saved_volume(app: tauri::AppHandle) -> Result<Option<f64>, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Application data directory not accessible: {e}"))?;
    let volume_path = app_dir.join("volume.json");

    if !volume_path.exists() {
        return Ok(None);
    }

    let json = match tokio::fs::read_to_string(&volume_path).await {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to read volume: {e}")),
    };

    let volume: f64 = match serde_json::from_str(&json) {
        Ok(volume) => volume,
        Err(e) => return Err(format!("Failed to parse volume: {e}")),
    };

    Ok(Some(volume))
}
