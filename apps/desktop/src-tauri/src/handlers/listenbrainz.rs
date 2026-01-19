use aurelia_core::listenbrainz_core;
use aurelia_core::listenbrainz_core::{
    ListenBrainzCredentials, ListenBrainzListen, ListenBrainzState,
};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_validate_token(
    user_token: String,
    state: State<'_, ListenBrainzState>,
) -> Result<ListenBrainzCredentials, String> {
    listenbrainz_core::listenbrainz_validate_token(user_token, &state).await
}

#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_submit_listen(
    listen: ListenBrainzListen,
    timestamp: f64,
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    listenbrainz_core::listenbrainz_submit_listen(listen, timestamp, &state).await
}

#[tauri::command]
#[specta::specta]
pub async fn listenbrainz_playing_now(
    artist: String,
    track: String,
    album: Option<String>,
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    listenbrainz_core::listenbrainz_playing_now(artist, track, album, &state).await
}

#[tauri::command]
#[specta::specta]
pub fn listenbrainz_set_credentials(
    credentials: ListenBrainzCredentials,
    state: State<'_, ListenBrainzState>,
) -> Result<(), String> {
    listenbrainz_core::listenbrainz_set_credentials(credentials, &state)
}

#[tauri::command]
#[specta::specta]
pub fn listenbrainz_clear_credentials(state: State<'_, ListenBrainzState>) -> Result<(), String> {
    listenbrainz_core::listenbrainz_clear_credentials(&state)
}

#[tauri::command]
#[specta::specta]
pub fn listenbrainz_is_authenticated(state: State<'_, ListenBrainzState>) -> Result<bool, String> {
    listenbrainz_core::listenbrainz_is_authenticated(&state)
}
