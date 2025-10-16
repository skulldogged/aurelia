//! Utility functions and constants

use std::path::PathBuf;

pub mod error_handling;

/// Application constants
pub mod constants {
    /// Jellyfin client information
    pub const JELLYFIN_CLIENT: &str = "Aurelia";
    pub const JELLYFIN_DEVICE: &str = "Desktop";
    pub const JELLYFIN_VERSION: &str = "0.1.0";
    pub const JELLYFIN_DEVICE_ID: &str = "1";

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
    format!(
        "MediaBrowser Client=\"{}\", Device=\"{}\", DeviceId=\"{}\", Version=\"{}\", Token=\"\"",
        constants::JELLYFIN_CLIENT,
        constants::JELLYFIN_DEVICE,
        constants::JELLYFIN_DEVICE_ID,
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
