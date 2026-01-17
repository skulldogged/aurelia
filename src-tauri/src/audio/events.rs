//! Audio playback event emission
//!
//! Provides event-based position updates and spectrum data for visualization

use crate::audio::AudioState;
use crate::audio::analyzer::{AnalyzerBuffer, SpectrumAnalyzer};
use serde::Serialize;
use std::sync::Arc;
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

/// Spectrum data event payload for visualizer
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectrumEvent {
    /// Frequency domain data (0-255 per bin)
    pub frequency_data: Vec<u8>,
    /// Time domain data (0-255, centered at 128)
    pub time_domain_data: Vec<u8>,
}

/// Flag to control the position event loop
static POSITION_LOOP_RUNNING: AtomicBool = AtomicBool::new(false);

/// Flag to control the spectrum event loop
static SPECTRUM_LOOP_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start the audio event loop that emits position updates
/// This replaces frontend polling with push-based events
pub fn start_audio_event_loop(app: AppHandle) {
    // Prevent multiple loops
    if POSITION_LOOP_RUNNING.swap(true, Ordering::SeqCst) {
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

/// Start the spectrum analyzer event loop
///
/// Runs at ~60fps (16ms interval) for smooth visualization
pub fn start_spectrum_event_loop(app: AppHandle, analyzer_buffer: Arc<AnalyzerBuffer>) {
    // Prevent multiple loops
    if SPECTRUM_LOOP_RUNNING.swap(true, Ordering::SeqCst) {
        debug!("Spectrum event loop already running");
        return;
    }

    debug!("Starting spectrum event loop");

    tauri::async_runtime::spawn(async move {
        let interval = Duration::from_millis(16); // ~60fps
        let mut spectrum_analyzer = SpectrumAnalyzer::new();

        loop {
            tokio::time::sleep(interval).await;

            // Skip if analyzer is disabled
            if !analyzer_buffer.is_enabled() {
                continue;
            }

            // Read samples and compute spectrum
            let samples = analyzer_buffer.read_samples();
            let frequency_data = spectrum_analyzer.compute_spectrum(&samples);
            let time_domain_data = spectrum_analyzer.compute_waveform(&samples);

            // Emit spectrum event
            let event = SpectrumEvent {
                frequency_data: frequency_data.to_vec(),
                time_domain_data: time_domain_data.to_vec(),
            };

            if let Err(e) = app.emit("audio:spectrum", &event) {
                trace!("Failed to emit spectrum event: {}", e);
            }
        }
    });
}
