//! Utility functions and constants

use serde_json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub mod error_handling;
pub mod lyrics;

/// Application constants
pub mod constants {
    /// Application version - single source of truth
    pub const APP_VERSION: &str = "0.1.0";

    /// Jellyfin client information
    pub const JELLYFIN_CLIENT: &str = "Aurelia";
    pub const JELLYFIN_DEVICE: &str = "Desktop";
    pub const JELLYFIN_VERSION: &str = APP_VERSION;

    /// `LrcLib` API endpoint
    pub const LRCLIB_SEARCH_URL: &str = "https://lrclib.net/api/search";

    /// Application directory name
    pub const APP_DIR_NAME: &str = "Aurelia";

    /// Audio containers that support seeking
    pub const SEEKABLE_CONTAINERS: &[&str] = &["flac", "mp3", "aac", "ogg"];
}

/// Get the application data directory
pub fn get_app_data_dir() -> Result<PathBuf, String> {
    let mut app_dir = dirs::data_dir().ok_or("Failed to get data directory")?;
    app_dir.push(constants::APP_DIR_NAME);
    Ok(app_dir)
}

/// Create the application data directory if it doesn't exist
pub fn ensure_app_data_dir() -> Result<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app directory: {e}"))?;
    Ok(app_dir)
}

/// Get or create a persistent device ID
fn get_device_id() -> Result<String, String> {
    let app_dir = ensure_app_data_dir()?;
    let device_id_path = app_dir.join("device_id.json");

    if device_id_path.exists() {
        let content = fs::read_to_string(&device_id_path)
            .map_err(|e| format!("Failed to read device ID: {e}"))?;
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse device ID JSON: {e}"))?;
        if let Some(id) = json["device_id"].as_str() {
            return Ok(id.to_string());
        }
    }

    let new_id = Uuid::new_v4().to_string();
    let json = serde_json::json!({ "device_id": new_id });
    fs::write(
        &device_id_path,
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .map_err(|e| format!("Failed to save device ID: {e}"))?;

    Ok(new_id)
}

/// Check if an audio container supports seeking
#[must_use]
pub fn supports_seeking(container: Option<&str>) -> bool {
    container.is_some_and(|c| {
        let c_lower = c.to_lowercase();
        constants::SEEKABLE_CONTAINERS
            .iter()
            .any(|&supported| supported == c_lower)
    })
}

/// Build Jellyfin authorization header
#[must_use]
pub fn build_jellyfin_auth_header() -> String {
    let device_id = get_device_id().unwrap_or_else(|_| "Aurelia-Fallback".to_string());
    format!(
        "MediaBrowser Client=\"{}\", Device=\"{}\", DeviceId=\"{}\", Version=\"{}\", Token=\"\"",
        constants::JELLYFIN_CLIENT,
        constants::JELLYFIN_DEVICE,
        device_id,
        constants::JELLYFIN_VERSION
    )
}

/// Build Jellyfin API URL
#[must_use]
pub fn build_jellyfin_url(server_url: &str, endpoint: &str) -> String {
    format!(
        "{}/{}",
        server_url.trim_end_matches('/'),
        endpoint.trim_start_matches('/')
    )
}

/// Pagination utilities
pub mod pagination {
    /// Apply pagination to a vector using optional offset and limit
    #[must_use]
    pub fn apply_pagination<T>(
        mut items: Vec<T>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Vec<T> {
        // Convert to usize safely - validation ensures non-negative values
        #[allow(clippy::cast_sign_loss)]
        if let Some(offset) = offset.filter(|&o| o >= 0) {
            items = items.into_iter().skip(offset as usize).collect();
        }

        #[allow(clippy::cast_sign_loss)]
        if let Some(limit) = limit.filter(|&l| l >= 0) {
            items = items.into_iter().take(limit as usize).collect();
        }

        items
    }

    /// Check if pagination parameters are valid
    pub fn validate_pagination(offset: Option<i32>, limit: Option<i32>) -> Result<(), String> {
        if let Some(offset) = offset
            && offset < 0
        {
            return Err("Offset must be non-negative".to_string());
        }

        if let Some(limit) = limit
            && limit <= 0
        {
            return Err("Limit must be positive".to_string());
        }

        Ok(())
    }
}
