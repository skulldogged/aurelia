//! Native audio playback backend using Rodio and Symphonia
//!
//! This module provides streaming audio playback with:
//! - HTTP streaming via stream-download
//! - 5-band parametric EQ
//! - Gapless playback
//! - Volume control
//! - Event-based position updates
//! - FFT-based spectrum analysis for visualization

mod analyzer;
mod commands;
mod eq;
mod events;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod media_controls;
mod player;
mod streaming;

pub use commands::*;
pub use events::*;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use media_controls::{
    MediaControlsState, media_clear_now_playing, media_set_button_enabled,
    media_set_playback_status, media_update_now_playing,
};
pub use player::AudioPlayer;
