//! Data models for the Jellyfin Music Player
//!
//! This module contains all the data structures used throughout the application,
//! including API response types, internal data models, and configuration structures.

pub mod auth;
pub mod jellyfin;
pub mod lrclib;
pub mod music;

pub use auth::*;
pub use jellyfin::*;
pub use lrclib::*;
pub use music::*;
