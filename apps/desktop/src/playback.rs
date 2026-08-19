use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

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
    pub did_auto_advance: bool,
    pub auto_advanced_token: Option<u64>,
}

#[derive(Clone)]
pub struct PlaybackController {
    audio: Arc<AudioState>,
    media_controls: Arc<MediaControlsState>,
    current: Arc<Mutex<Option<CurrentPlayback>>>,
    prepared: Arc<Mutex<Option<PreparedPlayback>>>,
    prepare_generation: Arc<AtomicU64>,
    prepare_lock: Arc<tokio::sync::Mutex<()>>,
    seek_generation: Arc<AtomicU64>,
    seek_lock: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Clone, Debug)]
struct CurrentPlayback {
    item: PlaybackItem,
    last_reported_second: u64,
}

#[derive(Clone, Debug)]
struct PreparedPlayback {
    token: u64,
    item: PlaybackItem,
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
            prepared: Arc::new(Mutex::new(None)),
            prepare_generation: Arc::new(AtomicU64::new(0)),
            prepare_lock: Arc::new(tokio::sync::Mutex::new(())),
            seek_generation: Arc::new(AtomicU64::new(0)),
            seek_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Invalidate any in-flight preparation and return the generation for its replacement.
    pub fn begin_prepare(&self) -> u64 {
        *self.prepared.lock().expect("playback state poisoned") = None;
        self.prepare_generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub async fn prepare_next(
        &self,
        prepared: Option<(u64, PlaybackItem)>,
        generation: u64,
    ) -> Result<()> {
        let _guard = self.prepare_lock.lock().await;
        if self.prepare_generation.load(Ordering::SeqCst) != generation {
            return Ok(());
        }

        let Some((token, item)) = prepared else {
            audio::audio_clear_prepared_next(&self.audio).await?;
            return Ok(());
        };
        let stream_url = aurelia_core::build_desktop_stream_url(
            item.server_url.clone(),
            item.token.clone(),
            item.id.clone(),
            item.container.clone(),
        );
        audio::audio_prepare_next(&self.audio, stream_url, item.token.clone()).await?;

        if self.prepare_generation.load(Ordering::SeqCst) != generation {
            audio::audio_clear_prepared_next(&self.audio).await?;
            return Ok(());
        }
        *self.prepared.lock().expect("playback state poisoned") =
            Some(PreparedPlayback { token, item });
        Ok(())
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

    pub fn begin_seek(&self) -> u64 {
        self.seek_generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub async fn seek(&self, position_seconds: f64, generation: u64) -> Result<()> {
        let _guard = self.seek_lock.lock().await;
        if self.seek_generation.load(Ordering::SeqCst) != generation {
            return Ok(());
        }
        audio::audio_seek(&self.audio, position_seconds).await?;
        if self.seek_generation.load(Ordering::SeqCst) != generation {
            return Ok(());
        }
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
        self.begin_prepare();
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
        let tick = audio::audio_poll_position(&self.audio).await;
        let (is_playing, is_finished, position_seconds, did_auto_advance) = match tick {
            Some(tick) => (
                tick.is_playing,
                tick.is_finished,
                tick.position,
                tick.did_auto_advance,
            ),
            None => (
                audio::audio_is_playing(&self.audio).await.unwrap_or(false),
                audio::audio_is_finished(&self.audio).await.unwrap_or(false),
                audio::audio_get_position(&self.audio).await.unwrap_or(0.0),
                false,
            ),
        };
        let auto_advanced_token = if did_auto_advance {
            self.finish_gapless_transition()
        } else {
            None
        };

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
            did_auto_advance,
            auto_advanced_token,
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

    fn finish_gapless_transition(&self) -> Option<u64> {
        let prepared = self
            .prepared
            .lock()
            .expect("playback state poisoned")
            .take()?;
        let previous = self
            .current
            .lock()
            .expect("playback state poisoned")
            .replace(CurrentPlayback {
                item: prepared.item.clone(),
                last_reported_second: 0,
            });

        let started_item = prepared.item.clone();
        tokio::spawn(async move {
            if let Some(previous) = previous {
                aurelia_core::report_playback_stop_event(
                    previous.item.server_url,
                    previous.item.token,
                    previous.item.user_id,
                    previous.item.id,
                    seconds_to_ticks(f64::from(previous.item.duration_seconds)),
                )
                .await
                .ok();
            }
            aurelia_core::report_playback_start_event(
                started_item.server_url,
                started_item.token,
                started_item.user_id,
                started_item.id,
                Some(0),
            )
            .await
            .ok();
        });
        let cover_url = prepared.item.album_id.as_ref().and_then(|album_id| {
            aurelia_core::build_image_url(
                prepared.item.server_url.clone(),
                prepared.item.token.clone(),
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
                title: prepared.item.title,
                artist: Some(prepared.item.artist),
                album: Some(prepared.item.album),
                duration: Some(f64::from(prepared.item.duration_seconds)),
                cover_url,
            })
            .ok();
        self.media_controls
            .set_playback_status(true, Some(0.0))
            .ok();
        Some(prepared.token)
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
