//! HTTP streaming audio source using stream-download
//!
//! Provides a Read + Seek wrapper around HTTP audio streams
//! that starts playback before the full file is downloaded.

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::io::{Read, Seek, SeekFrom};
use stream_download::{
    http::HttpStream, storage::temp::TempStorageProvider, Settings, StreamDownload,
};

/// A streaming audio source that downloads from HTTP while playing
pub struct StreamingSource {
    reader: StreamDownload<TempStorageProvider>,
}

impl StreamingSource {
    /// Create a new streaming source from a URL with authentication
    ///
    /// # Arguments
    /// * `url` - The audio stream URL
    /// * `auth_token` - Jellyfin authentication token
    ///
    /// # Returns
    /// A StreamingSource that can be read while still downloading
    #[allow(dead_code)]
    pub async fn new(url: &str, auth_token: &str) -> Result<Self> {
        Self::with_start_time(url, auth_token, None).await
    }

    /// Create a new streaming source starting from a specific position
    ///
    /// # Arguments
    /// * `url` - The audio stream URL (base URL without startTimeTicks)
    /// * `auth_token` - Jellyfin authentication token
    /// * `start_time_ticks` - Optional start position in Jellyfin ticks (10,000 ticks = 1ms)
    ///
    /// # Returns
    /// A StreamingSource that can be read while still downloading
    pub async fn with_start_time(
        url: &str,
        auth_token: &str,
        start_time_ticks: Option<u64>,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("MediaBrowser Token=\"{auth_token}\""))
                .context("Invalid auth token")?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        // Append startTimeTicks to the URL if provided
        let final_url = match start_time_ticks {
            Some(ticks) if ticks > 0 => {
                if url.contains('?') {
                    format!("{}&startTimeTicks={}", url, ticks)
                } else {
                    format!("{}?startTimeTicks={}", url, ticks)
                }
            }
            _ => url.to_string(),
        };

        let stream = HttpStream::new(client, final_url.parse().context("Invalid URL")?)
            .await
            .context("Failed to create HTTP stream")?;

        let reader = StreamDownload::from_stream(
            stream,
            TempStorageProvider::default(),
            Settings::default()
                .prefetch_bytes(256 * 1024) // Start playing after 256KB buffered
                .cancel_on_drop(true),
        )
        .await
        .context("Failed to create stream download")?;

        Ok(Self { reader })
    }

    /// Get the content length if known
    pub fn content_length(&self) -> Option<u64> {
        self.reader.content_length()
    }
}

impl Read for StreamingSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Seek for StreamingSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}
