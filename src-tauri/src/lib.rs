//! Main library entry point
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the database
    tauri::async_runtime::block_on(async {
        if let Err(e) = db::initialize_database().await {
            eprintln!("Failed to initialize database: {}", e);
        }
    });

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        handlers::auth::login_to_jellyfin,
        handlers::auth::save_credentials,
        handlers::auth::get_saved_credentials,
        handlers::auth::save_volume,
        handlers::auth::get_saved_volume,
        handlers::music::get_music_library,
        handlers::music::get_all_albums,
        handlers::music::get_all_artists,
        handlers::music::get_cached_artists,
        handlers::music::get_artists_with_songs,
        handlers::music::get_audio_stream_url,
        handlers::music::toggle_favorite_status,
        handlers::music::sync_music_library,
        handlers::music::clear_music_cache,
        handlers::music::get_recently_played,
        handlers::music::get_artist_details,
        handlers::lyrics::get_lyrics,
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // This is also required if you want to use events
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
