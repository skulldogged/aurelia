use crate::ApiResult;

pub async fn register_client_capabilities(
    server_url: String,
    token: String,
    device_id: String,
) -> ApiResult<()> {
    aurelia_core::register_client_capabilities(server_url, token, device_id).await
}

pub async fn report_playback_start(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    position_ticks: Option<i64>,
) -> ApiResult<()> {
    aurelia_core::report_playback_start_event(server_url, token, user_id, item_id, position_ticks)
        .await
}

pub async fn report_playback_progress(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    position_ticks: i64,
    is_paused: bool,
) -> ApiResult<()> {
    aurelia_core::report_playback_progress_event(
        server_url,
        token,
        user_id,
        item_id,
        position_ticks,
        is_paused,
    )
    .await
}

pub async fn report_playback_stop(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    position_ticks: i64,
) -> ApiResult<()> {
    aurelia_core::report_playback_stop_event(server_url, token, user_id, item_id, position_ticks)
        .await
}
