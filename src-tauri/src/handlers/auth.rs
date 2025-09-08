//! Authentication-related command handlers

use crate::models::{Credentials, LoginResponse};
use crate::services::JellyfinClient;
use crate::utils;
use std::fs;

/// Login to Jellyfin server
#[tauri::command]
pub async fn login_to_jellyfin(
    server_url: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let client = JellyfinClient::new(server_url);
    client
        .authenticate(&username, &password)
        .await
        .map_err(|e| e.to_string())
}

/// Save user credentials to disk
#[tauri::command]
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

    let json = serde_json::to_string_pretty(&credentials)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    fs::write(&credentials_path, json).map_err(|e| format!("Failed to save credentials: {}", e))
}

/// Load saved credentials from disk
#[tauri::command]
pub fn get_saved_credentials() -> Result<Option<Credentials>, String> {
    let app_dir = utils::get_app_data_dir()?;
    let credentials_path = app_dir.join("credentials.json");

    if !credentials_path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&credentials_path)
        .map_err(|e| format!("Failed to read credentials: {}", e))?;

    let credentials: Credentials =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse credentials: {}", e))?;

    Ok(Some(credentials))
}

/// Save user volume preference
#[tauri::command]
pub fn save_volume(volume: f64) -> Result<(), String> {
    let app_dir = utils::ensure_app_data_dir()?;
    let volume_path = app_dir.join("volume.json");

    let json =
        serde_json::to_string(&volume).map_err(|e| format!("Failed to serialize volume: {}", e))?;

    fs::write(&volume_path, json).map_err(|e| format!("Failed to save volume: {}", e))
}

/// Load saved volume preference
#[tauri::command]
pub fn get_saved_volume() -> Result<Option<f64>, String> {
    let app_dir = utils::get_app_data_dir()?;
    let volume_path = app_dir.join("volume.json");

    if !volume_path.exists() {
        return Ok(None);
    }

    let json =
        fs::read_to_string(&volume_path).map_err(|e| format!("Failed to read volume: {}", e))?;

    let volume: f64 =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse volume: {}", e))?;

    Ok(Some(volume))
}
