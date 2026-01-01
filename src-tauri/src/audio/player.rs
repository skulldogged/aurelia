//! Audio player with streaming, EQ, and gapless playback
//!
//! Core audio playback engine using Rodio for output and
//! stream-download for HTTP streaming.

use crate::audio::eq::{EQSource, ParametricEQ};
use crate::audio::streaming::StreamingSource;
use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, trace, warn};

/// Prepared audio source ready for gapless transition
struct PreparedSource {
    url: String,
    token: String,
}

/// Information needed to restart a stream from a specific position
#[derive(Clone)]
struct CurrentTrack {
    url: String,
    token: String,
}

/// Main audio player for the application
pub struct AudioPlayer {
    // The stream must be kept alive for playback to work
    _stream: OutputStream,
    sink: Option<Sink>,
    next_source: Option<PreparedSource>,
    eq: Arc<Mutex<ParametricEQ>>,
    volume: f32,
    // Playback state
    is_playing: Arc<AtomicBool>,
    current_url: Option<String>,
    // Track info for seek restart
    current_track: Option<CurrentTrack>,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Result<Self> {
        // Create output stream with default device using builder pattern
        let stream = OutputStreamBuilder::open_default_stream()
            .context("Failed to open audio output device")?;

        let eq = Arc::new(Mutex::new(ParametricEQ::new(44100)));

        info!("Audio player initialized with default output device");

        Ok(Self {
            _stream: stream,
            sink: None,
            next_source: None,
            eq,
            volume: 1.0,
            is_playing: Arc::new(AtomicBool::new(false)),
            current_url: None,
            current_track: None,
        })
    }

    /// Play audio from a URL
    pub async fn play_url(&mut self, url: &str, token: &str) -> Result<()> {
        self.play_url_from_position(url, token, None).await
    }

    /// Play audio from a URL starting at a specific position
    ///
    /// # Arguments
    /// * `url` - The audio stream URL
    /// * `token` - Jellyfin authentication token
    /// * `start_position_secs` - Optional start position in seconds
    pub async fn play_url_from_position(
        &mut self,
        url: &str,
        token: &str,
        start_position_secs: Option<f64>,
    ) -> Result<()> {
        info!(
            "Playing audio from URL: {} (start: {:?}s)",
            url, start_position_secs
        );

        // Stop any current playback
        self.stop();

        // Convert seconds to Jellyfin ticks (10,000 ticks = 1ms, so 10,000,000 ticks = 1s)
        let start_time_ticks = start_position_secs.map(|secs| (secs * 10_000_000.0) as u64);

        // Create streaming source with optional start time
        let streaming = StreamingSource::with_start_time(url, token, start_time_ticks)
            .await
            .context("Failed to create streaming source")?;

        let content_length = streaming.content_length();
        debug!("Stream content length: {:?}", content_length);

        // Create decoder using builder for better seeking support
        let reader = BufReader::new(streaming);
        let mut decoder_builder = Decoder::builder().with_data(reader);

        // If we know the content length, set it to enable backward seeking
        if let Some(len) = content_length {
            trace!("Setting byte_len for decoder: {} bytes", len);
            decoder_builder = decoder_builder.with_byte_len(len);
        }

        let decoder = decoder_builder
            .build()
            .context("Failed to decode audio stream")?;

        // Get sample rate for EQ
        let sample_rate = decoder.sample_rate();
        if let Ok(mut eq) = self.eq.lock() {
            *eq = ParametricEQ::new(sample_rate);
        }

        // Apply EQ to the source
        let eq_source = EQSource::new(decoder, Arc::clone(&self.eq));

        // Apply volume amplification
        let volume_source = eq_source.amplify(self.volume);

        // Create a sink connected to the stream's mixer
        let sink = Sink::connect_new(&self._stream.mixer());
        sink.append(volume_source);

        self.sink = Some(sink);
        self.current_url = Some(url.to_string());
        self.current_track = Some(CurrentTrack {
            url: url.to_string(),
            token: token.to_string(),
        });
        self.is_playing.store(true, Ordering::SeqCst);

        info!("Audio playback started");
        Ok(())
    }

    /// Prepare the next track for gapless transition
    pub fn prepare_next(&mut self, url: &str, token: &str) {
        debug!("Preparing next track: {}", url);
        self.next_source = Some(PreparedSource {
            url: url.to_string(),
            token: token.to_string(),
        });
    }

    /// Advance to the prepared next track (gapless)
    pub async fn advance_to_next(&mut self) -> Result<()> {
        let next = self.next_source.take().context("No next track prepared")?;

        info!("Advancing to next track (gapless): {}", next.url);
        self.play_url(&next.url, &next.token).await
    }

    /// Pause playback
    pub fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.is_playing.store(false, Ordering::SeqCst);
            debug!("Playback paused");
        }
    }

    /// Resume playback
    pub fn resume(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
            self.is_playing.store(true, Ordering::SeqCst);
            debug!("Playback resumed");
        }
    }

    /// Stop playback completely
    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.is_playing.store(false, Ordering::SeqCst);
        self.current_url = None;
        self.current_track = None;
        debug!("Playback stopped");
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.volume = clamped;
        if let Some(sink) = &self.sink {
            sink.set_volume(clamped);
        }
        debug!("Volume set to {}", clamped);
    }

    /// Get current volume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    /// Check if playback is finished
    pub fn is_finished(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.empty()
        } else {
            true
        }
    }

    /// Get current playback position in seconds
    pub fn get_position(&self) -> f64 {
        if let Some(sink) = &self.sink {
            sink.get_pos().as_secs_f64()
        } else {
            0.0
        }
    }

    /// Seek to a position in seconds (synchronous, may fail for backward seeks)
    /// Note: Rodio's try_seek may not work for all source types
    pub fn seek(&self, position_secs: f64) -> Result<()> {
        if let Some(sink) = &self.sink {
            let duration = std::time::Duration::from_secs_f64(position_secs);
            sink.try_seek(duration)
                .map_err(|e| anyhow::anyhow!("Failed to seek: {:?}", e))?;
            debug!("Seeked to {} seconds", position_secs);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No audio playing"))
        }
    }

    /// Seek to a position in seconds with fallback to stream restart
    ///
    /// First attempts a native seek. If that fails (common for backward seeks
    /// in streaming audio), falls back to restarting the stream from the
    /// target position using Jellyfin's startTimeTicks parameter.
    pub async fn seek_with_fallback(&mut self, position_secs: f64) -> Result<()> {
        // First, try native seek
        if let Some(sink) = &self.sink {
            let duration = std::time::Duration::from_secs_f64(position_secs);
            match sink.try_seek(duration) {
                Ok(()) => {
                    debug!("Native seek to {} seconds succeeded", position_secs);
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Native seek failed ({}), falling back to stream restart",
                        e
                    );
                }
            }
        } else {
            return Err(anyhow::anyhow!("No audio playing"));
        }

        // Native seek failed, try to restart the stream from the target position
        let track = self
            .current_track
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No track info available for seek restart"))?;

        info!(
            "Restarting stream from position {} seconds",
            position_secs
        );

        // Restart playback from the target position
        self.play_url_from_position(&track.url, &track.token, Some(position_secs))
            .await?;

        debug!(
            "Stream restart seek to {} seconds completed",
            position_secs
        );
        Ok(())
    }

    /// Check if we have track info available for seek restart
    pub fn can_seek_restart(&self) -> bool {
        self.current_track.is_some()
    }

    /// Set EQ band gain
    pub fn set_eq_band(&self, band: usize, gain_db: f32) -> Result<()> {
        let mut eq = self
            .eq
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock EQ"))?;
        eq.set_band_gain(band, gain_db);
        debug!("EQ band {} set to {} dB", band, gain_db);
        Ok(())
    }

    /// Get EQ band gain
    pub fn get_eq_band(&self, band: usize) -> f32 {
        if let Ok(eq) = self.eq.lock() {
            eq.get_band_gain(band)
        } else {
            0.0
        }
    }

    /// Get all EQ band gains
    pub fn get_all_eq_bands(&self) -> [f32; 5] {
        if let Ok(eq) = self.eq.lock() {
            eq.get_all_gains()
        } else {
            [0.0; 5]
        }
    }

    /// Enable or disable EQ
    pub fn set_eq_enabled(&self, enabled: bool) -> Result<()> {
        let mut eq = self
            .eq
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock EQ"))?;
        eq.set_enabled(enabled);
        debug!("EQ enabled: {}", enabled);
        Ok(())
    }

    /// Check if EQ is enabled
    pub fn is_eq_enabled(&self) -> bool {
        if let Ok(eq) = self.eq.lock() {
            eq.is_enabled()
        } else {
            false
        }
    }

    /// Reset EQ to flat
    pub fn reset_eq(&self) -> Result<()> {
        let mut eq = self
            .eq
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock EQ"))?;
        eq.reset();
        debug!("EQ reset to flat");
        Ok(())
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create default audio player")
    }
}
