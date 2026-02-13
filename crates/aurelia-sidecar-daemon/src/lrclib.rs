//! LrcLib API client for fetching lyrics as fallback

use aurelia_lyrics::{parse_auto, ParsedLyrics};
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

const LRCLIB_GET_URL: &str = "https://lrclib.net/api/get";

pub struct LrcLibClient {
    client: Client,
}

#[derive(Deserialize)]
struct LrcLibTrackResponse {
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
}

impl LrcLibClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Get lyrics by artist, title, album, and duration
    pub async fn get(
        &self,
        artist: &str,
        title: &str,
        album: Option<&str>,
        duration: Option<i64>,
    ) -> anyhow::Result<Option<ParsedLyrics>> {
        debug!("Getting LrcLib lyrics for '{}' by '{}'", title, artist);

        let mut query = vec![
            ("artist_name", artist.to_string()),
            ("track_name", title.to_string()),
        ];

        if let Some(album) = album {
            query.push(("album_name", album.to_string()));
        }

        if let Some(duration) = duration {
            query.push(("duration", duration.to_string()));
        }

        let response = self.client.get(LRCLIB_GET_URL).query(&query).send().await?;

        if !response.status().is_success() {
            debug!("LrcLib get failed: HTTP {}", response.status());
            return Ok(None);
        }

        let track: LrcLibTrackResponse = response.json().await?;

        let lyrics_text = track.synced_lyrics.as_ref().or(track.plain_lyrics.as_ref());

        if let Some(text) = lyrics_text {
            let lyrics = parse_auto(text, "lrc")?;
            if !lyrics.is_empty() {
                debug!("Found lyrics from LrcLib");
                return Ok(Some(lyrics));
            }
        }

        debug!("No lyrics found on LrcLib");
        Ok(None)
    }
}

impl Default for LrcLibClient {
    fn default() -> Self {
        Self::new()
    }
}
