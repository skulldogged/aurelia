//! LrcLib API service client

use crate::error::{AppError, AppResult};
use crate::models::LrcLibTrackResponse;
use crate::utils;

/// LrcLib API client for fetching lyrics
pub struct LrcLibClient {
    client: reqwest::Client,
}

impl LrcLibClient {
    /// Create a new LrcLib client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Search for lyrics by artist and track name
    pub async fn search_lyrics(&self, artist: &str, title: &str) -> AppResult<Vec<LrcLibTrackResponse>> {
        let response = self
            .client
            .get(utils::constants::LRCLIB_SEARCH_URL)
            .query(&[("artist_name", artist), ("track_name", title)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to search for lyrics: HTTP {}",
                response.status()
            )));
        }

        let search_results: Vec<LrcLibTrackResponse> = response.json().await?;
        Ok(search_results)
    }

    /// Get the best lyrics match from search results
    pub fn get_best_lyrics(search_results: &[LrcLibTrackResponse]) -> Option<String> {
        // Take the first result that has lyrics
        search_results
            .iter()
            .find_map(|track| track.synced_lyrics.as_ref())
            .or_else(|| {
                search_results
                    .iter()
                    .find_map(|track| track.plain_lyrics.as_ref())
            })
            .cloned()
    }
}

impl Default for LrcLibClient {
    fn default() -> Self {
        Self::new()
    }
}
