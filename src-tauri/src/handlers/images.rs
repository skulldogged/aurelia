//! Image caching handlers
//!
//! This module provides Tauri commands for caching images locally
//! to improve performance and reduce network requests.

use crate::error::AppResult;
use base64::Engine;
use tauri::{AppHandle, Manager};
use tokio::fs;

/// Get the cache directory path for images
fn get_image_cache_dir(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    let cache_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::FileSystem(e.to_string()))?
        .join("image_cache");

    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }

    Ok(cache_dir)
}

fn generate_cache_key(item_id: &str, image_type: &str) -> String {
    format!("{}_{}", item_id, image_type)
}

#[tauri::command]
#[specta::specta]
pub async fn get_cached_image_data_url(
    app: AppHandle,
    item_id: String,
    image_type: String,
) -> Result<Option<String>, String> {
    let cache_dir = get_image_cache_dir(&app).map_err(|e| e.to_string())?;
    let cache_key = generate_cache_key(&item_id, &image_type);
    let cache_path = cache_dir.join(format!("{}.jpg", cache_key));

    if cache_path.exists() {
        let image_data = fs::read(&cache_path)
            .await
            .map_err(|e| format!("Failed to read cached image: {}", e))?;

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_data);
        let data_url = format!("data:image/jpeg;base64,{}", base64_data);

        Ok(Some(data_url))
    } else {
        Ok(None)
    }
}

#[tauri::command]
#[specta::specta]
pub async fn cache_image_from_url(
    app: AppHandle,
    item_id: String,
    image_type: String,
    image_url: String,
    _server_url: String,
    token: String,
) -> Result<String, String> {
    let cache_dir = get_image_cache_dir(&app).map_err(|e| e.to_string())?;
    let cache_key = generate_cache_key(&item_id, &image_type);
    let cache_path = cache_dir.join(format!("{}.jpg", cache_key));

    if cache_path.exists() {
        return Ok(cache_path.to_string_lossy().to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&image_url)
        .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
        .send()
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {}: {}",
            response.status(),
            response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error")
        ));
    }

    let image_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image data: {}", e))?;

    fs::write(&cache_path, &image_bytes)
        .await
        .map_err(|e| format!("Failed to save image to cache: {}", e))?;

    let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    let data_url = format!("data:image/jpeg;base64,{}", base64_data);
    Ok(data_url)
}

#[tauri::command]
#[specta::specta]
pub async fn clear_image_cache(app: AppHandle) -> Result<(), String> {
    let cache_dir = get_image_cache_dir(&app).map_err(|e| e.to_string())?;

    if cache_dir.exists() {
        let mut entries = fs::read_dir(&cache_dir)
            .await
            .map_err(|e| format!("Failed to read cache directory: {}", e))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            if entry.path().is_file() {
                fs::remove_file(entry.path())
                    .await
                    .map_err(|e| format!("Failed to remove cache file: {}", e))?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_image_cache_stats(app: AppHandle) -> Result<String, String> {
    let cache_dir = get_image_cache_dir(&app).map_err(|e| e.to_string())?;

    let mut total_size = 0u64;
    let mut file_count = 0u64;

    if cache_dir.exists() {
        let mut entries = fs::read_dir(&cache_dir)
            .await
            .map_err(|e| format!("Failed to read cache directory: {}", e))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            if entry.path().is_file()
                && let Ok(metadata) = entry.metadata().await
            {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }

    serde_json::to_string(&serde_json::json!({
        "total_size": total_size,
        "file_count": file_count,
        "cache_dir": cache_dir.to_string_lossy()
    }))
    .map_err(|e| format!("Failed to serialize cache stats: {}", e))
}
