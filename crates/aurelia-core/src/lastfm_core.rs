use crate::models::LastFmCredentials;
use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

const LASTFM_API_URL: &str = "https://ws.audioscrobbler.com/2.0/";

#[derive(Default)]
pub struct LastFmState {
    credentials: Arc<Mutex<Option<LastFmCredentials>>>,
    api_secret: Arc<Mutex<Option<String>>>,
    client: Arc<Mutex<Client>>,
}

impl LastFmState {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(None)),
            api_secret: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(Client::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct LastFmErrorResponse {
    error: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct LastFmSessionResponse {
    session: LastFmSessionInfo,
}

#[derive(Debug, Deserialize)]
struct LastFmSessionInfo {
    name: String,
    key: String,
}

fn build_api_signature(params: &[(String, String)], api_secret: &str) -> String {
    let mut sorted = params.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut signature = String::new();
    for (key, value) in sorted {
        signature.push_str(&key);
        signature.push_str(&value);
    }
    signature.push_str(api_secret);

    format!("{:x}", md5::compute(signature))
}

async fn send_lastfm_request(
    client: &Client,
    mut params: Vec<(String, String)>,
    api_secret: &str,
) -> Result<serde_json::Value, String> {
    let signature_params: Vec<(String, String)> = params
        .iter()
        .filter(|(key, _)| key != "format")
        .cloned()
        .collect();

    let api_sig = build_api_signature(&signature_params, api_secret);
    params.push(("api_sig".to_string(), api_sig));
    params.push(("format".to_string(), "json".to_string()));

    let response = client
        .post(LASTFM_API_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Last.fm request failed: {e}");
            format!("Network error: {e}")
        })?;

    let body = response.text().await.map_err(|e| {
        error!("Failed to read Last.fm response: {e}");
        format!("Failed to read response: {e}")
    })?;

    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        error!("Failed to parse Last.fm response: {e}");
        format!("Failed to parse response: {e}")
    })?;

    if let Ok(err) = serde_json::from_value::<LastFmErrorResponse>(json.clone())
        && err.error != 0
    {
        return Err(format!("Last.fm error {}: {}", err.error, err.message));
    }

    Ok(json)
}

pub fn lastfm_set_api_secret(api_secret: String, state: &LastFmState) -> Result<(), String> {
    *state.api_secret.lock().unwrap() = Some(api_secret);
    Ok(())
}

pub fn lastfm_set_credentials(
    credentials: LastFmCredentials,
    state: &LastFmState,
) -> Result<(), String> {
    info!("Setting Last.fm credentials");
    *state.credentials.lock().unwrap() = Some(credentials);
    Ok(())
}

pub fn lastfm_clear_credentials(state: &LastFmState) -> Result<(), String> {
    info!("Clearing Last.fm credentials");
    *state.credentials.lock().unwrap() = None;
    *state.api_secret.lock().unwrap() = None;
    Ok(())
}

pub fn lastfm_is_authenticated(state: &LastFmState) -> Result<bool, String> {
    Ok(state.credentials.lock().unwrap().is_some())
}

pub async fn lastfm_authenticate(
    api_key: String,
    api_secret: String,
    token: String,
    state: &LastFmState,
) -> Result<LastFmCredentials, String> {
    info!("Authenticating with Last.fm");

    let client = state.client.lock().unwrap().clone();
    let params = vec![
        ("api_key".to_string(), api_key.clone()),
        ("method".to_string(), "auth.getSession".to_string()),
        ("token".to_string(), token),
    ];

    let json = send_lastfm_request(&client, params, &api_secret).await?;
    let session: LastFmSessionResponse = serde_json::from_value(json).map_err(|e| {
        error!("Failed to parse Last.fm session: {e}");
        format!("Failed to parse session response: {e}")
    })?;

    let credentials = LastFmCredentials {
        session_key: session.session.key,
        username: session.session.name,
        api_key: Some(api_key),
    };

    *state.credentials.lock().unwrap() = Some(credentials.clone());
    *state.api_secret.lock().unwrap() = Some(api_secret);

    Ok(credentials)
}

pub async fn lastfm_scrobble(
    artist: String,
    track: String,
    album: Option<String>,
    timestamp: Option<i64>,
    state: &LastFmState,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Not authenticated with Last.fm".to_string())?;

    let api_secret = state
        .api_secret
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Last.fm API secret not available".to_string())?;

    let ts = timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp());

    let api_key = credentials
        .api_key
        .clone()
        .ok_or_else(|| "Last.fm API key not available".to_string())?;

    let mut params = vec![
        ("api_key".to_string(), api_key),
        ("method".to_string(), "track.scrobble".to_string()),
        ("sk".to_string(), credentials.session_key),
        ("artist".to_string(), artist),
        ("track".to_string(), track),
        ("timestamp".to_string(), ts.to_string()),
    ];

    if let Some(album) = album {
        params.push(("album".to_string(), album));
    }

    let client = state.client.lock().unwrap().clone();
    let _ = send_lastfm_request(&client, params, &api_secret).await?;

    debug!("Successfully scrobbled track on Last.fm");
    Ok(())
}

pub async fn lastfm_update_now_playing(
    artist: String,
    track: String,
    album: Option<String>,
    state: &LastFmState,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Not authenticated with Last.fm".to_string())?;

    let api_secret = state
        .api_secret
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Last.fm API secret not available".to_string())?;

    let api_key = credentials
        .api_key
        .clone()
        .ok_or_else(|| "Last.fm API key not available".to_string())?;

    let mut params = vec![
        ("api_key".to_string(), api_key),
        ("method".to_string(), "track.updateNowPlaying".to_string()),
        ("sk".to_string(), credentials.session_key),
        ("artist".to_string(), artist),
        ("track".to_string(), track),
    ];

    if let Some(album) = album {
        params.push(("album".to_string(), album));
    }

    let client = state.client.lock().unwrap().clone();
    let _ = send_lastfm_request(&client, params, &api_secret).await?;

    debug!("Successfully updated Last.fm now playing");
    Ok(())
}
