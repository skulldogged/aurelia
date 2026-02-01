//! Audio player with streaming, EQ, and gapless playback
//!
//! Core audio playback engine using Rodio for output and
//! stream-download for HTTP streaming.

use crate::audio::analyzer::{AnalyzerBuffer, AnalyzerSource};
use crate::audio::eq::{EQSettings, EQSource};
use crate::audio::streaming::StreamingSource;
use anyhow::{Context, Result};
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread;
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

/// Command sent to the audio thread
enum AudioThreadCommand {
    CreateSink(Sender<Sink>),
}

/// Main audio player for the application
pub struct AudioPlayer {
    // Command channel to the audio thread
    cmd_tx: Sender<AudioThreadCommand>,
    sink: Option<Sink>,
    next_source: Option<PreparedSource>,
    /// Lock-free EQ settings shared with audio processing thread
    eq_settings: Arc<EQSettings>,
    /// Lock-free analyzer buffer for visualization
    analyzer_buffer: Arc<AnalyzerBuffer>,
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
        let (cmd_tx, cmd_rx) = channel::<AudioThreadCommand>();
        let (init_tx, init_rx) = channel::<Result<()>>();

        // Spawn a thread to own the stream (which is !Send on macOS)
        thread::spawn(move || {
            // Create output stream with default device using builder pattern
            let stream_result = OutputStreamBuilder::open_default_stream()
                .context("Failed to open audio output device");

            let stream = match stream_result {
                Ok(s) => {
                    let _ = init_tx.send(Ok(()));
                    s
                }
                Err(e) => {
                    let _ = init_tx.send(Err(e));
                    return;
                }
            };

            // Loop to handle commands and keep stream alive
            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    AudioThreadCommand::CreateSink(reply_tx) => {
                        // Create a sink connected to the stream's mixer
                        let sink = Sink::connect_new(&stream.mixer());
                        let _ = reply_tx.send(sink);
                    }
                }
            }
        });

        // Wait for initialization
        init_rx
            .recv()
            .context("Failed to receive initialization response")??;

        // Create lock-free EQ settings
        let eq_settings = Arc::new(EQSettings::new(44100));

        // Create lock-free analyzer buffer
        let analyzer_buffer = Arc::new(AnalyzerBuffer::new());

        info!("Audio player initialized with default output device");

        Ok(Self {
            cmd_tx,
            sink: None,
            next_source: None,
            eq_settings,
            analyzer_buffer,
            volume: 1.0,
            is_playing: Arc::new(AtomicBool::new(false)),
            current_url: None,
            current_track: None,
        })
    }

    /// Get a reference to the analyzer buffer for the event loop
    pub fn analyzer_buffer(&self) -> Arc<AnalyzerBuffer> {
        Arc::clone(&self.analyzer_buffer)
    }

    /// Enable or disable the spectrum analyzer
    pub fn set_analyzer_enabled(&self, enabled: bool) {
        self.analyzer_buffer.set_enabled(enabled);
        debug!("Spectrum analyzer enabled: {}", enabled);
    }

    /// Check if spectrum analyzer is enabled
    pub fn is_analyzer_enabled(&self) -> bool {
        self.analyzer_buffer.is_enabled()
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

        // Update EQ sample rate (lock-free, preserves band gains and enabled state)
        let sample_rate = decoder.sample_rate();
        self.eq_settings.update_sample_rate(sample_rate);

        // Apply EQ to the source (lock-free processing)
        let eq_source = EQSource::new(decoder, Arc::clone(&self.eq_settings));

        // Wrap with analyzer for visualization (lock-free sample capture)
        let analyzer_source = AnalyzerSource::new(eq_source, Arc::clone(&self.analyzer_buffer));

        // Create a sink via the background thread
        let (sink_tx, sink_rx) = channel();
        self.cmd_tx
            .send(AudioThreadCommand::CreateSink(sink_tx))
            .context("Failed to request audio sink")?;

        let sink = sink_rx.recv().context("Failed to receive audio sink")?;

        // Volume is controlled via sink.set_volume() only (not amplify) to avoid
        // multiplicative volume issues when tracks are reloaded
        sink.set_volume(self.volume);
        sink.append(analyzer_source);
        sink.play();

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
                    warn!("Native seek failed ({}), falling back to stream restart", e);
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

        info!("Restarting stream from position {} seconds", position_secs);

        // Restart playback from the target position
        self.play_url_from_position(&track.url, &track.token, Some(position_secs))
            .await?;

        debug!("Stream restart seek to {} seconds completed", position_secs);
        Ok(())
    }

    /// Set EQ band gain (lock-free)
    pub fn set_eq_band(&self, band: usize, gain_db: f32) -> Result<()> {
        self.eq_settings.set_band_gain(band, gain_db);
        debug!("EQ band {} set to {} dB", band, gain_db);
        Ok(())
    }

    /// Get EQ band gain (lock-free)
    pub fn get_eq_band(&self, band: usize) -> f32 {
        self.eq_settings.get_band_gain(band)
    }

    /// Get all EQ band gains (lock-free)
    pub fn get_all_eq_bands(&self) -> [f32; 5] {
        self.eq_settings.get_all_gains()
    }

    /// Enable or disable EQ (lock-free)
    pub fn set_eq_enabled(&self, enabled: bool) -> Result<()> {
        self.eq_settings.set_enabled(enabled);
        debug!("EQ enabled: {}", enabled);
        Ok(())
    }

    /// Check if EQ is enabled (lock-free)
    pub fn is_eq_enabled(&self) -> bool {
        self.eq_settings.is_enabled()
    }

    /// Reset EQ to flat (lock-free)
    pub fn reset_eq(&self) -> Result<()> {
        self.eq_settings.reset();
        debug!("EQ reset to flat");
        Ok(())
    }

    /// Reinitialize the audio output stream
    ///
    /// This recreates the background thread and output stream while preserving
    /// EQ settings and analyzer buffer. Use this after the audio device changes
    /// (e.g., headphones disconnected on Android).
    pub fn reinit(&mut self) -> Result<()> {
        info!("Reinitializing audio output stream");

        // Stop any current playback
        self.stop();

        // Create new command channel (dropping old cmd_tx kills the old thread)
        let (cmd_tx, cmd_rx) = channel::<AudioThreadCommand>();
        let (init_tx, init_rx) = channel::<Result<()>>();

        // Spawn a new thread to own the stream
        thread::spawn(move || {
            let stream_result = OutputStreamBuilder::open_default_stream()
                .context("Failed to open audio output device");

            let stream = match stream_result {
                Ok(s) => {
                    let _ = init_tx.send(Ok(()));
                    s
                }
                Err(e) => {
                    let _ = init_tx.send(Err(e));
                    return;
                }
            };

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    AudioThreadCommand::CreateSink(reply_tx) => {
                        let sink = Sink::connect_new(&stream.mixer());
                        let _ = reply_tx.send(sink);
                    }
                }
            }
        });

        // Wait for initialization
        init_rx
            .recv()
            .context("Failed to receive initialization response")??;

        // Update command channel (preserving eq_settings and analyzer_buffer)
        self.cmd_tx = cmd_tx;
        self.sink = None;
        self.next_source = None;
        self.is_playing.store(false, Ordering::SeqCst);
        self.current_url = None;
        self.current_track = None;

        info!("Audio output stream reinitialized");
        Ok(())
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create default audio player")
    }
}
