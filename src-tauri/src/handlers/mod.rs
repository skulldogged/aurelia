//! Tauri command handlers
//!
//! This module contains all the Tauri command handlers that are exposed
//! to the frontend. These handlers coordinate between the services and
//! provide a clean API boundary.

pub mod appearance;
pub mod auth;
pub mod images;
pub mod lyrics;
pub mod music;
pub mod playlists;

pub use appearance::*;
pub use auth::*;
pub use images::*;
pub use lyrics::*;
pub use music::*;
