//! Jellyfin Music Player - Rust Backend
//!
//! This is the main library for the Tauri application backend.
//! It provides a clean, modular interface to Jellyfin media server APIs.

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

// Re-export commonly used types for convenience
pub use models::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = db::initialize_database() {
        eprintln!("Failed to initialize database: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            handlers::login_to_jellyfin,
            handlers::save_credentials,
            handlers::get_saved_credentials,
            handlers::get_music_library,
            handlers::get_all_artists,
            handlers::get_artist_details,
            handlers::get_albums_with_songs,
            handlers::get_artists_with_songs,
            handlers::get_audio_stream_url,
            handlers::save_volume,
            handlers::get_saved_volume,
            handlers::toggle_favorite_status,
            handlers::clear_music_cache,
            handlers::get_lyrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
