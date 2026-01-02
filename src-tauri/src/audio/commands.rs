//! Tauri command handlers for audio playback
//!
//! Exposes audio player functionality to the frontend.

use crate::audio::AudioPlayer;
use crate::audio::events::start_audio_event_loop;
use std::sync::Mutex;
use tauri::State;
use tracing::{error, info};

/// Global audio player state
pub struct AudioState {
    pub player: Mutex<Option<AudioPlayer>>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            player: Mutex::new(None),
        }
    }
}

/// Initialize the audio player
#[tauri::command]
#[specta::specta]
pub async fn audio_init(
    app: tauri::AppHandle,
    audio_state: State<'_, AudioState>,
) -> Result<(), String> {
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;

    if player_guard.is_none() {
        let player = AudioPlayer::new().map_err(|e| {
            error!("Failed to initialize audio player: {}", e);
            e.to_string()
        })?;
        *player_guard = Some(player);
        info!("Audio player initialized");

        // Start the audio event loop for position updates
        drop(player_guard); // Release lock before starting event loop
        start_audio_event_loop(app);
    }

    Ok(())
}

/// Play audio from a URL
#[tauri::command]
#[specta::specta]
pub async fn audio_play(
    audio_state: State<'_, AudioState>,
    url: String,
    token: String,
) -> Result<(), String> {
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;

    let _player = player_guard
        .as_mut()
        .ok_or("Audio player not initialized")?;

    // We need to release the lock before await, so we'll use a different approach
    drop(player_guard);

    // Re-acquire and play
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_mut()
        .ok_or("Audio player not initialized")?;

    // Use tokio spawn_blocking for the async operation
    let url_clone = url.clone();
    let token_clone = token.clone();

    // Since play_url is async, we need to handle this differently
    // For now, we'll create a new runtime context
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { player.play_url(&url_clone, &token_clone).await })
    })
    .map_err(|e| {
        error!("Failed to play audio: {}", e);
        e.to_string()
    })
}

/// Pause audio playback
#[tauri::command]
#[specta::specta]
pub fn audio_pause(audio_state: State<'_, AudioState>) -> Result<(), String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    player.pause();
    Ok(())
}

/// Resume audio playback
#[tauri::command]
#[specta::specta]
pub fn audio_resume(audio_state: State<'_, AudioState>) -> Result<(), String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    player.resume();
    Ok(())
}

/// Stop audio playback
#[tauri::command]
#[specta::specta]
pub fn audio_stop(audio_state: State<'_, AudioState>) -> Result<(), String> {
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_mut()
        .ok_or("Audio player not initialized")?;
    player.stop();
    Ok(())
}

/// Set audio volume (0.0 to 1.0)
#[tauri::command]
#[specta::specta]
pub fn audio_set_volume(audio_state: State<'_, AudioState>, volume: f32) -> Result<(), String> {
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_mut()
        .ok_or("Audio player not initialized")?;
    player.set_volume(volume);
    Ok(())
}

/// Get current volume
#[tauri::command]
#[specta::specta]
pub fn audio_get_volume(audio_state: State<'_, AudioState>) -> Result<f32, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.get_volume())
}

/// Check if audio is currently playing
#[tauri::command]
#[specta::specta]
pub fn audio_is_playing(audio_state: State<'_, AudioState>) -> Result<bool, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.is_playing())
}

/// Check if playback is finished
#[tauri::command]
#[specta::specta]
pub fn audio_is_finished(audio_state: State<'_, AudioState>) -> Result<bool, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.is_finished())
}

/// Get current playback position in seconds
#[tauri::command]
#[specta::specta]
pub fn audio_get_position(audio_state: State<'_, AudioState>) -> Result<f64, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.get_position())
}

/// Seek to a position in seconds (with fallback to stream restart for backward seeks)
#[tauri::command]
#[specta::specta]
pub async fn audio_seek(
    audio_state: State<'_, AudioState>,
    position_secs: f64,
) -> Result<(), String> {
    // Try native seek first (fast path) - hold lock briefly
    {
        let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
        let player = player_guard
            .as_ref()
            .ok_or("Audio player not initialized")?;

        // Native seek is synchronous and fast
        if player.seek(position_secs).is_ok() {
            return Ok(());
        }
        // Native seek failed, we'll try fallback below
    }

    // Fallback: restart stream from target position (slow, async)
    // Lock is released here so event loop and other commands can work
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
            let player = player_guard
                .as_mut()
                .ok_or_else(|| "Audio player not initialized".to_string())?;
            player.seek_with_fallback(position_secs).await.map_err(|e| {
                error!("Failed to seek: {}", e);
                e.to_string()
            })
        })
    })
}

/// Prepare next track for gapless playback
#[tauri::command]
#[specta::specta]
pub fn audio_prepare_next(
    audio_state: State<'_, AudioState>,
    url: String,
    token: String,
) -> Result<(), String> {
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_mut()
        .ok_or("Audio player not initialized")?;
    player.prepare_next(&url, &token);
    Ok(())
}

/// Advance to the prepared next track (gapless)
#[tauri::command]
#[specta::specta]
pub async fn audio_advance_gapless(audio_state: State<'_, AudioState>) -> Result<(), String> {
    let mut player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_mut()
        .ok_or("Audio player not initialized")?;

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async { player.advance_to_next().await })
    })
    .map_err(|e| {
        error!("Failed to advance to next track: {}", e);
        e.to_string()
    })
}

/// Enable or disable EQ
#[tauri::command]
#[specta::specta]
pub fn audio_set_eq_enabled(
    audio_state: State<'_, AudioState>,
    enabled: bool,
) -> Result<(), String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    player.set_eq_enabled(enabled).map_err(|e| e.to_string())
}

/// Check if EQ is enabled
#[tauri::command]
#[specta::specta]
pub fn audio_is_eq_enabled(audio_state: State<'_, AudioState>) -> Result<bool, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.is_eq_enabled())
}

/// Set EQ band gain (band: 0-4, gain: -20 to +20 dB)
#[tauri::command]
#[specta::specta]
pub fn audio_set_eq_band(
    audio_state: State<'_, AudioState>,
    band: u32,
    gain_db: f32,
) -> Result<(), String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    player
        .set_eq_band(band as usize, gain_db)
        .map_err(|e| e.to_string())
}

/// Get EQ band gain
#[tauri::command]
#[specta::specta]
pub fn audio_get_eq_band(audio_state: State<'_, AudioState>, band: u32) -> Result<f32, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.get_eq_band(band as usize))
}

/// Get all EQ band gains
#[tauri::command]
#[specta::specta]
pub fn audio_get_all_eq_bands(audio_state: State<'_, AudioState>) -> Result<Vec<f32>, String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    Ok(player.get_all_eq_bands().to_vec())
}

/// Reset EQ to flat
#[tauri::command]
#[specta::specta]
pub fn audio_reset_eq(audio_state: State<'_, AudioState>) -> Result<(), String> {
    let player_guard = audio_state.player.lock().map_err(|e| e.to_string())?;
    let player = player_guard
        .as_ref()
        .ok_or("Audio player not initialized")?;
    player.reset_eq().map_err(|e| e.to_string())
}
