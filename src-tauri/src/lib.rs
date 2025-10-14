pub mod cache;
pub mod database;
pub mod discord_rpc;
pub mod error;
pub mod handlers;
pub mod lastfm;
pub mod listenbrainz;
pub mod models;
pub mod services;
pub mod system_tray;
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

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        handlers::appearance::get_blur_mode,
        handlers::appearance::set_blur_mode,
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
        handlers::music::get_instant_mix,
        handlers::music::report_playback_start,
        handlers::music::report_playback_progress,
        handlers::music::report_playback_stop,
        handlers::music::mark_item_played,
        handlers::music::get_song_share_urls,
        handlers::music::get_album_share_urls,
        handlers::music::get_artist_share_urls,
        handlers::lyrics::get_lyrics,
        handlers::images::get_image,
        handlers::images::clear_image_cache,
        handlers::images::get_image_cache_stats,
        handlers::images::clear_image_from_cache,
        handlers::playlists::get_playlists,
        handlers::playlists::create_playlist,
        handlers::playlists::update_playlist,
        handlers::playlists::delete_playlist,
        handlers::playlists::add_playlist_items,
        handlers::playlists::remove_playlist_items,
        handlers::playlists::get_playlist_items,
        discord_rpc::discord_rpc_start,
        discord_rpc::discord_rpc_stop,
        discord_rpc::discord_rpc_is_running,
        discord_rpc::discord_rpc_set_activity,
        discord_rpc::discord_rpc_clear_activity,
        lastfm::lastfm_authenticate,
        lastfm::lastfm_scrobble,
        lastfm::lastfm_update_now_playing,
        lastfm::lastfm_set_credentials,
        lastfm::lastfm_clear_credentials,
        lastfm::lastfm_is_authenticated,
        lastfm::lastfm_start_auth_server,
        listenbrainz::listenbrainz_validate_token,
        listenbrainz::listenbrainz_submit_listen,
        listenbrainz::listenbrainz_playing_now,
        listenbrainz::listenbrainz_set_credentials,
        listenbrainz::listenbrainz_clear_credentials,
        listenbrainz::listenbrainz_is_authenticated,
        system_tray::show_main_window,
        system_tray::hide_main_window,
        system_tray::quit_application,
        system_tray::set_minimize_to_tray,
        system_tray::set_close_to_tray,
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .manage(discord_rpc::DiscordRpcState::new())
        .manage(lastfm::LastFmState::new())
        .manage(listenbrainz::ListenBrainzState::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            tauri::async_runtime::spawn(async move {
                if let Err(e) = cache::init().await {
                    error!("Failed to initialize cache: {}", e);
                }
            });

            if let Err(e) = system_tray::setup_system_tray(app.handle()) {
                error!("Failed to setup system tray: {}", e);
            }

            system_tray::setup_window_behavior(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
