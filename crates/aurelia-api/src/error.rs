//! Error handling for the API layer

pub use aurelia_core::error::AppError;

/// Response wrapper for consistent API responses
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: impl ToString) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(error.to_string()),
        }
    }
}
