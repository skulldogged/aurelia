//! Authentication-related command handlers

use crate::models::{Credentials, LoginResponse};
use crate::services::JellyfinClient;
use crate::utils;
use crate::utils::error_handling;
use std::fs;
use tracing::error;

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

/// Save user credentials to disk
#[tauri::command]
#[specta::specta]
pub fn save_credentials(
    server_url: String,
    username: String,
    token: String,
    user_id: String,
) -> Result<(), String> {
    let app_dir = utils::ensure_app_data_dir()?;
    let credentials_path = app_dir.join("credentials.json");

    let credentials = Credentials {
        server_url,
        username,
        token,
        user_id,
    };

    let json = match serde_json::to_string_pretty(&credentials) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize credentials: {e}")),
    };

    fs::write(&credentials_path, json).map_err(|e| format!("Failed to save credentials: {e}"))
}

/// Load saved credentials from disk
#[tauri::command]
#[specta::specta]
pub fn get_saved_credentials() -> Result<Option<Credentials>, String> {
    let app_dir = match utils::get_app_data_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(format!("Application data directory not accessible: {e}")),
    };
    let credentials_path = app_dir.join("credentials.json");

    if !credentials_path.exists() {
        return Ok(None);
    }

    let json = match fs::read_to_string(&credentials_path) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to read saved credentials: {e}")),
    };

    let credentials: Credentials = match serde_json::from_str(&json) {
        Ok(credentials) => credentials,
        Err(e) => return Err(format!("Saved credentials are corrupted: {e}")),
    };

    Ok(Some(credentials))
}

/// Save user volume preference
#[tauri::command]
#[specta::specta]
pub fn save_volume(volume: f64) -> Result<(), String> {
    let app_dir = utils::ensure_app_data_dir()?;
    let volume_path = app_dir.join("volume.json");

    let json = match serde_json::to_string(&volume) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize volume: {e}")),
    };

    fs::write(&volume_path, json).map_err(|e| format!("Failed to save volume: {e}"))
}

/// Load saved volume preference
#[tauri::command]
#[specta::specta]
pub fn get_saved_volume() -> Result<Option<f64>, String> {
    let app_dir = utils::get_app_data_dir()?;
    let volume_path = app_dir.join("volume.json");

    if !volume_path.exists() {
        return Ok(None);
    }

    let json = match fs::read_to_string(&volume_path) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to read volume: {e}")),
    };

    let volume: f64 = match serde_json::from_str(&json) {
        Ok(volume) => volume,
        Err(e) => return Err(format!("Failed to parse volume: {e}")),
    };

    Ok(Some(volume))
}
