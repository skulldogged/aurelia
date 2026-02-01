pub mod system_tray;

pub use anyhow::Result;

use aurelia_api::traits::tauri_commands;
use aurelia_api::Api;
use aurelia_api::tauri_impl::TauriApiImpl;
use aurelia_core::{db, listenbrainz_core, state};
use aurelia_core::audio;
use aurelia_core::{discord_rpc, media_controls};
use std::sync::Once;
use tauri::Manager;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT_LOGGING: Once = Once::new();

fn init_logging() {
    INIT_LOGGING.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "aurelia=debug,tauri=info,warn".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
}

#[allow(clippy::large_stack_frames)]
pub fn run() {
    init_logging();
    info!("Starting Tauri application");

    #[allow(unused_mut)]
    let mut tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .manage(listenbrainz_core::ListenBrainzState::new())
        .manage(state::AppState::new())
        .manage(audio::AudioState::new())
        .manage(discord_rpc::DiscordRpcState::new())
        .manage(media_controls::MediaControlsState::new());

    // Register all commands from the unified API
    tauri_builder
        .invoke_handler(tauri::generate_handler![
            // Auth
            tauri_commands::login_to_jellyfin,
            tauri_commands::save_credentials,
            tauri_commands::get_saved_credentials,
            tauri_commands::clear_saved_credentials,
            tauri_commands::save_volume,
            tauri_commands::get_saved_volume,
            // Library
            tauri_commands::get_library,
            tauri_commands::sync_library,
            tauri_commands::get_sync_state,
            // Songs
            tauri_commands::get_song,
            tauri_commands::toggle_favorite_status,
            tauri_commands::get_instant_mix,
            tauri_commands::get_song_share_urls,
            // Artists
            tauri_commands::get_artist,
            tauri_commands::get_related_artists,
            tauri_commands::get_artist_share_urls,
            // Albums
            tauri_commands::get_album,
            tauri_commands::get_album_share_urls,
            // Playlists
            tauri_commands::get_playlists,
            tauri_commands::get_playlist_items,
            tauri_commands::create_playlist,
            tauri_commands::update_playlist,
            tauri_commands::delete_playlist,
            tauri_commands::add_playlist_items,
            tauri_commands::remove_playlist_items,
            // Home
            tauri_commands::get_home_view_data,
            tauri_commands::get_recently_played,
            // Images
            tauri_commands::get_image,
            tauri_commands::clear_image_cache,
            tauri_commands::get_image_cache_stats,
            tauri_commands::clear_image_from_cache,
            // Audio streaming
            tauri_commands::get_audio_stream_url,
            // Lyrics
            tauri_commands::get_lyrics,
            // Cache
            tauri_commands::clear_cache,
            // Session / Playback
            tauri_commands::register_client_capabilities,
            tauri_commands::report_playback_start,
            tauri_commands::report_playback_progress,
            tauri_commands::report_playback_stop,
            tauri_commands::mark_item_played,
            // ListenBrainz
            tauri_commands::listenbrainz_set_credentials,
            tauri_commands::listenbrainz_clear_credentials,
            tauri_commands::listenbrainz_is_authenticated,
            tauri_commands::listenbrainz_validate_token,
            tauri_commands::listenbrainz_submit_listen,
            tauri_commands::listenbrainz_playing_now,
            // Audio
            tauri_commands::audio_init,
            tauri_commands::audio_play,
            tauri_commands::audio_pause,
            tauri_commands::audio_resume,
            tauri_commands::audio_stop,
            tauri_commands::audio_get_volume,
            tauri_commands::audio_set_volume,
            tauri_commands::audio_seek,
            tauri_commands::audio_get_position,
            tauri_commands::audio_is_playing,
            tauri_commands::audio_is_finished,
            tauri_commands::audio_advance_gapless,
            tauri_commands::audio_prepare_next,
            tauri_commands::audio_set_eq_enabled,
            tauri_commands::audio_is_eq_enabled,
            tauri_commands::audio_set_eq_band,
            tauri_commands::audio_get_eq_band,
            tauri_commands::audio_get_all_eq_bands,
            tauri_commands::audio_reset_eq,
            tauri_commands::audio_set_analyzer_enabled,
            tauri_commands::audio_is_analyzer_enabled,
            tauri_commands::audio_reinit,
            // Discord RPC
            tauri_commands::discord_rpc_start,
            tauri_commands::discord_rpc_stop,
            tauri_commands::discord_rpc_is_running,
            tauri_commands::discord_rpc_set_activity,
            tauri_commands::discord_rpc_clear_activity,
            // Media controls
            tauri_commands::media_update_now_playing,
            tauri_commands::media_clear_now_playing,
            tauri_commands::media_set_playback_status,
            tauri_commands::media_set_button_enabled,
            // Last.fm
            tauri_commands::lastfm_set_credentials,
            tauri_commands::lastfm_clear_credentials,
            tauri_commands::lastfm_is_authenticated,
            tauri_commands::lastfm_start_auth_server,
            tauri_commands::lastfm_authenticate,
            tauri_commands::lastfm_scrobble,
            tauri_commands::lastfm_update_now_playing,
            // Window management
            tauri_commands::show_main_window,
            tauri_commands::hide_main_window,
            tauri_commands::quit_application,
            tauri_commands::set_minimize_to_tray,
            tauri_commands::set_close_to_tray,
        ])
        .setup(move |app| {
            let handle = app.handle();
            info!("Setting up application...");

            info!("Initializing database...");
            let app_data_dir = handle
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            if let Err(e) = db::init(&app_data_dir) {
                error!("Failed to initialize database: {}", e);
            }
            info!("Database initialized.");

            let app_state = handle.state::<state::AppState>();
            let songs = app_state.songs.clone();
            let artists = app_state.artists.clone();
            let albums = app_state.albums.clone();

            let handle_for_async = handle.clone();
            tauri::async_runtime::spawn(async move {
                info!("Starting background library load...");
                let handle = handle_for_async;
                tauri::async_runtime::spawn_blocking(move || {
                    let songs_res = db::songs::get_all();
                    let artists_res = db::artists::get_all();
                    let albums_res = db::albums::get_all();

                    match (songs_res, artists_res, albums_res) {
                        (Ok(s), Ok(ar), Ok(al)) => {
                            info!(
                                "Loaded {} songs, {} artists, and {} albums from database",
                                s.len(),
                                ar.len(),
                                al.len()
                            );

                            *songs.lock().unwrap() = s;
                            *artists.lock().unwrap() = ar;
                            *albums.lock().unwrap() = al;

                            info!("Triggering library sync on startup.");
                            let handle = handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let api = TauriApiImpl::new(handle);
                                if let Ok(Some(_creds)) = api.get_saved_credentials().await {
                                    if let Err(e) = api.sync_library().await {
                                        error!("Failed to sync library: {}", e);
                                    }
                                }
                            });
                        }
                        (Err(e), _, _) => error!("Failed to load songs from database: {}", e),
                        (_, Err(e), _) => error!("Failed to load artists from database: {}", e),
                        (_, _, Err(e)) => error!("Failed to load albums from database: {}", e),
                    }
                    info!("Background library load finished.");
                });
            });

            if let Err(e) = system_tray::setup_system_tray(handle) {
                error!("Failed to setup system tray: {}", e);
            }
            system_tray::setup_window_behavior(handle);

            // Initialize OS media controls (SMTC on Windows, MPRIS on Linux, etc.)
            let media_state = handle.state::<media_controls::MediaControlsState>();
            let hwnd = {
                #[cfg(target_os = "windows")]
                {
                    handle
                        .get_webview_window("main")
                        .and_then(|w| {
                            use raw_window_handle::HasWindowHandle;
                            let handle = w.window_handle().ok()?;
                            match handle.as_raw() {
                                raw_window_handle::RawWindowHandle::Win32(h) => {
                                    Some(h.hwnd.get() as *mut std::ffi::c_void)
                                }
                                _ => None,
                            }
                        })
                }
                #[cfg(not(target_os = "windows"))]
                { None }
            };
            if let Err(e) = media_state.init(hwnd) {
                error!("Failed to initialize media controls: {}", e);
            } else {
                let handle_clone = handle.clone();
                if let Err(e) = media_state.attach_handler(move |event| {
                    use tauri::Emitter;
                    match event {
                        media_controls::MediaEvent::Play => {
                            let _ = handle_clone.emit("media-control-play", ());
                        }
                        media_controls::MediaEvent::Pause => {
                            let _ = handle_clone.emit("media-control-pause", ());
                        }
                        media_controls::MediaEvent::Toggle => {
                            let _ = handle_clone.emit("media-control-toggle", ());
                        }
                        media_controls::MediaEvent::Next => {
                            let _ = handle_clone.emit("media-control-next", ());
                        }
                        media_controls::MediaEvent::Previous => {
                            let _ = handle_clone.emit("media-control-previous", ());
                        }
                        media_controls::MediaEvent::Stop => {
                            let _ = handle_clone.emit("media-control-stop", ());
                        }
                        media_controls::MediaEvent::Seek(seconds) => {
                            let _ = handle_clone.emit("media-control-seek", seconds);
                        }
                    }
                }) {
                    error!("Failed to attach media control handlers: {}", e);
                } else {
                    info!("OS media controls initialized");
                }
            }

            // Spawn audio position update task
            {
                use std::time::Duration;
                use tauri::Emitter;

                let handle_for_audio = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_millis(250));
                    loop {
                        interval.tick().await;

                        let audio_state = handle_for_audio.state::<aurelia_core::audio::AudioState>();

                        // Check if player is initialized and playing
                        let player_guard = audio_state.player.lock().await;
                        if let Some(player) = player_guard.as_ref() {
                            if player.is_playing() {
                                let position = player.get_position();
                                let is_finished = player.is_finished();

                                // Emit position update to frontend
                                let _ = handle_for_audio.emit("audio:position", serde_json::json!({
                                    "position": position,
                                    "isFinished": is_finished
                                }));

                                if is_finished {
                                    // Track ended, could emit a separate event here if needed
                                    debug!("Audio playback finished");
                                }
                            }
                        }
                    }
                });
            }

            info!("Application setup finished.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
