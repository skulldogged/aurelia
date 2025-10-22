#![cfg(target_os = "android")]

use serde::{de::DeserializeOwned, ser::Serializer, Deserialize, Serialize};
use tauri::plugin::{Builder, PluginApi, PluginHandle, TauriPlugin};
use tauri::{command, AppHandle, Manager, Runtime};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            Err(NowPlayingError::Plugin(self.message.unwrap_or_else(|| "Unknown error".to_string())))
        }
    }
}

pub struct AndroidNowPlaying<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> AndroidNowPlaying<R> {
    fn new(handle: PluginHandle<R>) -> Self {
        Self(handle)
    }

    fn update(&self, payload: NowPlayingPayload) -> Result<()> {
        let response: PluginResponse = self
            .0
            .run_mobile_plugin("updateNowPlaying", payload)?;
        response.into_result()
    }

    fn clear(&self) -> Result<()> {
        let response: PluginResponse = self.0.run_mobile_plugin("clearNowPlaying", ())?;
        response.into_result()
    }
}

pub trait AndroidNowPlayingExt<R: Runtime> {
    fn android_now_playing(&self) -> &AndroidNowPlaying<R>;
}

impl<R: Runtime, T: Manager<R>> AndroidNowPlayingExt<R> for T {
    fn android_now_playing(&self) -> &AndroidNowPlaying<R> {
        self.state::<AndroidNowPlaying<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("android-now-playing")
        .invoke_handler(tauri::generate_handler![update_now_playing, clear_now_playing])
        .setup(|app, api| {
            let plugin = mobile_init(app, api)?;
            app.manage(plugin);
            Ok(())
        })
        .build()
}

fn mobile_init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<AndroidNowPlaying<R>> {
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "NowPlayingPlugin")?;
    Ok(AndroidNowPlaying::new(handle))
}

#[command]
pub async fn update_now_playing<R: Runtime>(app: AppHandle<R>, payload: NowPlayingPayload) -> Result<()> {
    app.android_now_playing().update(payload)
}

#[command]
pub async fn clear_now_playing<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.android_now_playing().clear()
}
