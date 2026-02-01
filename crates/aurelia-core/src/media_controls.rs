//! OS Media Controls integration using souvlaki
//!
//! Provides system-level Now Playing integration:
//! - Windows: SMTC (System Media Transport Controls)
//! - macOS: MPNowPlayingInfoCenter
//! - Linux: MPRIS2 via D-Bus
//!
//! Only available with the `desktop` feature.

use souvlaki::{
    MediaButton, MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, error, info};

// Re-export NowPlayingPayload from models
pub use crate::models::NowPlayingPayload;

/// Media control event that can be handled by the application
#[derive(Debug, Clone)]
pub enum MediaEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,
    Seek(f64),
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

impl MediaControlsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize media controls
    ///
    /// # Arguments
    /// * `hwnd` - Optional window handle (required for Windows SMTC)
    pub fn init(&self, hwnd: Option<*mut std::ffi::c_void>) -> Result<(), String> {
        info!("Initializing OS media controls");

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

        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;
        *guard = Some(controls);

        info!("OS media controls initialized successfully");
        Ok(())
    }

    /// Attach event handler to media controls
    ///
    /// # Arguments
    /// * `handler` - Callback function that receives media events
    pub fn attach_handler<F>(&self, handler: F) -> Result<(), String>
    where
        F: Fn(MediaEvent) + Send + 'static,
    {
        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;

        if let Some(controls) = guard.as_mut() {
            controls
                .attach(move |event| {
                    debug!("Media control event: {:?}", event);

                    let media_event = match event {
                        MediaControlEvent::Play => MediaEvent::Play,
                        MediaControlEvent::Pause => MediaEvent::Pause,
                        MediaControlEvent::Toggle => MediaEvent::Toggle,
                        MediaControlEvent::Next => MediaEvent::Next,
                        MediaControlEvent::Previous => MediaEvent::Previous,
                        MediaControlEvent::Stop => MediaEvent::Stop,
                        MediaControlEvent::Seek(direction) => {
                            // Convert seek direction to seconds
                            match direction {
                                souvlaki::SeekDirection::Forward => MediaEvent::Seek(10.0),
                                souvlaki::SeekDirection::Backward => MediaEvent::Seek(-10.0),
                            }
                        }
                        MediaControlEvent::SetPosition(pos) => {
                            MediaEvent::Seek(pos.0.as_secs_f64())
                        }
                        _ => return, // Ignore other events
                    };

                    handler(media_event);
                })
                .map_err(|e| {
                    error!("Failed to attach media control handlers: {:?}", e);
                    format!("Failed to attach media control handlers: {:?}", e)
                })?;

            info!("Media control handlers attached");
        }

        Ok(())
    }

    /// Update the Now Playing metadata
    pub fn update_now_playing(&self, payload: NowPlayingPayload) -> Result<(), String> {
        debug!("Updating Now Playing: {:?}", payload);

        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;

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
                    duration: payload.duration.map(std::time::Duration::from_secs_f64),
                    cover_url: payload.cover_url.as_deref(),
                })
                .map_err(|e| format!("Failed to set metadata: {:?}", e))?;

            debug!("Now Playing updated successfully");
        }

        Ok(())
    }

    /// Update playback state (playing/paused)
    pub fn set_playback_status(
        &self,
        is_playing: bool,
        position_secs: Option<f64>,
    ) -> Result<(), String> {
        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;

        if let Some(controls) = guard.as_mut() {
            let progress = position_secs
                .map(|p| souvlaki::MediaPosition(std::time::Duration::from_secs_f64(p)));

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
    pub fn clear_now_playing(&self) -> Result<(), String> {
        debug!("Clearing Now Playing");

        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;

        if let Some(controls) = guard.as_mut() {
            controls
                .set_playback(MediaPlayback::Stopped)
                .map_err(|e| format!("Failed to clear playback: {:?}", e))?;
        }

        Ok(())
    }

    /// Enable or disable a media control button
    pub fn set_button_enabled(&self, button: &str, enabled: bool) -> Result<(), String> {
        debug!("Setting button {} enabled: {}", button, enabled);

        let media_button = match button {
            "play" => MediaButton::Play,
            "pause" => MediaButton::Pause,
            "stop" => MediaButton::Stop,
            "next" => MediaButton::Next,
            "previous" => MediaButton::Previous,
            "seek" => MediaButton::Seek,
            _ => return Err(format!("Unknown button: {}", button)),
        };

        let mut guard = self.controls.lock().map_err(|e| e.to_string())?;

        if let Some(controls) = guard.as_mut() {
            controls
                .set_button_enabled(media_button, enabled)
                .map_err(|e| format!("Failed to set button enabled: {:?}", e))?;
        }

        Ok(())
    }
}
