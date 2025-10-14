//! Playlist-related command handlers

use crate::models::music::{Playlist, PlaylistCreateData, PlaylistUpdateData};
use crate::services::JellyfinClient;

/// Get all playlists from Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn get_playlists() -> Result<Vec<Playlist>, String> {
    let client = get_jellyfin_client().await?;
    let user_id = get_current_user_id().await?;

    client
        .get_playlists(&user_id)
        .await
        .map_err(|e| format!("Failed to get playlists: {}", e))
}

/// Create a new playlist on Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn create_playlist(data: PlaylistCreateData) -> Result<Playlist, String> {
    let client = get_jellyfin_client().await?;

    client
        .create_playlist(&data)
        .await
        .map_err(|e| format!("Failed to create playlist: {}", e))
}

/// Update an existing playlist on Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn update_playlist(
    playlist_id: String,
    updates: PlaylistUpdateData,
) -> Result<Playlist, String> {
    let client = get_jellyfin_client().await?;

    client
        .update_playlist(&playlist_id, &updates)
        .await
        .map_err(|e| format!("Failed to update playlist: {}", e))
}

/// Delete a playlist from Jellyfin server
#[tauri::command]
#[specta::specta]
pub async fn delete_playlist(app: tauri::AppHandle, playlist_id: String) -> Result<(), String> {
    let client = get_jellyfin_client().await?;

    // Delete the playlist from the server
    client
        .delete_playlist(&playlist_id)
        .await
        .map_err(|e| format!("Failed to delete playlist: {}", e))?;

    // Clear the cached image for this playlist
    if let Err(e) = crate::handlers::images::clear_image_from_cache(
        app,
        playlist_id.clone(),
        "Primary".to_string(),
    )
    .await
    {
        tracing::warn!("Failed to delete cached playlist image: {}", e);
    }

    Ok(())
}

/// Add items to a playlist
#[tauri::command]
#[specta::specta]
pub async fn add_playlist_items(playlist_id: String, item_ids: Vec<String>) -> Result<(), String> {
    let client = get_jellyfin_client().await?;

    client
        .add_playlist_items(&playlist_id, &item_ids)
        .await
        .map_err(|e| format!("Failed to add items to playlist: {}", e))
}

/// Remove items from a playlist
#[tauri::command]
#[specta::specta]
pub async fn remove_playlist_items(
    playlist_id: String,
    item_ids: Vec<String>,
) -> Result<(), String> {
    let client = get_jellyfin_client().await?;

    client
        .remove_playlist_items(&playlist_id, &item_ids)
        .await
        .map_err(|e| format!("Failed to remove items from playlist: {}", e))
}

/// Get items in a playlist
#[tauri::command]
#[specta::specta]
pub async fn get_playlist_items(
    playlist_id: String,
) -> Result<Vec<crate::models::music::Song>, String> {
    let client = get_jellyfin_client().await?;

    client
        .get_playlist_items(&playlist_id)
        .await
        .map_err(|e| format!("Failed to get playlist items: {}", e))
}

/// Helper function to get authenticated Jellyfin client
pub async fn get_jellyfin_client() -> Result<JellyfinClient, String> {
    let creds = crate::handlers::auth::get_saved_credentials().await
        .map_err(|e| format!("No saved credentials found: {}", e))?
        .ok_or("No saved credentials found")?;

    Ok(JellyfinClient::with_auth(creds.server_url, creds.token))
}

/// Helper function to get current user ID
pub async fn get_current_user_id() -> Result<String, String> {
    let creds = crate::handlers::auth::get_saved_credentials().await
        .map_err(|e| format!("No saved credentials found: {}", e))?
        .ok_or("No saved credentials found")?;

    Ok(creds.user_id)
}
