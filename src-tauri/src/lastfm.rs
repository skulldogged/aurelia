use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, warn};

const LASTFM_API_URL: &str = "https://ws.audioscrobbler.com/2.0/";
const CALLBACK_PORT: u16 = 3000;

// Helper functions for color manipulation
fn lighten_color(hex: &str, amount: f32) -> String {
    adjust_color_brightness(hex, amount)
}

fn darken_color(hex: &str, amount: f32) -> String {
    adjust_color_brightness(hex, -amount)
}

fn adjust_color_brightness(hex: &str, amount: f32) -> String {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let adjust = |c: u8| -> u8 {
        let adjusted = (c as f32 + (255.0 * amount)).clamp(0.0, 255.0);
        adjusted as u8
    };

    format!("#{:02x}{:02x}{:02x}", adjust(r), adjust(g), adjust(b))
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LastFmCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub session_key: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LastFmScrobble {
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub timestamp: f64,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LastFmAuthResponse {
    session: LastFmSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LastFmSession {
    name: String,
    key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LastFmErrorResponse {
    error: u32,
    message: String,
}

#[derive(Default)]
pub struct LastFmState {
    credentials: Arc<Mutex<Option<LastFmCredentials>>>,
    client: Arc<Mutex<Client>>,
}

impl LastFmState {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(Client::new())),
        }
    }
}

fn generate_signature(params: &HashMap<&str, String>, secret: &str) -> String {
    let mut sorted_params: Vec<(&str, &str)> =
        params.iter().map(|(k, v)| (*k, v.as_str())).collect();
    sorted_params.sort_by(|a, b| a.0.cmp(b.0));

    let concatenated = sorted_params
        .iter()
        .map(|(k, v)| format!("{}{}", k, v))
        .collect::<String>();

    let signature_string = format!("{}{}", concatenated, secret);
    format!("{:x}", md5::compute(signature_string))
}

#[tauri::command]
#[specta::specta]
pub async fn lastfm_authenticate(
    api_key: String,
    api_secret: String,
    token: String,
    state: State<'_, LastFmState>,
) -> Result<LastFmCredentials, String> {
    info!("Authenticating with Last.fm");

    let client = state.client.lock().unwrap().clone();

    let mut params = HashMap::new();
    params.insert("api_key", api_key.clone());
    params.insert("method", "auth.getSession".to_string());
    params.insert("token", token.clone());

    let api_sig = generate_signature(&params, &api_secret);
    params.insert("api_sig", api_sig);
    params.insert("format", "json".to_string());

    let response = client
        .post(LASTFM_API_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send authentication request: {}", e);
            format!("Failed to send authentication request: {}", e)
        })?;

    let status = response.status();
    let text = response.text().await.map_err(|e| {
        error!("Failed to read response: {}", e);
        format!("Failed to read response: {}", e)
    })?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_str::<LastFmErrorResponse>(&text) {
            error!("Last.fm authentication failed: {}", error.message);
            return Err(error.message);
        }
        return Err(format!("Authentication failed with status: {}", status));
    }

    let auth_response: LastFmAuthResponse = serde_json::from_str(&text).map_err(|e| {
        error!("Failed to parse authentication response: {}", e);
        format!("Failed to parse authentication response: {}", e)
    })?;

    let credentials = LastFmCredentials {
        api_key: api_key.clone(),
        api_secret: api_secret.clone(),
        session_key: Some(auth_response.session.key.clone()),
        username: Some(auth_response.session.name.clone()),
    };

    *state.credentials.lock().unwrap() = Some(credentials.clone());

    info!(
        "Successfully authenticated with Last.fm as {}",
        auth_response.session.name
    );

    Ok(credentials)
}

#[tauri::command]
#[specta::specta]
pub async fn lastfm_scrobble(
    scrobble: LastFmScrobble,
    state: State<'_, LastFmState>,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Not authenticated with Last.fm".to_string())?;

    let session_key = credentials
        .session_key
        .ok_or_else(|| "No session key available".to_string())?;

    debug!("Scrobbling track: {} - {}", scrobble.artist, scrobble.track);

    let client = state.client.lock().unwrap().clone();

    let mut params = HashMap::new();
    params.insert("api_key", credentials.api_key.clone());
    params.insert("method", "track.scrobble".to_string());
    params.insert("artist[0]", scrobble.artist.clone());
    params.insert("track[0]", scrobble.track.clone());
    params.insert("timestamp[0]", scrobble.timestamp.to_string());
    params.insert("sk", session_key.clone());

    if let Some(album) = &scrobble.album {
        params.insert("album[0]", album.clone());
    }

    if let Some(duration) = scrobble.duration {
        params.insert("duration[0]", duration.to_string());
    }

    let api_sig = generate_signature(&params, &credentials.api_secret);
    params.insert("api_sig", api_sig);
    params.insert("format", "json".to_string());

    let response = client
        .post(LASTFM_API_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send scrobble request: {}", e);
            format!("Failed to send scrobble request: {}", e)
        })?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        if let Ok(error) = serde_json::from_str::<LastFmErrorResponse>(&text) {
            error!("Last.fm scrobble failed: {}", error.message);
            return Err(error.message);
        }
        return Err(format!("Scrobble failed with status: {}", status));
    }

    info!(
        "Successfully scrobbled: {} - {}",
        scrobble.artist, scrobble.track
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn lastfm_update_now_playing(
    artist: String,
    track: String,
    album: Option<String>,
    state: State<'_, LastFmState>,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Not authenticated with Last.fm".to_string())?;

    let session_key = credentials
        .session_key
        .ok_or_else(|| "No session key available".to_string())?;

    debug!("Updating now playing: {} - {}", artist, track);

    let client = state.client.lock().unwrap().clone();

    let mut params = HashMap::new();
    params.insert("api_key", credentials.api_key.clone());
    params.insert("method", "track.updateNowPlaying".to_string());
    params.insert("artist", artist.clone());
    params.insert("track", track.clone());
    params.insert("sk", session_key.clone());

    if let Some(album_name) = album {
        params.insert("album", album_name);
    }

    let api_sig = generate_signature(&params, &credentials.api_secret);
    params.insert("api_sig", api_sig);
    params.insert("format", "json".to_string());

    let response = client
        .post(LASTFM_API_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to update now playing: {}", e);
            format!("Failed to update now playing: {}", e)
        })?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        if let Ok(error) = serde_json::from_str::<LastFmErrorResponse>(&text) {
            warn!("Last.fm now playing update failed: {}", error.message);
            return Err(error.message);
        }
        return Err(format!("Now playing update failed with status: {}", status));
    }

    debug!("Successfully updated now playing");

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn lastfm_set_credentials(
    credentials: LastFmCredentials,
    state: State<LastFmState>,
) -> Result<(), String> {
    info!("Setting Last.fm credentials");
    *state.credentials.lock().unwrap() = Some(credentials);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn lastfm_clear_credentials(state: State<LastFmState>) -> Result<(), String> {
    info!("Clearing Last.fm credentials");
    *state.credentials.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn lastfm_is_authenticated(state: State<LastFmState>) -> Result<bool, String> {
    let credentials = state.credentials.lock().unwrap();
    Ok(credentials
        .as_ref()
        .is_some_and(|c| c.session_key.is_some()))
}

#[tauri::command]
#[specta::specta]
pub fn lastfm_start_auth_server(
    app: AppHandle,
    primary_color: String,
    background_color: String,
    text_color: String,
) -> Result<(), String> {
    info!(
        "Starting Last.fm OAuth callback server on port {}",
        CALLBACK_PORT
    );

    thread::spawn(move || {
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT)) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to start callback server: {}", e);
                return;
            }
        };

        info!(
            "Callback server listening on http://127.0.0.1:{}",
            CALLBACK_PORT
        );

        // Only handle one request then stop
        if let Ok((mut stream, _)) = listener.accept() {
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();

            if reader.read_line(&mut request_line).is_ok() {
                debug!("Received callback request: {}", request_line);

                // Parse URL from request line (e.g., "GET /?token=abc123 HTTP/1.1")
                if let Some(url_part) = request_line.split_whitespace().nth(1) {
                    // Parse token from URL
                    if let Some(query_start) = url_part.find('?') {
                        let query = &url_part[query_start + 1..];
                        for param in query.split('&') {
                            if let Some((key, value)) = param.split_once('=')
                                && key == "token"
                            {
                                info!("Received Last.fm token via callback");

                                // Emit event to frontend with the token
                                if let Err(e) =
                                    app.emit("lastfm://token-received", value.to_string())
                                {
                                    error!("Failed to emit token event: {}", e);
                                }

                                // Send a nice HTML response with user's theme colors
                                let html = format!(
                                    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Last.fm Authorization</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            background: {background};
        }}
        .container {{
            background: {bg_card};
            padding: 3rem;
            border-radius: 1rem;
            text-align: center;
            max-width: 500px;
            border: 1px solid {border};
        }}
        h1 {{
            color: {text};
            margin-bottom: 1rem;
            font-size: 2rem;
        }}
        p {{
            color: {text};
            opacity: 0.8;
            line-height: 1.6;
            margin-bottom: 1.5rem;
        }}
        .success {{
            color: {primary};
            font-weight: 600;
            font-size: 1.1rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>✓ Authorization Successful</h1>
        <p class="success">You can now close this window and return to the application.</p>
        <p>The authorization token has been automatically sent to Aurelia.</p>
    </div>
</body>
</html>"#,
                                    background = &background_color,
                                    bg_card = darken_color(&background_color, 0.05),
                                    border = lighten_color(&background_color, 0.1),
                                    text = &text_color,
                                    primary = &primary_color
                                );

                                let response = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    html.len(),
                                    html
                                );

                                let _ = stream.write_all(response.as_bytes());
                                let _ = stream.flush();
                                return;
                            }
                        }
                    }

                    // No token found - send error page
                    let html = format!(
                        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Last.fm Authorization</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            background: {background};
        }}
        .container {{
            background: {bg_card};
            padding: 3rem;
            border-radius: 1rem;
            text-align: center;
            max-width: 500px;
            border: 1px solid {border};
        }}
        h1 {{
            color: {text};
            margin-bottom: 1rem;
            font-size: 2rem;
        }}
        p {{
            color: {text};
            opacity: 0.8;
            line-height: 1.6;
        }}
        .error {{
            color: #e53e3e;
            font-weight: 600;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>✗ Authorization Failed</h1>
        <p class="error">No token received from Last.fm</p>
        <p>Please try again or contact support if the issue persists.</p>
    </div>
</body>
</html>"#,
                        background = &background_color,
                        bg_card = darken_color(&background_color, 0.05),
                        border = lighten_color(&background_color, 0.1),
                        text = &text_color
                    );

                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        html.len(),
                        html
                    );

                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
            }
        }
    });

    Ok(())
}
