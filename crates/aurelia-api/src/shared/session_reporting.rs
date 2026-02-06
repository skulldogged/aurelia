use crate::ApiResult;
use aurelia_core::models::jellyfin::{
    ClientCapabilities, DeviceProfile, DirectPlayProfile, SubtitleProfile, TranscodingProfile,
};
use aurelia_core::services::JellyfinClient;

fn default_audio_capabilities(device_id: String) -> ClientCapabilities {
    let device_profile = DeviceProfile {
        name: Some("Aurelia Audio Profile".to_string()),
        id: Some(device_id),
        max_streaming_bitrate: Some(140000000),
        max_static_bitrate: Some(140000000),
        music_streaming_transcoding_bitrate: Some(384000),
        max_static_music_bitrate: Some(4000000),
        direct_play_profiles: vec![
            DirectPlayProfile {
                container: "mp3".to_string(),
                audio_codec: Some("mp3".to_string()),
                video_codec: None,
                profile_type: "Audio".to_string(),
            },
            DirectPlayProfile {
                container: "flac".to_string(),
                audio_codec: Some("flac".to_string()),
                video_codec: None,
                profile_type: "Audio".to_string(),
            },
            DirectPlayProfile {
                container: "ogg".to_string(),
                audio_codec: Some("vorbis".to_string()),
                video_codec: None,
                profile_type: "Audio".to_string(),
            },
        ],
        transcoding_profiles: vec![TranscodingProfile {
            container: "mp3".to_string(),
            profile_type: "Audio".to_string(),
            video_codec: None,
            audio_codec: Some("mp3".to_string()),
            protocol: "http".to_string(),
            estimate_content_length: None,
            enable_mpegts_m2_ts_mode: None,
            transcode_seek_info: None,
            copy_timestamps: None,
            context: Some("Streaming".to_string()),
            enable_subtitles_in_manifest: None,
            max_audio_channels: None,
            min_segments: None,
            segment_length: None,
            break_on_non_key_frames: None,
            conditions: vec![],
            enable_audio_vbr_encoding: None,
        }],
        container_profiles: vec![],
        codec_profiles: vec![],
        subtitle_profiles: vec![SubtitleProfile {
            format: "srt".to_string(),
            method: "External".to_string(),
            didl_mode: None,
            language: None,
            container: None,
        }],
    };

    ClientCapabilities {
        playable_media_types: vec!["Audio".to_string()],
        supported_commands: vec![
            "PlayNow".to_string(),
            "PlayNext".to_string(),
            "SetVolume".to_string(),
            "ToggleMute".to_string(),
        ],
        supports_media_control: true,
        supports_persistent_identifier: true,
        device_profile,
        app_store_url: None,
        icon_url: None,
    }
}

pub async fn register_client_capabilities(
    server_url: String,
    token: String,
    device_id: String,
) -> ApiResult<()> {
    let client = JellyfinClient::with_auth(server_url, token);
    let capabilities = default_audio_capabilities(device_id);
    client.register_capabilities(&capabilities).await
}

pub async fn report_playback_start(
    server_url: String,
    token: String,
    item_id: String,
    position_ticks: Option<i64>,
) -> ApiResult<()> {
    let client = JellyfinClient::with_auth(server_url, token);
    client.report_playback_start(&item_id, position_ticks).await
}

pub async fn report_playback_progress(
    server_url: String,
    token: String,
    item_id: String,
    position_ticks: i64,
    is_paused: bool,
) -> ApiResult<()> {
    let client = JellyfinClient::with_auth(server_url, token);
    client
        .report_playback_progress(&item_id, Some(position_ticks), None, Some(is_paused))
        .await
}

pub async fn report_playback_stop(
    server_url: String,
    token: String,
    item_id: String,
    position_ticks: i64,
) -> ApiResult<()> {
    let client = JellyfinClient::with_auth(server_url, token);
    client
        .report_playback_stop(&item_id, Some(position_ticks))
        .await
}
