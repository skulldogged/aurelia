//! Aurelia API - Unified API layer for all platforms
//!
//! This crate provides:
//! - The `Api` trait definition (single source of truth)
//! - Runtime implementations for Tauri (desktop) and Axum (web)
//! - Generated code for both platforms

// Re-export the macro
pub use aurelia_api_macros::aurelia_api;

// Re-export core types
pub use aurelia_core::{
    error::AppError,
    models::*,
    listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen},
    domain,
};

// The Api trait with macro annotations
pub mod traits;
pub use traits::{Api, ApiResult};

// Platform-specific implementations
#[cfg(feature = "desktop")]
pub mod tauri_impl;

#[cfg(feature = "web")]
pub mod axum_impl;

// Error handling
pub mod error;
