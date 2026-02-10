//! Minimal Jellyfin API client for the daemon

use aurelia_lyrics::ParsedLyrics;
use reqwest::Client;
use serde::Deserialize;

pub struct JellyfinClient {
    client: Client,
    base_url: String,
    api_key: String,
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
    
    /// Get the file path for a media item
    pub async fn get_item_path(&self, item_id: &str) -> anyhow::Result<std::path::PathBuf> {
        let url = format!(
            "{}/Items/{}?api_key={}&fields=Path",
            self.base_url, item_id, self.api_key
        );
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to get item: HTTP {}", response.status());
        }
        
        #[derive(Deserialize)]
        struct ItemResponse {
            Path: Option<String>,
        }
        
        let item: ItemResponse = response.json().await?;
        
        item.Path
            .map(|p| std::path::PathBuf::from(p))
            .ok_or_else(|| anyhow::anyhow!("Item has no path"))
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
            Lyrics: Vec<LyricLine>,
        }
        
        #[derive(Deserialize)]
        struct LyricLine {
            Text: String,
            Start: i64, // in ticks (1 tick = 100 nanoseconds)
        }
        
        let lyrics: LyricsResponse = response.json().await?;
        
        let synced: Vec<aurelia_lyrics::ParsedLyricsLine> = lyrics
            .Lyrics
            .into_iter()
            .map(|line| aurelia_lyrics::ParsedLyricsLine {
                // Convert ticks to milliseconds (1 tick = 100ns = 0.0001ms)
                time_ms: line.Start / 10_000,
                end_time_ms: None,
                line: line.Text,
                words: None,
                agent_id: None,
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
        })
    }
}
