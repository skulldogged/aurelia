//! Error handling utilities using anyhow for ergonomic error propagation
//!
//! This module provides utilities to reduce boilerplate when working with errors,
//! especially for the common patterns found throughout the codebase.

use crate::error::{AppError, AppResult};
use reqwest::Response;

/// Convert a `reqwest::Response` into an `AppResult`, checking for HTTP errors
pub async fn handle_http_response(response: Response) -> AppResult<Response> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(AppError::Http {
            status,
            detail: message,
        })
    }
}

/// Get JSON from a response, converting errors to `AppError`
pub async fn get_json_from_response<T: serde::de::DeserializeOwned>(
    response: Response,
) -> AppResult<T> {
    let text = response.text().await?;
    serde_json::from_str(&text)
        .map_err(|e| AppError::ApiParse(format!("Failed to parse JSON: {e} (response: {text})")))
}

/// Helper for cache operations that might fail
pub fn handle_cache_error<T>(result: Result<T, impl std::fmt::Display>) -> AppResult<T> {
    result.map_err(|e| AppError::Database(format!("Cache operation failed: {e}")))
}

/// Create a network error from any error type
pub fn network_error<E: std::fmt::Display>(err: E) -> AppError {
    AppError::Network(err.to_string())
}

/// Create an auth error from any error type
pub fn auth_error<E: std::fmt::Display>(err: E) -> AppError {
    AppError::Auth(err.to_string())
}

/// Create a general error from any error type
pub fn general_error<E: std::fmt::Display>(err: E) -> AppError {
    AppError::General(err.to_string())
}

/// Convert an `anyhow::Error` to `AppError` (for cases where we use anyhow internally)
#[must_use]
pub fn anyhow_to_app_error(err: &anyhow::Error) -> AppError {
    AppError::General(err.to_string())
}

/// Convert `AppError` to `anyhow::Error` (for bridging between error types)
#[must_use]
pub fn app_error_to_anyhow(err: AppError) -> anyhow::Error {
    anyhow::Error::from(err)
}

/// Create a network error with additional context
pub fn network_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::Network(format!("{context}: {err}"))
}

/// Create an auth error with additional context
pub fn auth_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::Auth(format!("{context}: {err}"))
}

/// Create a database error with additional context
pub fn database_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::Database(format!("{context}: {err}"))
}

/// Create a file system error with additional context
pub fn filesystem_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::FileSystem(format!("{context}: {err}"))
}

/// Create an API parsing error with additional context
pub fn api_parse_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::ApiParse(format!("{context}: {err}"))
}

/// Create a configuration error with additional context
pub fn config_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::Config(format!("{context}: {err}"))
}

/// Create a serialization error with additional context
pub fn serialization_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::Serialization(format!("{context}: {err}"))
}

/// Create a general error with additional context
pub fn general_error_with_context<E: std::fmt::Display>(err: E, context: &str) -> AppError {
    AppError::General(format!("{context}: {err}"))
}

/// Log an error with context and return it
pub fn log_and_return_error(err: AppError, operation: &str) -> AppError {
    tracing::error!("{} failed: {}", operation, err);
    err
}

/// Convert an `AppError` to a user-friendly string message
#[must_use]
pub fn error_to_user_message(err: &AppError) -> String {
    match err {
        AppError::Network(msg) => format!("Network error: {msg}"),
        AppError::Auth(msg) => format!("Authentication failed: {msg}"),
        AppError::Database(msg) => format!("Database error: {msg}"),
        AppError::Serialization(msg) => format!("Data processing error: {msg}"),
        AppError::FileSystem(msg) => format!("File system error: {msg}"),
        AppError::ApiParse(msg) => format!("Server response error: {msg}"),
        AppError::Config(msg) => format!("Configuration error: {msg}"),
        AppError::Http { status, detail } => format!("Server error ({status}): {detail}"),
        AppError::General(msg) => msg.clone(),
        AppError::UniFfi(msg) => msg.clone(),
    }
}
