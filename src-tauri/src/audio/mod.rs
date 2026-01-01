//! Native audio playback backend using Rodio and Symphonia
//!
//! This module provides streaming audio playback with:
//! - HTTP streaming via stream-download
//! - 5-band parametric EQ
//! - Gapless playback
//! - Volume control
//! - Event-based position updates

mod commands;
mod eq;
mod events;
mod player;
mod streaming;

pub use commands::*;
pub use events::*;
pub use player::AudioPlayer;
