//! Tauri command handlers
//!
//! This module contains all the Tauri command handlers that are exposed
//! to the frontend. These handlers coordinate between the services and
//! provide a clean API boundary.

pub mod auth;
pub mod lyrics;
pub mod music;

pub use auth::*;
pub use lyrics::*;
pub use music::*;
