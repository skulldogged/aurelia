use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use aurelia_core::{
    audio::{self, AudioState},
    media_controls::{MediaControlsState, MediaEvent},
    models::NowPlayingPayload,
};

const PROGRESS_REPORT_INTERVAL_SECONDS: u64 = 10;

#[derive(Clone, Debug)]
pub struct PlaybackItem {
    pub server_url: String,
    pub token: String,
    pub user_id: String,
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_id: Option<String>,
    pub container: Option<String>,
    pub duration_seconds: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlaybackSnapshot {
    pub is_playing: bool,
    pub is_finished: bool,
    pub position_seconds: f64,
}

#[derive(Clone)]
pub struct PlaybackController {
    audio: Arc<AudioState>,
    media_controls: Arc<MediaControlsState>,
    current: Arc<Mutex<Option<CurrentPlayback>>>,
}

#[derive(Clone, Debug)]
struct CurrentPlayback {
    item: PlaybackItem,
    last_reported_second: u64,
}

impl PlaybackController {
    pub fn new() -> Self {
        let media_controls = Arc::new(MediaControlsState::new());
        if let Err(error) = media_controls.init(None) {
            tracing::warn!("Could not initialize desktop media controls: {error}");
        }

        Self {
            audio: Arc::new(AudioState::new()),
            media_controls,
            current: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn play(&self, item: PlaybackItem, volume: f32) -> Result<()> {
        self.stop().await?;

        audio::audio_init(&self.audio)
            .await
            .context("Could not initialize the audio output")?;
        audio::audio_set_volume(&self.audio, volume)
            .await
            .context("Could not set the playback volume")?;

        let stream_url = aurelia_core::build_desktop_stream_url(
            item.server_url.clone(),
            item.token.clone(),
            item.id.clone(),
            item.container.clone(),
        );
        audio::audio_play(&self.audio, stream_url, None, item.token.clone())
            .await
            .with_context(|| format!("Could not play {}", item.title))?;

        aurelia_core::report_playback_start_event(
            item.server_url.clone(),
            item.token.clone(),
            item.user_id.clone(),
            item.id.clone(),
            Some(0),
        )
        .await
        .ok();

        let cover_url = item.album_id.as_ref().and_then(|album_id| {
            aurelia_core::build_image_url(
                item.server_url.clone(),
                item.token.clone(),
                album_id.clone(),
                "Primary".to_string(),
                Some(400),
                Some(90),
            )
            .ok()
            .flatten()
        });
        self.media_controls
            .update_now_playing(NowPlayingPayload {
                title: item.title.clone(),
                artist: Some(item.artist.clone()),
                album: Some(item.album.clone()),
                duration: Some(f64::from(item.duration_seconds)),
                cover_url,
            })
            .ok();
        self.media_controls
            .set_playback_status(true, Some(0.0))
            .ok();

        *self.current.lock().expect("playback state poisoned") = Some(CurrentPlayback {
            item,
            last_reported_second: 0,
        });
        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        audio::audio_pause(&self.audio).await?;
        let position = audio::audio_get_position(&self.audio).await?;
        self.report_progress(position, true).await;
        self.media_controls
            .set_playback_status(false, Some(position))
            .ok();
        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        audio::audio_resume(&self.audio).await?;
        let position = audio::audio_get_position(&self.audio).await?;
        self.report_progress(position, false).await;
        self.media_controls
            .set_playback_status(true, Some(position))
            .ok();
        Ok(())
    }

    pub async fn seek(&self, position_seconds: f64) -> Result<()> {
        audio::audio_seek(&self.audio, position_seconds).await?;
        let is_paused = !audio::audio_is_playing(&self.audio).await?;
        self.report_progress(position_seconds, is_paused).await;
        self.media_controls
            .set_playback_status(!is_paused, Some(position_seconds))
            .ok();
        Ok(())
    }

    pub async fn set_volume(&self, volume: f32) -> Result<()> {
        audio::audio_set_volume(&self.audio, volume).await
    }

    pub async fn stop(&self) -> Result<()> {
        let current = self.current.lock().expect("playback state poisoned").take();
        let position = audio::audio_get_position(&self.audio).await.unwrap_or(0.0);

        if let Some(current) = current {
            aurelia_core::report_playback_stop_event(
                current.item.server_url,
                current.item.token,
                current.item.user_id,
                current.item.id,
                seconds_to_ticks(position),
            )
            .await
            .ok();
        }

        if audio::audio_stop(&self.audio).await.is_err() {
            // Stopping before the output has been initialized is intentionally a no-op.
        }
        self.media_controls.clear_now_playing().ok();
        Ok(())
    }

    pub async fn poll(&self) -> Result<PlaybackSnapshot> {
        let is_playing = audio::audio_is_playing(&self.audio).await.unwrap_or(false);
        let is_finished = audio::audio_is_finished(&self.audio).await.unwrap_or(false);
        let position_seconds = audio::audio_get_position(&self.audio).await.unwrap_or(0.0);

        let second = position_seconds.max(0.0).floor() as u64;
        let should_report = {
            let mut current = self.current.lock().expect("playback state poisoned");
            current.as_mut().is_some_and(|current| {
                if is_playing
                    && second.saturating_sub(current.last_reported_second)
                        >= PROGRESS_REPORT_INTERVAL_SECONDS
                {
                    current.last_reported_second = second;
                    true
                } else {
                    false
                }
            })
        };
        if should_report {
            self.report_progress(position_seconds, false).await;
            self.media_controls
                .set_playback_status(true, Some(position_seconds))
                .ok();
        }

        Ok(PlaybackSnapshot {
            is_playing,
            is_finished,
            position_seconds,
        })
    }

    pub fn pop_media_event(&self) -> Option<MediaEvent> {
        self.media_controls.pop_event()
    }

    async fn report_progress(&self, position_seconds: f64, is_paused: bool) {
        let item = self
            .current
            .lock()
            .expect("playback state poisoned")
            .as_ref()
            .map(|current| current.item.clone());
        let Some(item) = item else {
            return;
        };

        aurelia_core::report_playback_progress_event(
            item.server_url,
            item.token,
            item.user_id,
            item.id,
            seconds_to_ticks(position_seconds),
            is_paused,
        )
        .await
        .ok();
    }
}

fn seconds_to_ticks(seconds: f64) -> i64 {
    (seconds.max(0.0) * 10_000_000.0).round() as i64
}

#[cfg(test)]
mod tests {
    use super::seconds_to_ticks;

    #[test]
    fn playback_seconds_convert_to_jellyfin_ticks() {
        assert_eq!(seconds_to_ticks(0.0), 0);
        assert_eq!(seconds_to_ticks(1.5), 15_000_000);
        assert_eq!(seconds_to_ticks(-4.0), 0);
    }
}
