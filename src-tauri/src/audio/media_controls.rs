//! OS Media Controls integration using souvlaki
//!
//! Provides system-level Now Playing integration:
//! - Windows: SMTC (System Media Transport Controls)
//! - macOS: MPNowPlayingInfoCenter
//! - Linux: MPRIS2 via D-Bus

use crate::audio::AudioState;
use serde::{Deserialize, Serialize};
use souvlaki::{MediaButton, MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{debug, error, info, warn};

/// Payload for updating Now Playing metadata
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingPayload {
    pub title: String,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub duration_secs: Option<f64>,
    #[serde(default)]
    pub cover_url: Option<String>,
}

/// State container for media controls
pub struct MediaControlsState {
    pub controls: Mutex<Option<MediaControls>>,
    /// Path to cached cover art file
    pub cached_cover_path: Mutex<Option<PathBuf>>,
}

impl Default for MediaControlsState {
    fn default() -> Self {
        Self {
            controls: Mutex::new(None),
            cached_cover_path: Mutex::new(None),
        }
    }
}

/// Initialize media controls with the app's window handle
pub fn init_media_controls(_app: &AppHandle) -> Result<MediaControlsState, String> {
    info!("Initializing OS media controls");

    // Get the HWND on Windows, which is required for SMTC
    #[cfg(target_os = "windows")]
    let hwnd: Option<*mut std::ffi::c_void> = {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        // Try to get the main window
        let window = _app.get_webview_window("main");
        match window {
            Some(win) => match win.window_handle() {
                Ok(handle) => match handle.as_raw() {
                    RawWindowHandle::Win32(h) => {
                        // Convert HWND (NonNull<c_void>) to *mut c_void
                        Some(h.hwnd.get() as *mut std::ffi::c_void)
                    }
                    _ => {
                        warn!("Unexpected window handle type, media controls may not work");
                        None
                    }
                },
                Err(e) => {
                    warn!(
                        "Failed to get window handle: {}, media controls may not work",
                        e
                    );
                    None
                }
            },
            None => {
                warn!("No main window found, media controls may not work on Windows");
                None
            }
        }
    };

    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    let config = PlatformConfig {
        dbus_name: "aurelia",
        display_name: "Aurelia",
        hwnd,
        app_id: Some("dev.pupbrained.aurelia"),
    };

    let controls = MediaControls::new(config).map_err(|e| {
        error!("Failed to create media controls: {:?}", e);
        format!("Failed to create media controls: {:?}", e)
    })?;

    info!("OS media controls initialized successfully");

    Ok(MediaControlsState {
        controls: Mutex::new(Some(controls)),
        cached_cover_path: Mutex::new(None),
    })
}

/// Attach event handlers to media controls
pub fn attach_media_handlers(state: &MediaControlsState, app: AppHandle) -> Result<(), String> {
    let mut guard = state.controls.lock().map_err(|e| e.to_string())?;

    if let Some(controls) = guard.as_mut() {
        let app_handle = app.clone();

        controls
            .attach(move |event| {
                debug!("Media control event: {:?}", event);

                // Get the audio state to control playback
                let audio_state = match app_handle.try_state::<AudioState>() {
                    Some(s) => s,
                    None => {
                        warn!("AudioState not available for media control event");
                        return;
                    }
                };

                match event {
                    MediaControlEvent::Play => {
                        if let Ok(guard) = audio_state.player.lock() {
                            if let Some(player) = guard.as_ref() {
                                player.resume();
                                let _ = app_handle.emit("media:play", ());
                            }
                        }
                    }
                    MediaControlEvent::Pause => {
                        if let Ok(guard) = audio_state.player.lock() {
                            if let Some(player) = guard.as_ref() {
                                player.pause();
                                let _ = app_handle.emit("media:pause", ());
                            }
                        }
                    }
                    MediaControlEvent::Toggle => {
                        if let Ok(guard) = audio_state.player.lock() {
                            if let Some(player) = guard.as_ref() {
                                if player.is_playing() {
                                    player.pause();
                                    let _ = app_handle.emit("media:pause", ());
                                } else {
                                    player.resume();
                                    let _ = app_handle.emit("media:play", ());
                                }
                            }
                        }
                    }
                    MediaControlEvent::Next => {
                        let _ = app_handle.emit("media:next", ());
                    }
                    MediaControlEvent::Previous => {
                        let _ = app_handle.emit("media:previous", ());
                    }
                    MediaControlEvent::Stop => {
                        if let Ok(mut guard) = audio_state.player.lock() {
                            if let Some(player) = guard.as_mut() {
                                player.stop();
                                let _ = app_handle.emit("media:stop", ());
                            }
                        }
                    }
                    _ => {}
                }
            })
            .map_err(|e| {
                error!("Failed to attach media control handlers: {:?}", e);
                format!("Failed to attach media control handlers: {:?}", e)
            })?;

        info!("Media control handlers attached");
    }

    Ok(())
}

/// Update the Now Playing metadata displayed by the OS
#[tauri::command]
#[specta::specta]
pub fn media_update_now_playing(
    state: State<'_, MediaControlsState>,
    payload: NowPlayingPayload,
) -> Result<(), String> {
    debug!("Updating Now Playing: {:?}", payload);

    let mut guard = state.controls.lock().map_err(|e| e.to_string())?;

    if let Some(controls) = guard.as_mut() {
        // Set playback status to playing
        controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .map_err(|e| format!("Failed to set playback status: {:?}", e))?;

        // Update metadata
        controls
            .set_metadata(MediaMetadata {
                title: Some(&payload.title),
                artist: payload.artist.as_deref(),
                album: payload.album.as_deref(),
                duration: payload
                    .duration_secs
                    .map(std::time::Duration::from_secs_f64),
                cover_url: payload.cover_url.as_deref(),
            })
            .map_err(|e| format!("Failed to set metadata: {:?}", e))?;

        debug!("Now Playing updated successfully");
    }

    Ok(())
}

/// Update playback state (playing/paused)
#[tauri::command]
#[specta::specta]
pub fn media_set_playback_status(
    state: State<'_, MediaControlsState>,
    is_playing: bool,
    position_secs: Option<f64>,
) -> Result<(), String> {
    let mut guard = state.controls.lock().map_err(|e| e.to_string())?;

    if let Some(controls) = guard.as_mut() {
        let progress =
            position_secs.map(|p| souvlaki::MediaPosition(std::time::Duration::from_secs_f64(p)));

        let playback = if is_playing {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        };

        controls
            .set_playback(playback)
            .map_err(|e| format!("Failed to set playback status: {:?}", e))?;
    }

    Ok(())
}

/// Clear the Now Playing display
#[tauri::command]
#[specta::specta]
pub fn media_clear_now_playing(state: State<'_, MediaControlsState>) -> Result<(), String> {
    debug!("Clearing Now Playing");

    let mut guard = state.controls.lock().map_err(|e| e.to_string())?;

    if let Some(controls) = guard.as_mut() {
        controls
            .set_playback(MediaPlayback::Stopped)
            .map_err(|e| format!("Failed to clear playback: {:?}", e))?;
    }

    Ok(())
}

/// Enable or disable a media control button
#[tauri::command]
#[specta::specta]
pub fn media_set_button_enabled(
    state: State<'_, MediaControlsState>,
    button: String,
    enabled: bool,
) -> Result<(), String> {
    debug!("Setting button {} enabled: {}", button, enabled);

    let media_button = match button.as_str() {
        "play" => MediaButton::Play,
        "pause" => MediaButton::Pause,
        "stop" => MediaButton::Stop,
        "next" => MediaButton::Next,
        "previous" => MediaButton::Previous,
        "seek" => MediaButton::Seek,
        _ => return Err(format!("Unknown button: {}", button)),
    };

    let mut guard = state.controls.lock().map_err(|e| e.to_string())?;

    if let Some(controls) = guard.as_mut() {
        controls
            .set_button_enabled(media_button, enabled)
            .map_err(|e| format!("Failed to set button enabled: {:?}", e))?;
    }

    Ok(())
}
