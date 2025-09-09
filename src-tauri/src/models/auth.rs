//! Authentication-related data models

use serde::{Deserialize, Serialize};
use specta::Type;

/// Response from successful Jellyfin login
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct LoginResponse {
    /// User authentication token
    pub token: String,
    /// User ID
    #[serde(rename = "userId")]
    pub user_id: String,
}

/// User credentials for Jellyfin authentication
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct Credentials {
    /// Jellyfin server URL
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
pub struct JellyfinUser {
    /// User ID
    #[serde(rename = "Id")]
    pub id: String,
}

/// Jellyfin authentication response
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct JellyfinAuthResponse {
    /// Access token for API calls
    #[serde(rename = "AccessToken")]
    pub access_token: String,
    /// User information
    #[serde(rename = "User")]
    pub user: JellyfinUser,
}
