//! Audio playback event emission
//!
//! Provides event-based position updates instead of frontend polling

use crate::audio::AudioState;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{debug, trace};

/// Audio position update event payload
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioPositionEvent {
    pub position: f64,
    pub is_finished: bool,
}

/// Flag to control the event loop
static EVENT_LOOP_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start the audio event loop that emits position updates
/// This replaces frontend polling with push-based events
pub fn start_audio_event_loop(app: AppHandle) {
    // Prevent multiple loops
    if EVENT_LOOP_RUNNING.swap(true, Ordering::SeqCst) {
        debug!("Audio event loop already running");
        return;
    }

    debug!("Starting audio event loop");

    tauri::async_runtime::spawn(async move {
        let interval = Duration::from_millis(250);

        loop {
            tokio::time::sleep(interval).await;

            // Get audio state
            let audio_state = match app.try_state::<AudioState>() {
                Some(state) => state,
                None => {
                    trace!("AudioState not available, waiting...");
                    continue;
                }
            };

            // Check player state (lock briefly)
            let (position, is_finished, is_playing) = {
                let player_guard = match audio_state.player.lock() {
                    Ok(guard) => guard,
                    Err(_) => continue,
                };

                match player_guard.as_ref() {
                    Some(player) => (
                        player.get_position(),
                        player.is_finished(),
                        player.is_playing(),
                    ),
                    None => continue,
                }
            };

            // Only emit events when playing
            if !is_playing && !is_finished {
                continue;
            }

            // Emit position event
            let event = AudioPositionEvent {
                position,
                is_finished,
            };

            if let Err(e) = app.emit("audio:position", &event) {
                trace!("Failed to emit audio position event: {}", e);
            }
        }
    });
}
