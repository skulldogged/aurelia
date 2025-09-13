//! Main library entry point
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod cache;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                if cfg!(debug_assertions) {
                    "tauri_app=debug".into()
                } else {
                    "tauri_app=info".into()
                }
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    init_logging();
    info!("Starting Tauri application");
    // Initialize the cache system
    tauri::async_runtime::block_on(async {
        if let Err(e) = cache::init().await {
            error!("Failed to initialize cache: {}", e);
        }
    });

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        handlers::auth::login_to_jellyfin,
        handlers::auth::save_credentials,
        handlers::auth::get_saved_credentials,
        handlers::auth::save_volume,
        handlers::auth::get_saved_volume,
        handlers::music::get_songs,
        handlers::music::get_song,
        handlers::music::get_artists,
        handlers::music::get_artist,
        handlers::music::get_albums,
        handlers::music::get_album,
        handlers::music::get_audio_stream_url,
        handlers::music::toggle_favorite_status,
        handlers::music::sync_library,
        handlers::music::clear_cache,
        handlers::music::get_recently_played,
        handlers::music::register_client_capabilities,
        handlers::music::report_playback_start,
        handlers::music::report_playback_progress,
        handlers::music::report_playback_stop,
        handlers::music::mark_item_played,
        handlers::lyrics::get_lyrics,
        handlers::images::get_cached_image_data_url,
        handlers::images::cache_image_from_url,
        handlers::images::clear_image_cache,
        handlers::images::get_image_cache_stats,
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // This is also required if you want to use events
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
