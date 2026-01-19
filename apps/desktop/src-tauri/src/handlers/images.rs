//! Image caching handlers
//!
//! This module provides Tauri commands for caching images locally
//! to improve performance and reduce network requests.

use aurelia_core::error::{AppError, AppResult};
use tauri::{AppHandle, Manager};

use tokio::fs;

/// Get the cache directory path for images
fn get_image_cache_dir(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    let cache_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))?
        .join("image_cache");

    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }

    Ok(cache_dir)
}

fn generate_cache_key(
    item_id: &str,
    image_type: &str,
    width: Option<u32>,
    quality: Option<u32>,
) -> String {
    let mut key = format!("{item_id}_{image_type}");
    if let Some(w) = width {
        key.push_str(&format!("_w{w}"));
    }
    if let Some(q) = quality {
        key.push_str(&format!("_q{q}"));
    }
    key
}

#[tauri::command]
#[specta::specta]
pub async fn get_image(
    app: AppHandle,
    item_id: String,
    image_type: String,
    server_url: String,
    token: String,
    width: Option<u32>,
    quality: Option<u32>,
) -> Result<Option<String>, String> {
    let cache_dir = match get_image_cache_dir(&app) {
        Ok(dir) => dir,
        Err(e) => return Err(e.to_string()),
    };
    let cache_key = generate_cache_key(&item_id, &image_type, width, quality);
    let cache_path = cache_dir.join(format!("{cache_key}.jpg"));
    let marker_path = cache_dir.join(format!("{cache_key}.404"));

    if marker_path.exists() {
        return Ok(None);
    }

    if cache_path.exists() {
        let asset_url = cache_path.to_string_lossy().to_string();
        return Ok(Some(asset_url));
    }

    let mut image_url = format!(
        "{}/Items/{}/Images/{}",
        server_url.trim_end_matches('/'),
        item_id,
        image_type
    );

    if width.is_some() || quality.is_some() {
        image_url.push('?');
        if let Some(w) = width {
            image_url.push_str(&format!("width={}&", w));
        }
        if let Some(q) = quality {
            image_url.push_str(&format!("quality={}&", q));
        }
    }

    let client = reqwest::Client::new();
    let response = match client
        .get(&image_url)
        .header("Authorization", format!("MediaBrowser Token=\"{token}\""))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::debug!("Failed to download image: {}", e);
            return Ok(None);
        }
    };

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        if let Err(e) = fs::write(&marker_path, "").await {
            tracing::warn!("Failed to write 404 marker to cache: {}", e);
        }
        return Ok(None);
    }

    if !response.status().is_success() {
        tracing::debug!("Failed to download image, status: {}", response.status());
        return Ok(None);
    }

    let image_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!("Failed to read image bytes: {}", e);
            return Ok(None);
        }
    };

    if let Err(e) = fs::write(&cache_path, &image_bytes).await {
        tracing::warn!("Failed to write image to cache: {}", e);
        return Ok(None);
    }

    let asset_url = cache_path.to_string_lossy().to_string();

    Ok(Some(asset_url))
}

#[tauri::command]
#[specta::specta]
pub async fn clear_image_cache(app: AppHandle) -> Result<(), String> {
    let cache_dir = match get_image_cache_dir(&app) {
        Ok(dir) => dir,
        Err(e) => return Err(e.to_string()),
    };

    if cache_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&cache_dir).await {
            return Err(format!("Failed to clear image cache: {e}"));
        }
        if let Err(e) = fs::create_dir_all(&cache_dir).await {
            return Err(format!("Failed to recreate image cache directory: {e}"));
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_image_cache_stats(app: AppHandle) -> Result<String, String> {
    let cache_dir = match get_image_cache_dir(&app) {
        Ok(dir) => dir,
        Err(e) => return Err(e.to_string()),
    };

    let mut total_size = 0u64;
    let mut file_count = 0u64;
    let mut marker_count = 0u64;

    if cache_dir.exists() {
        let mut entries = match fs::read_dir(&cache_dir).await {
            Ok(entries) => entries,
            Err(e) => return Err(format!("Failed to read cache directory: {e}")),
        };
        while let Some(entry) = match entries.next_entry().await {
            Ok(entry) => entry,
            Err(e) => return Err(format!("Failed to read directory entry: {e}")),
        } {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
            {
                if ext == "jpg" {
                    if let Ok(metadata) = entry.metadata().await {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                } else if ext == "404" {
                    marker_count += 1;
                }
            }
        }
    }

    let stats = serde_json::json!({
        "total_size": total_size,
        "file_count": file_count,
        "marker_count": marker_count,
        "cache_dir": cache_dir.to_string_lossy()
    });

    serde_json::to_string(&stats).map_err(|e| format!("Failed to serialize cache stats: {e}"))
}

/// Delete a specific cached image by item ID and type
#[tauri::command]
#[specta::specta]
pub async fn clear_image_from_cache(
    app: AppHandle,
    item_id: String,
    image_type: String,
) -> Result<(), String> {
    let cache_dir = match get_image_cache_dir(&app) {
        Ok(dir) => dir,
        Err(e) => return Err(e.to_string()),
    };

    let prefix = format!("{item_id}_{image_type}");

    let mut entries = fs::read_dir(&cache_dir).await.map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with(&prefix) {
                if let Err(e) = fs::remove_file(&path).await {
                    tracing::warn!("Failed to delete cached file {:?}: {}", path, e);
                } else {
                    tracing::info!("Successfully deleted cached file: {:?}", path);
                }
            }
        }
    }

    Ok(())
}
