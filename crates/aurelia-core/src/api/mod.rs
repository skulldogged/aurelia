//! API abstraction layer
//!
//! This module provides trait definitions and implementations for the Aurelia API.
//! It's the single source of truth for both Tauri (desktop) and HTTP (web) implementations.

pub mod traits;

pub use traits::{Api, ApiResult};
