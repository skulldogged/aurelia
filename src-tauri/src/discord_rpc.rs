use anyhow::Result;
use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, ActivityType, Assets, Button, Timestamps},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{Arc, Mutex};
use tauri::State;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RpcActivity {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub start_timestamp: Option<i32>,
    pub end_timestamp: Option<i32>,
    pub buttons: Option<Vec<RpcButton>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RpcButton {
    pub label: String,
    pub url: String,
}

#[derive(Default)]
pub struct DiscordRpcState {
    client: Arc<Mutex<Option<DiscordIpcClient>>>,
    app_id: Arc<Mutex<Option<String>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl DiscordRpcState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            app_id: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn discord_rpc_start(app_id: String, state: State<DiscordRpcState>) -> Result<(), String> {
    info!("Starting Discord RPC with app ID: {}", app_id);

    // Store the app ID
    {
        let mut stored_app_id = state.app_id.lock().unwrap();
        *stored_app_id = Some(app_id.clone());
    }

    let mut client_lock = state.client.lock().unwrap();
    let mut is_connected = state.is_connected.lock().unwrap();

    // Close existing client if any
    if let Some(mut client) = client_lock.take() {
        let _ = client.close();
        debug!("Closed existing Discord RPC client");
    }

    // Create new client
    let mut client = DiscordIpcClient::new(&app_id).map_err(|e| {
        error!("Failed to create Discord IPC client: {}", e);
        format!("Failed to create Discord IPC client: {}", e)
    })?;

    // Connect to Discord
    client.connect().map_err(|e| {
        error!("Failed to connect to Discord: {}", e);
        format!("Failed to connect to Discord: {}", e)
    })?;

    info!("Successfully connected to Discord");

    *client_lock = Some(client);
    *is_connected = true;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn discord_rpc_stop(state: State<DiscordRpcState>) -> Result<(), String> {
    debug!("Stopping Discord RPC");

    let mut client_lock = state.client.lock().unwrap();
    let mut is_connected = state.is_connected.lock().unwrap();

    if let Some(mut client) = client_lock.take() {
        client.close().map_err(|e| {
            error!("Failed to close Discord RPC client: {}", e);
            format!("Failed to close Discord RPC client: {}", e)
        })?;
        info!("Discord RPC client closed successfully");
    }

    *is_connected = false;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn discord_rpc_is_running(state: State<DiscordRpcState>) -> Result<bool, String> {
    let is_connected = state.is_connected.lock().unwrap();
    Ok(*is_connected)
}

#[tauri::command]
#[specta::specta]
pub fn discord_rpc_set_activity(
    activity: RpcActivity,
    state: State<DiscordRpcState>,
) -> Result<(), String> {
    debug!("Setting Discord activity: {:?}", activity);

    let mut client_lock = state.client.lock().unwrap();
    let is_connected = state.is_connected.lock().unwrap();

    if !*is_connected {
        warn!("Cannot set activity: Discord RPC is not connected");
        return Err("Discord RPC is not connected".to_string());
    }

    let client = client_lock.as_mut().ok_or_else(|| {
        error!("Discord RPC client is not initialized");
        "Discord RPC client is not initialized".to_string()
    })?;

    let mut rpc_activity = Activity::new().activity_type(ActivityType::Listening);

    if let Some(details) = &activity.details {
        rpc_activity = rpc_activity.details(details);
    }

    if let Some(state_text) = &activity.state {
        rpc_activity = rpc_activity.state(state_text);
    }

    // Handle assets (images)
    if activity.large_image.is_some()
        || activity.large_text.is_some()
        || activity.small_image.is_some()
        || activity.small_text.is_some()
    {
        let mut assets = Assets::new();

        if let Some(large_image) = &activity.large_image {
            assets = assets.large_image(large_image);
        }

        if let Some(large_text) = &activity.large_text {
            assets = assets.large_text(large_text);
        }

        if let Some(small_image) = &activity.small_image {
            assets = assets.small_image(small_image);
        }

        if let Some(small_text) = &activity.small_text {
            assets = assets.small_text(small_text);
        }

        rpc_activity = rpc_activity.assets(assets);
    }

    // Handle timestamps
    if activity.start_timestamp.is_some() || activity.end_timestamp.is_some() {
        let mut timestamps = Timestamps::new();

        if let Some(start) = activity.start_timestamp {
            timestamps = timestamps.start(start.into());
        }

        if let Some(end) = activity.end_timestamp {
            timestamps = timestamps.end(end.into());
        }

        rpc_activity = rpc_activity.timestamps(timestamps);
    }

    // Handle buttons
    if let Some(buttons) = &activity.buttons {
        for button in buttons {
            rpc_activity = rpc_activity.buttons(vec![Button::new(&button.label, &button.url)]);
        }
    }

    client.set_activity(rpc_activity).map_err(|e| {
        error!("Failed to set Discord activity: {}", e);

        // If we failed to set activity, the connection might be dead
        let mut is_connected_mut = state.is_connected.lock().unwrap();
        *is_connected_mut = false;

        format!("Failed to set Discord activity: {}", e)
    })?;

    debug!("Discord activity set successfully");

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn discord_rpc_clear_activity(state: State<DiscordRpcState>) -> Result<(), String> {
    debug!("Clearing Discord activity");

    let mut client_lock = state.client.lock().unwrap();
    let is_connected = state.is_connected.lock().unwrap();

    if !*is_connected {
        warn!("Cannot clear activity: Discord RPC is not connected");
        return Ok(());
    }

    let client = client_lock.as_mut().ok_or_else(|| {
        error!("Discord RPC client is not initialized");
        "Discord RPC client is not initialized".to_string()
    })?;

    client.clear_activity().map_err(|e| {
        error!("Failed to clear Discord activity: {}", e);

        // If we failed to clear activity, the connection might be dead
        let mut is_connected_mut = state.is_connected.lock().unwrap();
        *is_connected_mut = false;

        format!("Failed to clear Discord activity: {}", e)
    })?;

    debug!("Discord activity cleared successfully");

    Ok(())
}
