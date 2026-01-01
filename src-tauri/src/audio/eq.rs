//! 5-band parametric equalizer using biquad filters
//!
//! Provides the same frequency bands as the previous WebAudio implementation:
//! - 60 Hz (Low Shelf)
//! - 250 Hz (Peaking)
//! - 1000 Hz (Peaking)
//! - 4000 Hz (Peaking)
//! - 16000 Hz (High Shelf)
//!
//! Uses a lock-free architecture for audio thread performance:
//! - Settings are stored atomically and can be updated from any thread
//! - Filter coefficients are rebuilt only when settings change
//! - Audio processing happens without any mutex locks

use biquad::{Biquad, Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use rodio::Source;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

/// EQ band configuration
#[derive(Debug, Clone, Copy)]
pub struct EQBand {
    pub frequency: f32,
    pub gain_db: f32,
    pub q: f32,
    pub filter_type: EQFilterType,
}

/// Filter type for each EQ band
#[derive(Debug, Clone, Copy)]
pub enum EQFilterType {
    LowShelf,
    Peaking,
    HighShelf,
}

/// Default EQ bands matching the WebAudio implementation
pub const DEFAULT_EQ_BANDS: [EQBand; 5] = [
    EQBand {
        frequency: 60.0,
        gain_db: 0.0,
        q: Q_BUTTERWORTH_F32,
        filter_type: EQFilterType::LowShelf,
    },
    EQBand {
        frequency: 250.0,
        gain_db: 0.0,
        q: Q_BUTTERWORTH_F32,
        filter_type: EQFilterType::Peaking,
    },
    EQBand {
        frequency: 1000.0,
        gain_db: 0.0,
        q: Q_BUTTERWORTH_F32,
        filter_type: EQFilterType::Peaking,
    },
    EQBand {
        frequency: 4000.0,
        gain_db: 0.0,
        q: Q_BUTTERWORTH_F32,
        filter_type: EQFilterType::Peaking,
    },
    EQBand {
        frequency: 16000.0,
        gain_db: 0.0,
        q: Q_BUTTERWORTH_F32,
        filter_type: EQFilterType::HighShelf,
    },
];

/// Atomic float wrapper using bit reinterpretation
struct AtomicF32(AtomicU32);

impl AtomicF32 {
    fn new(val: f32) -> Self {
        Self(AtomicU32::new(val.to_bits()))
    }

    fn load(&self, ordering: Ordering) -> f32 {
        f32::from_bits(self.0.load(ordering))
    }

    fn store(&self, val: f32, ordering: Ordering) {
        self.0.store(val.to_bits(), ordering);
    }
}

/// Lock-free EQ settings that can be shared between threads
///
/// Settings changes increment the generation counter, which signals
/// the audio thread to rebuild its local filter state.
pub struct EQSettings {
    /// Band gains in dB (-20 to +20)
    band_gains: [AtomicF32; 5],
    /// Whether the EQ is enabled
    enabled: AtomicBool,
    /// Current sample rate
    sample_rate: AtomicU32,
    /// Generation counter - incremented on any setting change
    generation: AtomicU64,
}

impl EQSettings {
    /// Create new settings with default flat EQ
    pub fn new(sample_rate: u32) -> Self {
        Self {
            band_gains: [
                AtomicF32::new(0.0),
                AtomicF32::new(0.0),
                AtomicF32::new(0.0),
                AtomicF32::new(0.0),
                AtomicF32::new(0.0),
            ],
            enabled: AtomicBool::new(false),
            sample_rate: AtomicU32::new(sample_rate),
            generation: AtomicU64::new(0),
        }
    }

    /// Set the gain for a specific band (0-4)
    pub fn set_band_gain(&self, band_index: usize, gain_db: f32) {
        if band_index >= 5 {
            return;
        }
        let clamped_gain = gain_db.clamp(-20.0, 20.0);
        self.band_gains[band_index].store(clamped_gain, Ordering::Release);
        self.generation.fetch_add(1, Ordering::Release);
    }

    /// Get the current gain for a band
    pub fn get_band_gain(&self, band_index: usize) -> f32 {
        if band_index >= 5 {
            return 0.0;
        }
        self.band_gains[band_index].load(Ordering::Acquire)
    }

    /// Get all band gains
    pub fn get_all_gains(&self) -> [f32; 5] {
        [
            self.band_gains[0].load(Ordering::Acquire),
            self.band_gains[1].load(Ordering::Acquire),
            self.band_gains[2].load(Ordering::Acquire),
            self.band_gains[3].load(Ordering::Acquire),
            self.band_gains[4].load(Ordering::Acquire),
        ]
    }

    /// Enable or disable the EQ
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
        self.generation.fetch_add(1, Ordering::Release);
    }

    /// Check if EQ is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Reset all bands to flat (0 dB)
    pub fn reset(&self) {
        for band in &self.band_gains {
            band.store(0.0, Ordering::Release);
        }
        self.generation.fetch_add(1, Ordering::Release);
    }

    /// Update sample rate
    pub fn update_sample_rate(&self, sample_rate: u32) {
        let current = self.sample_rate.load(Ordering::Acquire);
        if current != sample_rate {
            self.sample_rate.store(sample_rate, Ordering::Release);
            self.generation.fetch_add(1, Ordering::Release);
        }
    }

    /// Get current generation (for change detection)
    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Get current sample rate
    fn sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Acquire)
    }
}

/// Local EQ processor state for the audio thread
///
/// This is NOT shared between threads - each EQSource has its own copy.
/// It syncs with the shared EQSettings only when the generation changes.
struct LocalEQProcessor {
    bands: [EQBand; 5],
    filters_left: [DirectForm1<f32>; 5],
    filters_right: [DirectForm1<f32>; 5],
    sample_rate: u32,
    enabled: bool,
    /// Last seen generation from EQSettings
    last_generation: u64,
}

impl LocalEQProcessor {
    /// Create a new local processor synced with settings
    fn new(settings: &EQSettings) -> Self {
        let sample_rate = settings.sample_rate();
        let gains = settings.get_all_gains();
        let enabled = settings.is_enabled();

        let mut bands = DEFAULT_EQ_BANDS;
        for (i, gain) in gains.iter().enumerate() {
            bands[i].gain_db = *gain;
        }

        let filters_left = Self::create_filters(&bands, sample_rate);
        let filters_right = Self::create_filters(&bands, sample_rate);

        Self {
            bands,
            filters_left,
            filters_right,
            sample_rate,
            enabled,
            last_generation: settings.generation(),
        }
    }

    fn create_filters(bands: &[EQBand; 5], sample_rate: u32) -> [DirectForm1<f32>; 5] {
        bands.map(|band| {
            let coeffs = Self::calculate_coefficients(&band, sample_rate);
            DirectForm1::<f32>::new(coeffs)
        })
    }

    fn calculate_coefficients(band: &EQBand, sample_rate: u32) -> Coefficients<f32> {
        let fs = sample_rate.hz();
        let f0 = band.frequency.hz();
        let db_gain = band.gain_db;

        match band.filter_type {
            EQFilterType::LowShelf => {
                Coefficients::<f32>::from_params(Type::LowShelf(db_gain), fs, f0, band.q)
                    .unwrap_or_else(|_| {
                        Coefficients::<f32>::from_params(Type::SinglePoleLowPass, fs, f0, band.q)
                            .unwrap()
                    })
            }
            EQFilterType::Peaking => {
                Coefficients::<f32>::from_params(Type::PeakingEQ(db_gain), fs, f0, band.q)
                    .unwrap_or_else(|_| {
                        Coefficients::<f32>::from_params(Type::AllPass, fs, f0, band.q).unwrap()
                    })
            }
            EQFilterType::HighShelf => {
                Coefficients::<f32>::from_params(Type::HighShelf(db_gain), fs, f0, band.q)
                    .unwrap_or_else(|_| {
                        Coefficients::<f32>::from_params(Type::SinglePoleLowPass, fs, f0, band.q)
                            .unwrap()
                    })
            }
        }
    }

    /// Sync with shared settings if they've changed
    ///
    /// Returns true if settings were updated
    fn sync_if_needed(&mut self, settings: &EQSettings) -> bool {
        let current_gen = settings.generation();
        if current_gen == self.last_generation {
            return false;
        }

        // Settings have changed - update local state
        let new_sample_rate = settings.sample_rate();
        let gains = settings.get_all_gains();
        self.enabled = settings.is_enabled();

        let sample_rate_changed = new_sample_rate != self.sample_rate;
        self.sample_rate = new_sample_rate;

        // Update band gains and rebuild filters
        for (i, gain) in gains.iter().enumerate() {
            self.bands[i].gain_db = *gain;
        }

        if sample_rate_changed {
            // Full rebuild with new sample rate
            self.filters_left = Self::create_filters(&self.bands, self.sample_rate);
            self.filters_right = Self::create_filters(&self.bands, self.sample_rate);
        } else {
            // Just update coefficients
            for i in 0..5 {
                let coeffs = Self::calculate_coefficients(&self.bands[i], self.sample_rate);
                self.filters_left[i] = DirectForm1::<f32>::new(coeffs);
                self.filters_right[i] = DirectForm1::<f32>::new(coeffs);
            }
        }

        self.last_generation = current_gen;
        true
    }

    /// Process a stereo sample pair through the EQ
    ///
    /// This is the hot path - no locks, no allocations
    #[inline]
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        let mut l = left;
        let mut r = right;

        for i in 0..5 {
            l = self.filters_left[i].run(l);
            r = self.filters_right[i].run(r);
        }

        (l, r)
    }
}

/// A source wrapper that applies EQ processing (lock-free)
///
/// Uses a local processor that syncs with shared settings only when they change.
/// Audio processing happens entirely without locks.
pub struct EQSource<S>
where
    S: Source<Item = f32>,
{
    source: S,
    settings: Arc<EQSettings>,
    processor: LocalEQProcessor,
    pending_left: Option<f32>,
    /// Counter for periodic settings check (not every sample)
    samples_since_sync: u32,
}

/// How often to check for settings changes (in sample pairs)
/// At 44.1kHz, 256 pairs = ~5.8ms latency for setting changes
const SYNC_INTERVAL: u32 = 256;

impl<S> EQSource<S>
where
    S: Source<Item = f32>,
{
    pub fn new(source: S, settings: Arc<EQSettings>) -> Self {
        let processor = LocalEQProcessor::new(&settings);
        Self {
            source,
            settings,
            processor,
            pending_left: None,
            samples_since_sync: 0,
        }
    }
}

impl<S> Iterator for EQSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Handle stereo samples in pairs - return pending right channel
        if let Some(right_processed) = self.pending_left.take() {
            return Some(right_processed);
        }

        // Periodically sync with shared settings (not every sample!)
        self.samples_since_sync += 1;
        if self.samples_since_sync >= SYNC_INTERVAL {
            self.samples_since_sync = 0;
            self.processor.sync_if_needed(&self.settings);
        }

        // Get next stereo pair from source
        let left = self.source.next()?;
        let right = self.source.next().unwrap_or(left);

        // Process through EQ (no locks!)
        let (l, r) = self.processor.process_stereo(left, right);

        self.pending_left = Some(r);
        Some(l)
    }
}

impl<S> Source for EQSource<S>
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
        // Clear any pending sample when seeking
        self.pending_left = None;
        // Force a settings sync after seek
        self.samples_since_sync = SYNC_INTERVAL;
        // Forward the seek to the underlying source
        self.source.try_seek(pos)
    }
}
