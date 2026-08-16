//! Native audio playback backend using Rodio and Symphonia
//!
//! This module provides streaming audio playback with:
//! - HTTP streaming via stream-download
//! - 5-band parametric EQ
//! - Gapless playback
//! - Volume control
//! - FFT-based spectrum analysis for visualization
//!
//! Only available with the `desktop` feature.

mod analyzer;
mod eq;
mod player;
mod streaming;

pub use analyzer::{
    AnalyzerBuffer, AnalyzerSource, FFT_SIZE, FREQUENCY_BIN_COUNT, SpectrumAnalyzer,
};
pub use eq::{DEFAULT_EQ_BANDS, EQBand, EQFilterType, EQSettings, EQSource};
pub use player::AudioPlayer;
pub use streaming::StreamingSource;

use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

/// Global audio player state
pub struct AudioState {
    pub player: TokioMutex<Option<AudioPlayer>>,
    /// Shared analyzer buffer for spectrum events
    pub analyzer_buffer: Mutex<Option<Arc<AnalyzerBuffer>>>,
    /// Persistent FFT processor so spectrum smoothing carries across frames.
    spectrum_analyzer: Mutex<SpectrumAnalyzer>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            player: TokioMutex::new(None),
            analyzer_buffer: Mutex::new(None),
            spectrum_analyzer: Mutex::new(SpectrumAnalyzer::new()),
        }
    }
}

impl AudioState {
    pub fn new() -> Self {
        Self::default()
    }
}

use anyhow::Result;

#[derive(Clone, Debug, uniffi::Record)]
pub struct SpectrumSnapshot {
    pub frequency_data: Vec<u8>,
    pub time_domain_data: Vec<u8>,
}

/// Initialize the audio player
pub async fn audio_init(state: &AudioState) -> Result<()> {
    let mut player_guard = state.player.lock().await;

    if player_guard.is_none() {
        let player = AudioPlayer::new()?;
        let analyzer_buffer = player.analyzer_buffer();
        *player_guard = Some(player);
        drop(player_guard);

        let mut analyzer_guard = state.analyzer_buffer.lock().unwrap();
        *analyzer_guard = Some(analyzer_buffer);
    }

    Ok(())
}

/// Play audio from URL
pub async fn audio_play(
    state: &AudioState,
    url: String,
    start_time_secs: Option<f64>,
    token: String,
) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;

    if let Some(start) = start_time_secs {
        player
            .play_url_from_position(&url, &token, Some(start))
            .await
    } else {
        player.play_url(&url, &token).await
    }
}

/// Pause playback
pub async fn audio_pause(state: &AudioState) -> Result<()> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.pause();
    Ok(())
}

/// Resume playback
pub async fn audio_resume(state: &AudioState) -> Result<()> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.resume();
    Ok(())
}

/// Stop playback
pub async fn audio_stop(state: &AudioState) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.stop();
    Ok(())
}

/// Set volume (0.0 to 1.0)
pub async fn audio_set_volume(state: &AudioState, volume: f32) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.set_volume(volume.clamp(0.0, 1.0));
    Ok(())
}

/// Get current volume
pub async fn audio_get_volume(state: &AudioState) -> Result<f32> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.get_volume())
}

/// Check if audio is currently playing
pub async fn audio_is_playing(state: &AudioState) -> Result<bool> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.is_playing())
}

/// Check if playback has finished
pub async fn audio_is_finished(state: &AudioState) -> Result<bool> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.is_finished())
}

/// Get current playback position in seconds
pub async fn audio_get_position(state: &AudioState) -> Result<f64> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.get_position())
}

/// Seek to position
pub async fn audio_seek(state: &AudioState, position_secs: f64) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.seek_with_fallback(position_secs).await
}

/// Read one EQ band gain
pub async fn audio_get_eq_band(state: &AudioState, band: u32) -> Result<f32> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.get_eq_band(band as usize))
}

/// Read all EQ band gains
pub async fn audio_get_all_eq_bands(state: &AudioState) -> Result<Vec<f32>> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.get_all_eq_bands().to_vec())
}

#[derive(Clone, Debug)]
pub struct AudioPositionTick {
    pub did_auto_advance: bool,
    pub is_finished: bool,
    pub is_playing: bool,
    pub position: f64,
}

/// Tick the player: maybe auto-advance, then report position if relevant.
pub async fn audio_poll_position(state: &AudioState) -> Option<AudioPositionTick> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard.as_mut()?;
    let did_auto_advance = player.auto_advance_to_next();
    let is_playing = player.is_playing();
    let is_finished = player.is_finished();
    if !is_playing && !is_finished && !did_auto_advance {
        return None;
    }
    Some(AudioPositionTick {
        did_auto_advance,
        is_finished,
        is_playing,
        position: player.get_position(),
    })
}

/// Prepare next track for gapless playback
pub async fn audio_prepare_next(state: &AudioState, url: String, token: String) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.prepare_next(&url, &token).await
}

/// Advance to next prepared track
pub async fn audio_advance_gapless(state: &AudioState) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    let player = player_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.advance_to_next().await
}

/// Enable/disable EQ
pub async fn audio_set_eq_enabled(state: &AudioState, enabled: bool) -> Result<()> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.set_eq_enabled(enabled)
}

/// Check if EQ is enabled
pub async fn audio_is_eq_enabled(state: &AudioState) -> Result<bool> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.is_eq_enabled())
}

/// Set EQ band parameters
pub async fn audio_set_eq_band(
    state: &AudioState,
    band: u8,
    _frequency: f32,
    gain: f32,
    _q: f32,
) -> Result<()> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.set_eq_band(band as usize, gain)
}

/// Reset EQ to default
pub async fn audio_reset_eq(state: &AudioState) -> Result<()> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.reset_eq()
}

/// Enable/disable analyzer
pub async fn audio_set_analyzer_enabled(state: &AudioState, enabled: bool) -> Result<()> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    player.set_analyzer_enabled(enabled);
    Ok(())
}

/// Check if analyzer is enabled
pub async fn audio_is_analyzer_enabled(state: &AudioState) -> Result<bool> {
    let player_guard = state.player.lock().await;
    let player = player_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;
    Ok(player.is_analyzer_enabled())
}

/// Read the current analyzer buffer and compute FFT/waveform data.
pub fn audio_get_spectrum_snapshot(state: &AudioState) -> Result<SpectrumSnapshot> {
    let analyzer_buffer = state
        .analyzer_buffer
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Audio player not initialized"))?;

    let samples = analyzer_buffer.read_samples();
    let mut analyzer = state.spectrum_analyzer.lock().unwrap();
    let frequency_data = analyzer.compute_spectrum(&samples).to_vec();
    let time_domain_data = analyzer.compute_waveform(&samples).to_vec();

    Ok(SpectrumSnapshot {
        frequency_data,
        time_domain_data,
    })
}

/// Reinitialize audio player
pub async fn audio_reinit(state: &AudioState) -> Result<()> {
    let mut player_guard = state.player.lock().await;
    if let Some(player) = player_guard.as_mut() {
        player.reinit()?;
    }
    Ok(())
}
