//! Jellyfin API-specific data models

use serde::{Deserialize, Serialize};
use specta::Type;

/// Jellyfin lyrics response
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct JellyfinLyrics {
    /// List of lyric lines with timestamps
    #[serde(rename = "Lyrics")]
    #[specta(rename = "lyrics")]
    pub lyrics: Vec<JellyfinLyricLine>,
}

/// Individual lyric line with optional timestamp
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct JellyfinLyricLine {
    /// Lyric text
    #[serde(rename = "Text")]
    #[specta(rename = "text")]
    pub text: String,
    /// Timestamp in ticks (100ns intervals from start)
    #[serde(rename = "Start")]
    #[specta(rename = "timestamp")]
    pub timestamp: Option<f64>,
}

/// Client capabilities for session registration
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// Supported media types for playback
    #[serde(rename = "PlayableMediaTypes")]
    pub playable_media_types: Vec<String>,
    /// Supported commands
    #[serde(rename = "SupportedCommands")]
    pub supported_commands: Vec<String>,
    /// Whether the client supports media control
    #[serde(rename = "SupportsMediaControl")]
    pub supports_media_control: bool,
    /// Whether the client supports persistent identifier
    #[serde(rename = "SupportsPersistentIdentifier")]
    pub supports_persistent_identifier: bool,
    /// Application version
    #[serde(rename = "AppVersion")]
    pub app_version: String,
    /// Application name
    #[serde(rename = "AppName")]
    pub app_name: String,
    /// Device name
    #[serde(rename = "DeviceName")]
    pub device_name: String,
    /// Device ID
    #[serde(rename = "DeviceId")]
    pub device_id: String,
}

/// Playback information for session reporting
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct PlaybackInfo {
    /// Item ID being played
    #[serde(rename = "ItemId")]
    pub item_id: String,
    /// Session ID
    #[serde(rename = "SessionId")]
    pub session_id: Option<String>,
    /// Media source ID
    #[serde(rename = "MediaSourceId")]
    pub media_source_id: String,
    /// Audio stream index
    #[serde(rename = "AudioStreamIndex")]
    pub audio_stream_index: Option<i32>,
    /// Subtitle stream index
    #[serde(rename = "SubtitleStreamIndex")]
    pub subtitle_stream_index: Option<i32>,
    /// Whether this is a live stream
    #[serde(rename = "IsPaused")]
    pub is_paused: bool,
    /// Whether playback is muted
    #[serde(rename = "IsMuted")]
    pub is_muted: bool,
    /// Current position in ticks
    #[serde(rename = "PositionTicks")]
    pub position_ticks: Option<f64>,
    /// Current volume level (0-100)
    #[serde(rename = "VolumeLevel")]
    pub volume_level: Option<i32>,
    /// Playback rate (1.0 = normal speed)
    #[serde(rename = "PlayMethod")]
    pub play_method: Option<String>,
    /// How the item is being played (Direct, Transcode, etc.)
    #[serde(rename = "PlaySessionId")]
    pub play_session_id: Option<String>,
    /// Repeat mode
    #[serde(rename = "RepeatMode")]
    pub repeat_mode: Option<String>,
}

/// Progress update for ongoing playback
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct PlaybackProgress {
    /// Item ID being played
    #[serde(rename = "ItemId")]
    pub item_id: String,
    /// Session ID
    #[serde(rename = "SessionId")]
    pub session_id: Option<String>,
    /// Media source ID
    #[serde(rename = "MediaSourceId")]
    pub media_source_id: String,
    /// Current position in ticks
    #[serde(rename = "PositionTicks")]
    pub position_ticks: Option<f64>,
    /// Whether playback is paused
    #[serde(rename = "IsPaused")]
    pub is_paused: bool,
    /// Whether playback is muted
    #[serde(rename = "IsMuted")]
    pub is_muted: bool,
    /// Current volume level (0-100)
    #[serde(rename = "VolumeLevel")]
    pub volume_level: Option<i32>,
    /// Playback rate
    #[serde(rename = "PlayMethod")]
    pub play_method: Option<String>,
    /// Play session ID
    #[serde(rename = "PlaySessionId")]
    pub play_session_id: Option<String>,
    /// Repeat mode
    #[serde(rename = "RepeatMode")]
    pub repeat_mode: Option<String>,
    /// Whether shuffle is enabled
    #[serde(rename = "ShuffleMode")]
    pub shuffle_mode: Option<String>,
    /// Live stream ID
    #[serde(rename = "LiveStreamId")]
    pub live_stream_id: Option<String>,
    /// Playback start time ticks
    #[serde(rename = "PlaybackStartTimeTicks")]
    pub playback_start_time_ticks: Option<f64>,
    /// Event type
    #[serde(rename = "EventName")]
    pub event_name: Option<String>,
}

/// Stop playback info
#[derive(Serialize, Deserialize, Debug, Type)]
#[specta(rename_all = "camelCase")]
pub struct PlaybackStop {
    /// Item ID that was playing
    #[serde(rename = "ItemId")]
    pub item_id: String,
    /// Session ID
    #[serde(rename = "SessionId")]
    pub session_id: Option<String>,
    /// Media source ID
    #[serde(rename = "MediaSourceId")]
    pub media_source_id: String,
    /// Position when stopped in ticks
    #[serde(rename = "PositionTicks")]
    pub position_ticks: Option<f64>,
    /// Play session ID
    #[serde(rename = "PlaySessionId")]
    pub play_session_id: Option<String>,
}

/// Progress update data for frontend
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct PlaybackProgressData {
    pub item_id: String,
    pub position_ticks: Option<f64>,
    pub is_paused: bool,
    pub volume_level: Option<i32>,
    pub is_muted: bool,
    pub repeat_mode: Option<String>,
    pub shuffle_mode: Option<String>,
}
