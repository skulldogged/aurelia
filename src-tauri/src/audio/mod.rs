//! Native audio playback backend using Rodio and Symphonia
//!
//! This module provides streaming audio playback with:
//! - HTTP streaming via stream-download
//! - 5-band parametric EQ
//! - Gapless playback
//! - Volume control

mod commands;
mod eq;
mod player;
mod streaming;

pub use commands::*;
pub use player::AudioPlayer;
