pub mod cache;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

pub use anyhow::Result;

#[cfg(debug_assertions)]
use specta_typescript::BigIntExportBehavior;
use specta_typescript::Typescript;
use std::process::Command;
use tauri_specta::{Builder, collect_commands};
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

#[cfg(debug_assertions)]
fn bunx_eslint_formatter(file: &std::path::Path) -> std::io::Result<()> {
    Command::new("bunx")
        .arg("eslint")
        .arg("--fix")
        .arg(file)
        .output()
        .map(|_| ())
        .map_err(std::io::Error::other)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::large_stack_frames)]
pub fn run() {
    init_logging();
    info!("Starting Tauri application");
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
        handlers::music::get_song_share_urls,
        handlers::music::get_album_share_urls,
        handlers::music::get_artist_share_urls,
        handlers::lyrics::get_lyrics,
        handlers::images::get_cached_image_data_url,
        handlers::images::cache_image_from_url,
        handlers::images::clear_image_cache,
        handlers::images::get_image_cache_stats,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            Typescript::default()
                .bigint(BigIntExportBehavior::BigInt)
                .formatter(bunx_eslint_formatter),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
