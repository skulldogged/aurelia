#![cfg(target_os = "android")]

use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::Serializer};
use tauri::plugin::{Builder, PluginApi, PluginHandle, TauriPlugin};
use tauri::{AppHandle, Manager, Runtime, command};
use thiserror::Error;

const PLUGIN_IDENTIFIER: &str = "dev.pupbrained.aurelia.plugin.nowplaying";

#[derive(Debug, Error)]
pub enum NowPlayingError {
    #[error(transparent)]
    Invoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error("android now playing plugin error: {0}")]
    Plugin(String),
}

type Result<T> = std::result::Result<T, NowPlayingError>;

impl Serialize for NowPlayingError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingPayload {
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub artists: Vec<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub position_seconds: Option<f64>,
    #[serde(default)]
    pub is_playing: bool,
    #[serde(default)]
    pub has_next: bool,
    #[serde(default)]
    pub has_previous: bool,
    #[serde(default)]
    pub is_shuffled: bool,
    #[serde(default)]
    pub repeat_mode: Option<String>,
    #[serde(default)]
    pub artwork_url: Option<String>,
    #[serde(default)]
    pub artwork_path: Option<String>,
    #[serde(default)]
    pub artwork_data: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginResponse {
    success: bool,
    #[serde(default)]
    message: Option<String>,
}

impl PluginResponse {
    fn into_result(self) -> Result<()> {
        if self.success {
            Ok(())
        } else {
            Err(NowPlayingError::Plugin(
                self.message.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }
}

pub struct AndroidNowPlaying(PluginHandle<tauri::Wry>);

impl AndroidNowPlaying {
    fn new(handle: PluginHandle<tauri::Wry>) -> Self {
        Self(handle)
    }

    fn update(&self, payload: NowPlayingPayload) -> Result<()> {
        let response: PluginResponse = self.0.run_mobile_plugin("updateNowPlaying", payload)?;
        response.into_result()
    }

    fn clear(&self) -> Result<()> {
        let response: PluginResponse = self.0.run_mobile_plugin("clearNowPlaying", ())?;
        response.into_result()
    }
}

pub trait AndroidNowPlayingExt {
    fn android_now_playing(&self) -> &AndroidNowPlaying;
}

impl<T: Manager<tauri::Wry>> AndroidNowPlayingExt for T {
    fn android_now_playing(&self) -> &AndroidNowPlaying {
        self.state::<AndroidNowPlaying>().inner()
    }
}

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("android-now-playing")
        .invoke_handler(tauri::generate_handler![
            update_now_playing,
            clear_now_playing
        ])
        .setup(|app, api| {
            let plugin = mobile_init(app, api)?;
            app.manage(plugin);
            Ok(())
        })
        .build()
}

fn mobile_init<C: DeserializeOwned>(
    _app: &AppHandle<tauri::Wry>,
    api: PluginApi<tauri::Wry, C>,
) -> Result<AndroidNowPlaying> {
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "NowPlayingPlugin")?;
    Ok(AndroidNowPlaying::new(handle))
}

#[command]
#[specta::specta]
pub async fn update_now_playing(
    app: AppHandle<tauri::Wry>,
    payload: NowPlayingPayload,
) -> std::result::Result<(), String> {
    let sanitized_payload = NowPlayingPayload {
        artwork_url: None,
        artwork_path: None,
        artwork_data: None,
        ..payload
    };

    app.android_now_playing()
        .update(sanitized_payload)
        .map_err(|e| e.to_string())
}

#[command]
#[specta::specta]
pub async fn clear_now_playing(app: AppHandle<tauri::Wry>) -> std::result::Result<(), String> {
    app.android_now_playing().clear().map_err(|e| e.to_string())
}
