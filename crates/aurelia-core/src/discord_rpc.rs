//! Discord Rich Presence integration
//!
//! Provides Discord RPC functionality for showing "Now Playing" status.
//! Only available with the `desktop` feature.

use anyhow::Result;
use discord_rich_presence::{
    activity::{Activity, ActivityType, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

// Re-export RpcActivity from models
pub use crate::models::RpcActivity;

/// Button data for Discord Rich Presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcButton {
    pub label: String,
    pub url: String,
}

/// Discord RPC client state
pub struct DiscordRpcState {
    client: Arc<Mutex<Option<DiscordIpcClient>>>,
    app_id: Arc<Mutex<Option<String>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl Default for DiscordRpcState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscordRpcState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            app_id: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Start Discord RPC with the given application ID
    pub fn start(&self, app_id: String) -> Result<()> {
        info!("Starting Discord RPC with app ID: {}", app_id);

        // Store the app ID
        {
            let mut stored_app_id = self.app_id.lock().unwrap();
            *stored_app_id = Some(app_id.clone());
        }

        let mut client_lock = self.client.lock().unwrap();
        let mut is_connected = self.is_connected.lock().unwrap();

        // Close existing client if any
        if let Some(mut client) = client_lock.take() {
            let _ = client.close();
            debug!("Closed existing Discord RPC client");
        }

        // Create new client
        let mut client = DiscordIpcClient::new(&app_id);

        // Connect to Discord
        client.connect().map_err(|e| {
            error!("Failed to connect to Discord: {}", e);
            anyhow::anyhow!("Failed to connect to Discord: {}", e)
        })?;

        info!("Successfully connected to Discord");

        *client_lock = Some(client);
        *is_connected = true;

        Ok(())
    }

    /// Stop Discord RPC
    pub fn stop(&self) -> Result<()> {
        debug!("Stopping Discord RPC");

        let mut client_lock = self.client.lock().unwrap();
        let mut is_connected = self.is_connected.lock().unwrap();

        if let Some(mut client) = client_lock.take() {
            client.close().map_err(|e| {
                error!("Failed to close Discord RPC client: {}", e);
                anyhow::anyhow!("Failed to close Discord RPC client: {}", e)
            })?;
            info!("Discord RPC client closed successfully");
        }

        *is_connected = false;

        Ok(())
    }

    /// Check if Discord RPC is currently running
    pub fn is_running(&self) -> bool {
        let is_connected = self.is_connected.lock().unwrap();
        *is_connected
    }

    /// Set the Discord activity
    pub fn set_activity(&self, activity: RpcActivity) -> Result<()> {
        debug!("Setting Discord activity: {:?}", activity);

        let mut client_lock = self.client.lock().unwrap();
        let is_connected = self.is_connected.lock().unwrap();

        if !*is_connected {
            warn!("Cannot set activity: Discord RPC is not connected");
            return Err(anyhow::anyhow!("Discord RPC is not connected"));
        }

        let client = client_lock.as_mut().ok_or_else(|| {
            error!("Discord RPC client is not initialized");
            anyhow::anyhow!("Discord RPC client is not initialized")
        })?;

        let mut rpc_activity = Activity::new().activity_type(ActivityType::Listening);

        if let Some(details) = &activity.details {
            rpc_activity = rpc_activity.details(details);
        }

        if let Some(state_text) = &activity.state {
            rpc_activity = rpc_activity.state(state_text);
        }

        // Handle assets (images)
        if activity.large_image_key.is_some()
            || activity.large_image_text.is_some()
            || activity.small_image_key.is_some()
            || activity.small_image_text.is_some()
        {
            let mut assets = Assets::new();

            if let Some(large_image) = &activity.large_image_key {
                assets = assets.large_image(large_image);
            }

            if let Some(large_text) = &activity.large_image_text {
                assets = assets.large_text(large_text);
            }

            if let Some(small_image) = &activity.small_image_key {
                assets = assets.small_image(small_image);
            }

            if let Some(small_text) = &activity.small_image_text {
                assets = assets.small_text(small_text);
            }

            rpc_activity = rpc_activity.assets(assets);
        }

        // Handle timestamps
        if activity.start_timestamp.is_some() || activity.end_timestamp.is_some() {
            let mut timestamps = Timestamps::new();

            if let Some(start) = activity.start_timestamp {
                timestamps = timestamps.start(start);
            }

            if let Some(end) = activity.end_timestamp {
                timestamps = timestamps.end(end);
            }

            rpc_activity = rpc_activity.timestamps(timestamps);
        }

        client.set_activity(rpc_activity).map_err(|e| {
            error!("Failed to set Discord activity: {}", e);

            // If we failed to set activity, the connection might be dead
            drop(is_connected);
            let mut is_connected_mut = self.is_connected.lock().unwrap();
            *is_connected_mut = false;

            anyhow::anyhow!("Failed to set Discord activity: {}", e)
        })?;

        debug!("Discord activity set successfully");

        Ok(())
    }

    /// Clear the Discord activity
    pub fn clear_activity(&self) -> Result<()> {
        debug!("Clearing Discord activity");

        let mut client_lock = self.client.lock().unwrap();
        let is_connected = self.is_connected.lock().unwrap();

        if !*is_connected {
            warn!("Cannot clear activity: Discord RPC is not connected");
            return Ok(());
        }

        let client = client_lock.as_mut().ok_or_else(|| {
            error!("Discord RPC client is not initialized");
            anyhow::anyhow!("Discord RPC client is not initialized")
        })?;

        client.clear_activity().map_err(|e| {
            error!("Failed to clear Discord activity: {}", e);

            // If we failed to clear activity, the connection might be dead
            drop(is_connected);
            let mut is_connected_mut = self.is_connected.lock().unwrap();
            *is_connected_mut = false;

            anyhow::anyhow!("Failed to clear Discord activity: {}", e)
        })?;

        debug!("Discord activity cleared successfully");

        Ok(())
    }
}
