//! Minimal Jellyfin API client for the daemon

use aurelia_lyrics::ParsedLyrics;
use reqwest::Client;
use serde::Deserialize;

pub struct JellyfinClient {
    client: Client,
    base_url: String,
    api_key: String,
}

/// Item metadata needed for lyrics lookup
#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub path: Option<std::path::PathBuf>,
    pub name: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub run_time_ticks: Option<i64>,
}

impl JellyfinClient {
    /// Create a new Jellyfin client
    pub async fn new(base_url: String, api_key: String) -> anyhow::Result<Self> {
        let client = Client::new();

        // Test connection
        let url = format!("{}/System/Info?api_key={}", base_url, api_key);
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to connect to Jellyfin: HTTP {}", response.status());
        }

        tracing::info!("Connected to Jellyfin server at {}", base_url);

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Get item info including path, artist, album, etc.
    pub async fn get_item_info(&self, item_id: &str) -> anyhow::Result<ItemInfo> {
        // Use the list endpoint to avoid potential 400 Bad Request issues on some servers
        // when accessing /Items/{Id} directly without user context.
        let url = format!(
            "{}/Items?ids={}&fields=Path&api_key={}",
            self.base_url, item_id, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get item info: HTTP {}", response.status());
        }

        #[derive(Deserialize)]
        struct ItemsResponse {
            #[serde(rename = "Items")]
            items: Vec<ItemResponse>,
        }

        #[derive(Deserialize)]
        struct ItemResponse {
            #[serde(rename = "Path")]
            path: Option<String>,
            #[serde(rename = "Name")]
            name: Option<String>,
            #[serde(rename = "Artists")]
            artists: Option<Vec<String>>,
            #[serde(rename = "Album")]
            album: Option<String>,
            #[serde(rename = "RunTimeTicks")]
            run_time_ticks: Option<i64>,
        }

        let list: ItemsResponse = response.json().await?;
        let item = list
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Item not found"))?;

        Ok(ItemInfo {
            path: item.path.map(std::path::PathBuf::from),
            name: item.name.unwrap_or_default(),
            artist: item.artists.and_then(|a| a.into_iter().next()),
            album: item.album,
            run_time_ticks: item.run_time_ticks,
        })
    }

    /// Get lyrics from Jellyfin's built-in API
    pub async fn get_lyrics(&self, item_id: &str) -> anyhow::Result<ParsedLyrics> {
        let url = format!(
            "{}/Audio/{}/Lyrics?api_key={}",
            self.base_url, item_id, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get lyrics: HTTP {}", response.status());
        }

        #[derive(Deserialize)]
        struct LyricsResponse {
            #[serde(rename = "Lyrics")]
            lyrics: Vec<LyricLine>,
        }

        #[derive(Deserialize)]
        struct LyricLine {
            #[serde(rename = "Text")]
            text: String,
            #[serde(rename = "Start")]
            start: i64, // in ticks (1 tick = 100 nanoseconds)
        }

        let lyrics: LyricsResponse = response.json().await?;

        let synced: Vec<aurelia_lyrics::ParsedLyricsLine> = lyrics
            .lyrics
            .into_iter()
            .map(|line| aurelia_lyrics::ParsedLyricsLine {
                // Convert ticks to milliseconds (1 tick = 100ns = 0.0001ms)
                time_ms: line.start / 10_000,
                end_time_ms: None,
                line: line.text,
                words: None,
                agent_id: None,
                translation: None,
            })
            .collect();

        let plain: Vec<String> = synced.iter().map(|l| l.line.clone()).collect();

        Ok(ParsedLyrics {
            plain,
            synced,
            sections: None,
            agents: None,
            songwriters: None,
            language: None,
            are_from_remote: true,
        })
    }
}
