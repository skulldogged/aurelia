//! Audio spectrum analyzer for visualization
//!
//! Provides FFT-based frequency analysis for audio visualization.
//! Uses a lock-free architecture to avoid blocking the audio thread:
//! - Audio thread writes samples to a ring buffer
//! - Event loop reads samples and computes FFT periodically
//! - Results are emitted as Tauri events

use rodio::Source;
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

/// FFT size - must be power of 2. 256 matches Web Audio API default
pub const FFT_SIZE: usize = 256;

/// Number of frequency bins (FFT_SIZE / 2)
pub const FREQUENCY_BIN_COUNT: usize = FFT_SIZE / 2;

/// Lock-free ring buffer for audio samples
///
/// Single producer (audio thread), single consumer (analyzer thread)
pub struct AnalyzerBuffer {
    /// Sample buffer (mono, mixed from stereo)
    buffer: Box<[std::sync::atomic::AtomicU32; FFT_SIZE]>,
    /// Write position (audio thread)
    write_pos: AtomicUsize,
    /// Whether analyzer is enabled
    enabled: AtomicBool,
}

impl AnalyzerBuffer {
    pub fn new() -> Self {
        // Initialize with zeros using array initialization
        let buffer: Box<[std::sync::atomic::AtomicU32; FFT_SIZE]> = {
            let mut vec = Vec::with_capacity(FFT_SIZE);
            for _ in 0..FFT_SIZE {
                vec.push(std::sync::atomic::AtomicU32::new(0f32.to_bits()));
            }
            vec.into_boxed_slice().try_into().unwrap()
        };

        Self {
            buffer,
            write_pos: AtomicUsize::new(0),
            enabled: AtomicBool::new(false),
        }
    }

    /// Write a sample to the buffer (called from audio thread)
    #[inline]
    pub fn write_sample(&self, sample: f32) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) % FFT_SIZE;
        self.buffer[pos].store(sample.to_bits(), Ordering::Relaxed);
    }

    /// Read all samples into a buffer for FFT processing
    /// Returns samples in order (oldest to newest)
    pub fn read_samples(&self) -> [f32; FFT_SIZE] {
        let mut samples = [0.0f32; FFT_SIZE];
        let write_pos = self.write_pos.load(Ordering::Acquire);

        for i in 0..FFT_SIZE {
            let read_pos = (write_pos + i) % FFT_SIZE;
            let bits = self.buffer[read_pos].load(Ordering::Relaxed);
            samples[i] = f32::from_bits(bits);
        }

        samples
    }

    /// Enable or disable the analyzer
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }

    /// Check if analyzer is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
}

impl Default for AnalyzerBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// FFT processor for computing frequency spectrum
pub struct SpectrumAnalyzer {
    planner: FftPlanner<f32>,
    /// Hann window for smoother frequency response
    window: [f32; FFT_SIZE],
    /// Smoothing factor for temporal smoothing (0-1, higher = more smoothing)
    smoothing: f32,
    /// Previous spectrum for smoothing
    prev_spectrum: [f32; FREQUENCY_BIN_COUNT],
}

impl SpectrumAnalyzer {
    pub fn new() -> Self {
        // Precompute Hann window
        let mut window = [0.0f32; FFT_SIZE];
        for (i, w) in window.iter_mut().enumerate() {
            let t = i as f32 / FFT_SIZE as f32;
            *w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * t).cos());
        }

        Self {
            planner: FftPlanner::new(),
            window,
            smoothing: 0.8, // Similar to Web Audio API default
            prev_spectrum: [0.0; FREQUENCY_BIN_COUNT],
        }
    }

    /// Compute frequency spectrum from time-domain samples
    ///
    /// Returns frequency magnitudes as bytes (0-255) for efficient transfer
    pub fn compute_spectrum(&mut self, samples: &[f32; FFT_SIZE]) -> [u8; FREQUENCY_BIN_COUNT] {
        let fft = self.planner.plan_fft_forward(FFT_SIZE);

        // Apply window and convert to complex
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .zip(self.window.iter())
            .map(|(s, w)| Complex::new(s * w, 0.0))
            .collect();

        // Perform FFT
        fft.process(&mut buffer);

        // Convert to magnitude spectrum (only first half is meaningful)
        let mut spectrum = [0u8; FREQUENCY_BIN_COUNT];

        for i in 0..FREQUENCY_BIN_COUNT {
            // Compute magnitude
            let magnitude = buffer[i].norm();

            // Convert to dB scale (with floor to avoid -inf)
            // Normalize to 0-255 range like Web Audio API's getByteFrequencyData
            let db = 20.0 * (magnitude / FFT_SIZE as f32).max(1e-10).log10();

            // Map dB to 0-255 (Web Audio uses -100dB to 0dB range approximately)
            // Adjust range for better visual results
            let min_db = -100.0f32;
            let max_db = 0.0f32;
            let normalized = ((db - min_db) / (max_db - min_db)).clamp(0.0, 1.0);

            // Apply temporal smoothing
            let smoothed =
                self.smoothing * self.prev_spectrum[i] + (1.0 - self.smoothing) * normalized;
            self.prev_spectrum[i] = smoothed;

            spectrum[i] = (smoothed * 255.0) as u8;
        }

        spectrum
    }

    /// Compute time-domain waveform data
    ///
    /// Returns waveform as bytes (0-255, centered at 128) for wave visualization
    pub fn compute_waveform(&self, samples: &[f32; FFT_SIZE]) -> [u8; FFT_SIZE] {
        let mut waveform = [128u8; FFT_SIZE];

        for (i, sample) in samples.iter().enumerate() {
            // Convert from -1.0..1.0 to 0..255 centered at 128
            let normalized = (sample.clamp(-1.0, 1.0) + 1.0) / 2.0;
            waveform[i] = (normalized * 255.0) as u8;
        }

        waveform
    }

    /// Set smoothing factor (0-1)
    pub fn set_smoothing(&mut self, smoothing: f32) {
        self.smoothing = smoothing.clamp(0.0, 1.0);
    }
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Source wrapper that taps audio samples for visualization
///
/// Writes samples to a shared buffer without blocking the audio thread.
pub struct AnalyzerSource<S>
where
    S: Source<Item = f32>,
{
    source: S,
    buffer: Arc<AnalyzerBuffer>,
    pending_right: Option<f32>,
}

impl<S> AnalyzerSource<S>
where
    S: Source<Item = f32>,
{
    pub fn new(source: S, buffer: Arc<AnalyzerBuffer>) -> Self {
        Self {
            source,
            buffer,
            pending_right: None,
        }
    }
}

impl<S> Iterator for AnalyzerSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Return pending right channel
        if let Some(right) = self.pending_right.take() {
            return Some(right);
        }

        // Get stereo pair
        let left = self.source.next()?;
        let right = self.source.next().unwrap_or(left);

        // Mix to mono and write to analyzer buffer
        let mono = (left + right) * 0.5;
        self.buffer.write_sample(mono);

        self.pending_right = Some(right);
        Some(left)
    }
}

impl<S> Source for AnalyzerSource<S>
where
    S: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.pending_right = None;
        self.source.try_seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_buffer() {
        let buffer = AnalyzerBuffer::new();
        buffer.set_enabled(true);

        // Write some samples
        for i in 0..FFT_SIZE {
            buffer.write_sample(i as f32 / FFT_SIZE as f32);
        }

        let samples = buffer.read_samples();
        assert_eq!(samples.len(), FFT_SIZE);
    }

    #[test]
    fn test_spectrum_analyzer() {
        let mut analyzer = SpectrumAnalyzer::new();

        // Generate a simple sine wave
        let mut samples = [0.0f32; FFT_SIZE];
        for (i, s) in samples.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 4.0 * i as f32 / FFT_SIZE as f32).sin();
        }

        let spectrum = analyzer.compute_spectrum(&samples);
        assert_eq!(spectrum.len(), FREQUENCY_BIN_COUNT);
    }
}
