//! Error handling for the Jellyfin Music Player
//!
//! This module provides a unified error type and proper error handling
//! throughout the application.

use std::fmt;

/// Application-specific error type
#[derive(Debug)]
pub enum AppError {
    /// Network-related errors
    Network(String),
    /// Authentication errors
    Auth(String),
    /// Database errors
    Database(String),
    /// Serialization/deserialization errors
    Serialization(String),
    /// File system errors
    FileSystem(String),
    /// API response parsing errors
    ApiParse(String),
    /// Configuration errors
    Config(String),
    /// Generic application errors
    General(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AppError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            AppError::ApiParse(msg) => write!(f, "API parsing error: {}", msg),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::General(msg) => write!(f, "Application error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileSystem(err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
