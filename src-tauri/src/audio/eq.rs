//! 5-band parametric equalizer using biquad filters
//!
//! Provides the same frequency bands as the previous WebAudio implementation:
//! - 60 Hz (Low Shelf)
//! - 250 Hz (Peaking)
//! - 1000 Hz (Peaking)
//! - 4000 Hz (Peaking)
//! - 16000 Hz (High Shelf)

use biquad::{Biquad, Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use rodio::Source;
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

/// 5-band parametric equalizer
pub struct ParametricEQ {
    bands: [EQBand; 5],
    filters_left: [DirectForm1<f32>; 5],
    filters_right: [DirectForm1<f32>; 5],
    sample_rate: u32,
    enabled: bool,
}

impl ParametricEQ {
    /// Create a new EQ with default flat settings
    pub fn new(sample_rate: u32) -> Self {
        let bands = DEFAULT_EQ_BANDS;
        let filters_left = Self::create_filters(&bands, sample_rate);
        let filters_right = Self::create_filters(&bands, sample_rate);

        Self {
            bands,
            filters_left,
            filters_right,
            sample_rate,
            enabled: false,
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

        // Convert dB gain to linear
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

    /// Set the gain for a specific band (0-4)
    pub fn set_band_gain(&mut self, band_index: usize, gain_db: f32) {
        if band_index >= 5 {
            return;
        }

        let clamped_gain = gain_db.clamp(-20.0, 20.0);
        self.bands[band_index].gain_db = clamped_gain;

        // Recalculate filter coefficients
        let coeffs = Self::calculate_coefficients(&self.bands[band_index], self.sample_rate);
        self.filters_left[band_index] = DirectForm1::<f32>::new(coeffs);
        self.filters_right[band_index] = DirectForm1::<f32>::new(coeffs);
    }

    /// Get the current gain for a band
    pub fn get_band_gain(&self, band_index: usize) -> f32 {
        if band_index >= 5 {
            return 0.0;
        }
        self.bands[band_index].gain_db
    }

    /// Get all band gains
    pub fn get_all_gains(&self) -> [f32; 5] {
        self.bands.map(|b| b.gain_db)
    }

    /// Enable or disable the EQ
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if EQ is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Reset all bands to flat (0 dB)
    pub fn reset(&mut self) {
        for i in 0..5 {
            self.set_band_gain(i, 0.0);
        }
    }

    /// Process a stereo sample pair through the EQ
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
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

/// A source wrapper that applies EQ processing
pub struct EQSource<S>
where
    S: Source<Item = f32>,
{
    source: S,
    eq: std::sync::Arc<std::sync::Mutex<ParametricEQ>>,
    pending_left: Option<f32>,
}

impl<S> EQSource<S>
where
    S: Source<Item = f32>,
{
    pub fn new(source: S, eq: std::sync::Arc<std::sync::Mutex<ParametricEQ>>) -> Self {
        Self {
            source,
            eq,
            pending_left: None,
        }
    }
}

impl<S> Iterator for EQSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Handle stereo samples in pairs
        if let Some(right_processed) = self.pending_left.take() {
            return Some(right_processed);
        }

        let left = self.source.next()?;
        let right = self.source.next().unwrap_or(left);

        let (l, r) = if let Ok(mut eq) = self.eq.lock() {
            eq.process_stereo(left, right)
        } else {
            (left, right)
        };

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
        // Forward the seek to the underlying source
        self.source.try_seek(pos)
    }
}
