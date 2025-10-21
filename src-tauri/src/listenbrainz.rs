use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{Arc, Mutex};
use tauri::State;
use tracing::{debug, error, info};

const LISTENBRAINZ_API_URL: &str = "https://api.listenbrainz.org/1";

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListenBrainzCredentials {
    pub user_token: String,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListenBrainzListen {
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListenPayload {
    listen_type: String,
    payload: Vec<PayloadItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    listened_at: Option<f64>,
    track_metadata: TrackMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackMetadata {
    artist_name: String,
    track_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_info: Option<AdditionalInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdditionalInfo {
    duration_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidateTokenResponse {
    code: i32,
    message: String,
    user_name: Option<String>,
    valid: bool,
}

#[derive(Default)]
pub struct ListenBrainzState {
    credentials: Arc<Mutex<Option<ListenBrainzCredentials>>>,
    client: Arc<Mutex<Client>>,
}

impl ListenBrainzState {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(Client::new())),
        }
    }
}

/// Validate a ListenBrainz user token and get the username
#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_validate_token(
    user_token: String,
    state: State<'_, ListenBrainzState>,
) -> Result<ListenBrainzCredentials, String> {
    info!("Validating ListenBrainz token");

    let client = state.client.lock().unwrap().clone();
    let url = format!("{}/validate-token", LISTENBRAINZ_API_URL);

    let response = client
        .get(&url)
        .header("Authorization", format!("Token {}", user_token))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to validate token: {}", e);
            format!("Network error: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        error!("Token validation failed with status: {}", status);
        return Err(format!("Invalid token (status: {})", status));
    }

    let validate_response: ValidateTokenResponse = response.json().await.map_err(|e| {
        error!("Failed to parse validation response: {}", e);
        format!("Failed to parse response: {}", e)
    })?;

    if !validate_response.valid {
        error!("Token is invalid");
        return Err("Token is not valid".to_string());
    }

    let username = validate_response.user_name.ok_or_else(|| {
        error!("No username in validation response");
        "No username returned".to_string()
    })?;

    info!("Token validated successfully for user: {}", username);

    let credentials = ListenBrainzCredentials {
        user_token,
        username: Some(username),
    };

    // Store credentials
    *state.credentials.lock().unwrap() = Some(credentials.clone());

    Ok(credentials)
}

/// Submit a scrobble (past listen) to ListenBrainz
#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_submit_listen(
    listen: ListenBrainzListen,
    timestamp: f64,
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Not authenticated with ListenBrainz".to_string())?;

    debug!(
        "Submitting listen to ListenBrainz: {} - {}",
        listen.artist, listen.track
    );

    let client = state.client.lock().unwrap().clone();
    let url = format!("{}/submit-listens", LISTENBRAINZ_API_URL);

    let payload = ListenPayload {
        listen_type: "single".to_string(),
        payload: vec![PayloadItem {
            listened_at: Some(timestamp),
            track_metadata: TrackMetadata {
                artist_name: listen.artist.clone(),
                track_name: listen.track.clone(),
                release_name: listen.album.clone(),
                additional_info: listen.duration.map(|d| AdditionalInfo {
                    duration_ms: Some(d * 1000.0), // Convert seconds to milliseconds
                }),
            },
        }],
    };

    debug!(
        "ListenBrainz payload: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("Token {}", credentials.user_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to submit listen: {}", e);
            format!("Network error: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Submit listen failed: {} - {}", status, body);
        return Err(format!("Failed to submit listen: {}", status));
    }

    debug!("Successfully submitted listen to ListenBrainz");
    Ok(())
}

/// Update "now playing" status on ListenBrainz
#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_playing_now(
    artist: String,
    track: String,
    album: Option<String>,
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Not authenticated with ListenBrainz".to_string())?;

    debug!("Updating ListenBrainz playing now: {} - {}", artist, track);

    let client = state.client.lock().unwrap().clone();
    let url = format!("{}/submit-listens", LISTENBRAINZ_API_URL);

    let payload = ListenPayload {
        listen_type: "playing_now".to_string(),
        payload: vec![PayloadItem {
            listened_at: None,
            track_metadata: TrackMetadata {
                artist_name: artist.clone(),
                track_name: track.clone(),
                release_name: album,
                additional_info: None,
            },
        }],
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Token {}", credentials.user_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to update playing now: {}", e);
            format!("Network error: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Playing now update failed: {} - {}", status, body);
        return Err(format!("Failed to update playing now: {}", status));
    }

    debug!("Successfully updated playing now on ListenBrainz");
    Ok(())
}

/// Set ListenBrainz credentials
#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_set_credentials(
    credentials: ListenBrainzCredentials,
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    info!("Setting ListenBrainz credentials");
    *state.credentials.lock().unwrap() = Some(credentials);
    Ok(())
}

/// Clear ListenBrainz credentials
#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_clear_credentials(
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    info!("Clearing ListenBrainz credentials");
    *state.credentials.lock().unwrap() = None;
    Ok(())
}

/// Check if authenticated with ListenBrainz
#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_is_authenticated(
    state: State<'_, ListenBrainzState>,
) -> Result<bool, String> {
    let is_auth = state.credentials.lock().unwrap().is_some();
    Ok(is_auth)
}
