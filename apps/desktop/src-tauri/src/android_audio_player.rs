#![cfg(target_os = "android")]

//! Android audio player bridge
//!
//! Provides Tauri commands that forward to the Kotlin ExoPlayer plugin.
//! This allows the frontend to use the same command API on all platforms.

use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::Serializer};
use tauri::plugin::{Builder, PluginApi, PluginHandle, TauriPlugin};
use tauri::{AppHandle, Manager};
use thiserror::Error;
use tracing::info;

const PLUGIN_IDENTIFIER: &str = "dev.pupbrained.aurelia.plugin.audio";

#[derive(Debug, Error)]
pub enum AudioPlayerError {
    #[error(transparent)]
    Invoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error("android audio player plugin error: {0}")]
    Plugin(String),
}

type Result<T> = std::result::Result<T, AudioPlayerError>;

impl Serialize for AudioPlayerError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginResponse {
    #[allow(dead_code)]
    success: Option<bool>,
    value: Option<serde_json::Value>,
    #[allow(dead_code)]
    message: Option<String>,
}

pub struct AndroidAudioPlayer(PluginHandle<tauri::Wry>);

impl AndroidAudioPlayer {
    fn new(handle: PluginHandle<tauri::Wry>) -> Self {
        Self(handle)
    }

    pub fn audio_init(&self) -> Result<()> {
        let _: PluginResponse = self.0.run_mobile_plugin("audioInit", ())?;
        Ok(())
    }

    pub fn audio_play(
        &self,
        url: &str,
        token: &str,
        title: Option<&str>,
        artist: Option<&str>,
        album: Option<&str>,
        artwork_url: Option<&str>,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Args<'a> {
            url: &'a str,
            token: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            artist: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            album: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            artwork_url: Option<&'a str>,
        }
        let _: PluginResponse = self.0.run_mobile_plugin(
            "audioPlay",
            Args {
                url,
                token,
                title,
                artist,
                album,
                artwork_url,
            },
        )?;
        Ok(())
    }

    pub fn audio_pause(&self) -> Result<()> {
        let _: PluginResponse = self.0.run_mobile_plugin("audioPause", ())?;
        Ok(())
    }

    pub fn audio_resume(&self) -> Result<()> {
        let _: PluginResponse = self.0.run_mobile_plugin("audioResume", ())?;
        Ok(())
    }

    pub fn audio_stop(&self) -> Result<()> {
        let _: PluginResponse = self.0.run_mobile_plugin("audioStop", ())?;
        Ok(())
    }

    pub fn audio_set_volume(&self, volume: f32) -> Result<()> {
        #[derive(Serialize)]
        struct Args {
            volume: f32,
        }
        let _: PluginResponse = self
            .0
            .run_mobile_plugin("audioSetVolume", Args { volume })?;
        Ok(())
    }

    pub fn audio_get_volume(&self) -> Result<f32> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioGetVolume", ())?;
        Ok(resp
            .value
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0))
    }

    pub fn audio_is_playing(&self) -> Result<bool> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioIsPlaying", ())?;
        Ok(resp.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub fn audio_is_finished(&self) -> Result<bool> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioIsFinished", ())?;
        Ok(resp.value.and_then(|v| v.as_bool()).unwrap_or(true))
    }

    pub fn audio_get_position(&self) -> Result<f64> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioGetPosition", ())?;
        Ok(resp.value.and_then(|v| v.as_f64()).unwrap_or(0.0))
    }

    pub fn audio_seek(&self, position_secs: f64) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Args {
            position_secs: f64,
        }
        let _: PluginResponse = self
            .0
            .run_mobile_plugin("audioSeek", Args { position_secs })?;
        Ok(())
    }

    pub fn audio_prepare_next(&self, url: &str, token: &str) -> Result<()> {
        #[derive(Serialize)]
        struct Args<'a> {
            url: &'a str,
            token: &'a str,
        }
        let _: PluginResponse = self
            .0
            .run_mobile_plugin("audioPrepareNext", Args { url, token })?;
        Ok(())
    }

    pub fn audio_advance_gapless(&self) -> Result<bool> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioAdvanceGapless", ())?;
        Ok(resp
            .value
            .as_ref()
            .and_then(|v| v.get("success"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    pub fn audio_set_eq_enabled(&self, enabled: bool) -> Result<()> {
        #[derive(Serialize)]
        struct Args {
            enabled: bool,
        }
        let _: PluginResponse = self
            .0
            .run_mobile_plugin("audioSetEqEnabled", Args { enabled })?;
        Ok(())
    }

    pub fn audio_is_eq_enabled(&self) -> Result<bool> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioIsEqEnabled", ())?;
        Ok(resp.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub fn audio_set_eq_band(&self, band: u32, gain_db: f32) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Args {
            band: u32,
            gain_db: f32,
        }
        let _: PluginResponse = self
            .0
            .run_mobile_plugin("audioSetEqBand", Args { band, gain_db })?;
        Ok(())
    }

    pub fn audio_get_eq_band(&self, band: u32) -> Result<f32> {
        #[derive(Serialize)]
        struct Args {
            band: u32,
        }
        let resp: PluginResponse = self.0.run_mobile_plugin("audioGetEqBand", Args { band })?;
        Ok(resp
            .value
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0))
    }

    pub fn audio_get_all_eq_bands(&self) -> Result<Vec<f32>> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioGetAllEqBands", ())?;
        Ok(resp
            .value
            .and_then(|v| v.as_array().cloned())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            })
            .unwrap_or_else(|| vec![0.0; 5]))
    }

    pub fn audio_reset_eq(&self) -> Result<()> {
        let _: PluginResponse = self.0.run_mobile_plugin("audioResetEq", ())?;
        Ok(())
    }

    /// Enable or disable spectrum analyzer.
    /// On Android, requires RECORD_AUDIO permission. If permission is not granted,
    /// this returns Ok(()) but the analyzer won't actually be enabled.
    /// Use audio_check_record_permission to check permission status first.
    pub fn audio_set_analyzer_enabled(&self, enabled: bool) -> Result<()> {
        #[derive(Serialize)]
        struct Args {
            enabled: bool,
        }
        let _resp: PluginResponse = self
            .0
            .run_mobile_plugin("audioSetAnalyzerEnabled", Args { enabled })?;
        Ok(())
    }

    pub fn audio_is_analyzer_enabled(&self) -> Result<bool> {
        let resp: PluginResponse = self.0.run_mobile_plugin("audioIsAnalyzerEnabled", ())?;
        Ok(resp.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub fn audio_reinit(&self) -> Result<()> {
        let _: PluginResponse = self.0.run_mobile_plugin("audioReinit", ())?;
        Ok(())
    }

    pub fn audio_check_record_permission(&self) -> Result<bool> {
        // Kotlin returns { "granted": bool } directly, not wrapped in PluginResponse
        #[derive(Debug, Deserialize)]
        struct PermissionResponse {
            granted: bool,
        }
        let resp: PermissionResponse =
            self.0.run_mobile_plugin("audioCheckRecordPermission", ())?;
        info!("audio_check_record_permission response: {:?}", resp);
        Ok(resp.granted)
    }

    pub fn audio_request_record_permission(&self) -> Result<bool> {
        // Kotlin returns { "granted": bool, "status"?: string } directly
        #[derive(Debug, Deserialize)]
        struct PermissionResponse {
            granted: bool,
        }
        let resp: PermissionResponse = self
            .0
            .run_mobile_plugin("audioRequestRecordPermission", ())?;
        info!("audio_request_record_permission response: {:?}", resp);
        Ok(resp.granted)
    }
}

pub trait AndroidAudioPlayerExt {
    fn android_audio_player(&self) -> &AndroidAudioPlayer;
}

impl<T: Manager<tauri::Wry>> AndroidAudioPlayerExt for T {
    fn android_audio_player(&self) -> &AndroidAudioPlayer {
        self.state::<AndroidAudioPlayer>().inner()
    }
}

// ============================================================================
// Tauri Commands - These mirror the desktop audio commands
// ============================================================================

#[tauri::command]
#[specta::specta]
pub async fn audio_init(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_init()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn audio_play(
    app: AppHandle,
    url: String,
    token: String,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    artwork_url: Option<String>,
) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_play(
            &url,
            &token,
            title.as_deref(),
            artist.as_deref(),
            album.as_deref(),
            artwork_url.as_deref(),
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_pause(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_pause()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_resume(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_resume()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_stop(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_stop()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_set_volume(app: AppHandle, volume: f32) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_set_volume(volume)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_get_volume(app: AppHandle) -> std::result::Result<f32, String> {
    app.android_audio_player()
        .audio_get_volume()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_is_playing(app: AppHandle) -> std::result::Result<bool, String> {
    app.android_audio_player()
        .audio_is_playing()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_is_finished(app: AppHandle) -> std::result::Result<bool, String> {
    app.android_audio_player()
        .audio_is_finished()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_get_position(app: AppHandle) -> std::result::Result<f64, String> {
    app.android_audio_player()
        .audio_get_position()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn audio_seek(app: AppHandle, position_secs: f64) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_seek(position_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_prepare_next(
    app: AppHandle,
    url: String,
    token: String,
) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_prepare_next(&url, &token)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn audio_advance_gapless(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_advance_gapless()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_set_eq_enabled(app: AppHandle, enabled: bool) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_set_eq_enabled(enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_is_eq_enabled(app: AppHandle) -> std::result::Result<bool, String> {
    app.android_audio_player()
        .audio_is_eq_enabled()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_set_eq_band(
    app: AppHandle,
    band: u32,
    gain_db: f32,
) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_set_eq_band(band, gain_db)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_get_eq_band(app: AppHandle, band: u32) -> std::result::Result<f32, String> {
    app.android_audio_player()
        .audio_get_eq_band(band)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_get_all_eq_bands(app: AppHandle) -> std::result::Result<Vec<f32>, String> {
    app.android_audio_player()
        .audio_get_all_eq_bands()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_reset_eq(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_reset_eq()
        .map_err(|e| e.to_string())
}

/// Enable or disable spectrum analyzer. On Android, requires RECORD_AUDIO permission.
/// Use audio_check_record_permission to verify permission before enabling.
#[tauri::command]
#[specta::specta]
pub fn audio_set_analyzer_enabled(
    app: AppHandle,
    enabled: bool,
) -> std::result::Result<(), String> {
    info!(
        "audio_set_analyzer_enabled command called: enabled={}",
        enabled
    );
    let result = app
        .android_audio_player()
        .audio_set_analyzer_enabled(enabled)
        .map_err(|e| e.to_string());
    info!("audio_set_analyzer_enabled command result: {:?}", result);
    result
}

#[tauri::command]
#[specta::specta]
pub fn audio_is_analyzer_enabled(app: AppHandle) -> std::result::Result<bool, String> {
    app.android_audio_player()
        .audio_is_analyzer_enabled()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_reinit(app: AppHandle) -> std::result::Result<(), String> {
    app.android_audio_player()
        .audio_reinit()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn audio_check_record_permission(app: AppHandle) -> std::result::Result<bool, String> {
    app.android_audio_player()
        .audio_check_record_permission()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn audio_request_record_permission(app: AppHandle) -> std::result::Result<bool, String> {
    app.android_audio_player()
        .audio_request_record_permission()
        .map_err(|e| e.to_string())
}

// ============================================================================
// Media Controls Commands (stubs for Android - handled natively by MediaSession)
// ============================================================================

/// Payload for updating Now Playing metadata (matches desktop)
#[derive(Clone, Debug, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingPayload {
    pub title: String,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub duration_secs: Option<f64>,
    #[serde(default)]
    pub cover_url: Option<String>,
}

/// Update Now Playing metadata - stub for Android (handled by MediaSession in Kotlin)
#[tauri::command]
#[specta::specta]
pub fn media_update_now_playing(_payload: NowPlayingPayload) -> std::result::Result<(), String> {
    // On Android, MediaSession metadata is set directly in Kotlin when audio_play is called.
    // This command exists for API compatibility with desktop.
    Ok(())
}

/// Update playback status - stub for Android (handled by MediaSession in Kotlin)
#[tauri::command]
#[specta::specta]
pub fn media_set_playback_status(
    _is_playing: bool,
    _position_secs: Option<f64>,
) -> std::result::Result<(), String> {
    // On Android, playback state is managed by ExoPlayer's MediaSession integration.
    Ok(())
}

/// Clear Now Playing - stub for Android (handled by MediaSession in Kotlin)
#[tauri::command]
#[specta::specta]
pub fn media_clear_now_playing() -> std::result::Result<(), String> {
    // On Android, stopping playback automatically clears the notification.
    Ok(())
}

/// Enable/disable media button - stub for Android (handled by MediaSession in Kotlin)
#[tauri::command]
#[specta::specta]
pub fn media_set_button_enabled(
    _button: String,
    _enabled: bool,
) -> std::result::Result<(), String> {
    // On Android, media button availability is determined by the MediaSession callbacks.
    Ok(())
}

// ============================================================================
// Plugin initialization
// ============================================================================

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("android-audio-player")
        .setup(|app, api| {
            info!("Initializing Android audio player plugin");
            let plugin = mobile_init(app, api)?;
            app.manage(plugin);
            Ok(())
        })
        .build()
}

fn mobile_init<C: DeserializeOwned>(
    _app: &AppHandle<tauri::Wry>,
    api: PluginApi<tauri::Wry, C>,
) -> Result<AndroidAudioPlayer> {
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "AudioPlayerPlugin")?;
    Ok(AndroidAudioPlayer::new(handle))
}
