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

#[cfg(test)]
mod tests {
    use super::ApiResponse;

    #[test]
    fn api_response_ok_sets_data() {
        let response = ApiResponse::ok(42);
        assert_eq!(response.status, "ok");
        assert_eq!(response.data, Some(42));
        assert!(response.error.is_none());
    }

    #[test]
    fn api_response_err_sets_error() {
        let response: ApiResponse<()> = ApiResponse::err("boom");
        assert_eq!(response.status, "error");
        assert!(response.data.is_none());
        assert_eq!(response.error.as_deref(), Some("boom"));
    }
}
