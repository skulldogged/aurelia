//! Error handling for the Jellyfin Music Player
//!
//! Uses thiserror for structured error types and anyhow for ergonomic error handling

/// Application-specific error type using thiserror
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("API parsing error: {0}")]
    ApiParse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    #[error("Application error: {0}")]
    General(String),
}

// Convert external errors to AppError
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

/// Result type alias for convenience
pub type AppResult<T> = std::result::Result<T, AppError>;

/// Convenience type for anyhow::Result (for internal use where we want ergonomic error handling)
pub type Result<T> = anyhow::Result<T>;
