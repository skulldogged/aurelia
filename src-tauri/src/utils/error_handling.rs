//! Error handling utilities using anyhow for ergonomic error propagation
//!
//! This module provides utilities to reduce boilerplate when working with errors,
//! especially for the common patterns found throughout the codebase.

use crate::error::{AppError, AppResult};
use reqwest::Response;

/// Convert a reqwest::Response into an AppResult, checking for HTTP errors
pub async fn handle_http_response(response: Response) -> AppResult<Response> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(AppError::Http { status, message })
    }
}

/// Get JSON from a response, converting errors to AppError
pub async fn get_json_from_response<T: serde::de::DeserializeOwned>(
    response: Response,
) -> AppResult<T> {
    let text = response.text().await?;
    serde_json::from_str(&text).map_err(|e| {
        AppError::ApiParse(format!("Failed to parse JSON: {} (response: {})", e, text))
    })
}

/// Helper for cache operations that might fail
pub fn handle_cache_error<T>(result: Result<T, impl std::fmt::Display>) -> AppResult<T> {
    result.map_err(|e| AppError::Database(format!("Cache operation failed: {}", e)))
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

/// Convert an anyhow::Error to AppError (for cases where we use anyhow internally)
pub fn anyhow_to_app_error(err: anyhow::Error) -> AppError {
    AppError::General(err.to_string())
}

/// Convert AppError to anyhow::Error (for bridging between error types)
pub fn app_error_to_anyhow(err: AppError) -> anyhow::Error {
    anyhow::Error::from(err)
}
