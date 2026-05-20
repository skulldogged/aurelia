//! Authentication-related data models

use serde::{Deserialize, Serialize};
use specta::Type;

/// Supported backend providers.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, uniffi::Enum)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum BackendProvider {
    Jellyfin,
}

impl Default for BackendProvider {
    fn default() -> Self {
        Self::Jellyfin
    }
}

pub const fn default_backend_provider() -> BackendProvider {
    BackendProvider::Jellyfin
}

/// Provider capabilities for feature gating.
#[derive(Serialize, Deserialize, Debug, Clone, Type, uniffi::Record)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    pub supports_client_capabilities_registration: bool,
    pub supports_playback_progress_reporting: bool,
    pub supports_sidecar_lyrics_lookup: bool,
    pub supports_server_lyrics: bool,
    pub supports_instant_mix: bool,
}

/// Authentication request payload.
#[derive(Serialize, Deserialize, Debug, Clone, Type, uniffi::Record)]
#[specta(rename_all = "camelCase")]
pub struct AuthRequest {
    pub provider: BackendProvider,
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    pub username: String,
    pub password: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
}

/// Response from successful login
#[derive(Serialize, Deserialize, Debug, Type, uniffi::Record)]
#[specta(rename_all = "camelCase")]
pub struct LoginResponse {
    /// User authentication token
    pub token: String,
    /// User ID
    #[serde(rename = "userId")]
    pub user_id: String,
}

/// User credentials for backend authentication
#[derive(Serialize, Deserialize, Debug, Clone, Type, uniffi::Record)]
#[specta(rename_all = "camelCase")]
pub struct Credentials {
    #[serde(default = "default_backend_provider")]
    pub provider: BackendProvider,
    /// Backend server URL
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    /// Username
    pub username: String,
    /// Authentication token
    pub token: String,
    /// User ID
    #[serde(rename = "userId")]
    pub user_id: String,
}

/// Jellyfin user information
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct JellyfinUser {
    /// User ID
    #[serde(rename = "Id")]
    #[specta(rename = "id")]
    pub id: String,
}

/// Jellyfin authentication response
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct JellyfinAuthResponse {
    /// Access token for API calls
    #[serde(rename = "AccessToken")]
    #[specta(rename = "accessToken")]
    pub access_token: String,
    /// User information
    #[serde(rename = "User")]
    #[specta(rename = "user")]
    pub user: JellyfinUser,
}
