//! Example client-side integration for the Sidecar Lyrics Daemon
//! This shows how Aurelia clients would connect to the daemon

use serde::Deserialize;

/// Configuration for connecting to the lyrics daemon
#[derive(Clone, Debug)]
pub struct LyricsDaemonConfig {
    pub daemon_url: String,
}

/// Client for the lyrics daemon API
pub struct LyricsDaemonClient {
    client: reqwest::Client,
    config: LyricsDaemonConfig,
}

/// Parsed lyrics response from daemon
#[derive(Deserialize, Debug, Clone)]
pub struct LyricsResponse {
    pub item_id: String,
    pub found: bool,
    pub source: Option<String>,
    pub lyrics: Option<ParsedLyrics>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedLyrics {
    pub plain: Vec<String>,
    pub synced: Vec<ParsedLyricsLine>,
    pub sections: Option<Vec<ParsedLyricsSection>>,
    pub agents: Option<Vec<ParsedLyricsAgent>>,
    pub songwriters: Option<Vec<String>>,
    pub language: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedLyricsLine {
    pub time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub line: String,
    pub words: Option<Vec<ParsedLyricsWord>>,
    pub agent_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedLyricsWord {
    pub time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub word: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedLyricsSection {
    pub name: String,
    pub start_line_index: usize,
    pub end_line_index: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedLyricsAgent {
    pub id: String,
    pub name: Option<String>,
}

impl LyricsDaemonClient {
    /// Create a new client
    pub fn new(daemon_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            config: LyricsDaemonConfig { daemon_url },
        }
    }
    
    /// Get lyrics for a Jellyfin item
    pub async fn get_lyrics(&self, item_id: &str) -> anyhow::Result<Option<ParsedLyrics>> {
        let url = format!("{}/lyrics/{}", self.config.daemon_url, item_id);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch lyrics: HTTP {}", response.status());
        }
        
        let result: LyricsResponse = response.json().await?;
        
        if result.found {
            Ok(result.lyrics)
        } else {
            Ok(None)
        }
    }
    
    /// Check daemon health
    pub async fn health_check(&self) -> anyhow::Result<HealthResponse> {
        let url = format!("{}/health", self.config.daemon_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }
}

#[derive(Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub jellyfin_connected: bool,
    pub cache_size: usize,
}

// ============================================
// INTEGRATION: How to use in Aurelia clients
// ============================================

/* 

## 1. DESKTOP CLIENT (Electron)

In your settings, add a field for the lyrics daemon URL:

```rust
pub struct AppSettings {
    pub jellyfin_url: String,
    pub jellyfin_token: Option<String>,
    // NEW: Lyrics daemon URL (optional)
    pub lyrics_daemon_url: Option<String>,
}
```

Modify the lyrics fetching logic:

```rust
pub async fn get_parsed_lyrics(&self, song: &Song) -> Result<Option<ParsedLyrics>> {
    // Priority 1: Try lyrics daemon if configured
    if let Some(daemon_url) = &self.settings.lyrics_daemon_url {
        let client = LyricsDaemonClient::new(daemon_url.clone());
        if let Ok(lyrics) = client.get_lyrics(&song.id).await {
            if lyrics.is_some() {
                return Ok(lyrics);
            }
        }
    }
    
    // Priority 2: Try local sidecar files (for desktop)
    if let Some(path) = &song.path {
        if let Ok(lyrics) = try_read_sidecar_lyrics(path).await {
            return Ok(Some(lyrics));
        }
    }
    
    // Priority 3: Try Jellyfin's built-in lyrics API
    if let Some(token) = &self.jellyfin_token {
        if let Ok(lyrics) = self.jellyfin.get_lyrics(&song.id).await {
            return Ok(Some(lyrics));
        }
    }
    
    // Priority 4: Fallback to LrcLib
    if let Ok(lyrics) = self.lrclib.search(&song).await {
        return Ok(Some(lyrics));
    }
    
    Ok(None)
}
```

## 2. WEB CLIENT (Vue/TypeScript)

Add to your settings store:

```typescript
interface Settings {
  jellyfinUrl: string;
  jellyfinToken: string | null;
  // NEW: Lyrics daemon URL (optional)
  lyricsDaemonUrl: string | null;
}
```

Create a simple daemon client:

```typescript
// services/lyricsDaemon.ts
export class LyricsDaemonClient {
  constructor(private baseUrl: string) {}
  
  async getLyrics(itemId: string): Promise<ParsedLyrics | null> {
    const response = await fetch(`${this.baseUrl}/lyrics/${itemId}`);
    if (!response.ok) throw new Error('Failed to fetch lyrics');
    
    const data = await response.json();
    return data.found ? data.lyrics : null;
  }
  
  async healthCheck(): Promise<HealthResponse> {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
  }
}
```

Update your lyrics component:

```typescript
// In your lyrics display component
async function loadLyrics(song: Song) {
  // Priority 1: Try lyrics daemon
  if (settings.lyricsDaemonUrl) {
    const daemon = new LyricsDaemonClient(settings.lyricsDaemonUrl);
    const lyrics = await daemon.getLyrics(song.id);
    if (lyrics) {
      displayLyrics(lyrics);
      return;
    }
  }
  
  // Priority 2: Try Jellyfin's API directly
  const jellyfinLyrics = await fetchJellyfinLyrics(song.id);
  if (jellyfinLyrics) {
    displayLyrics(jellyfinLyrics);
    return;
  }
  
  // Priority 3: LrcLib fallback
  const lrclibLyrics = await searchLrcLib(song);
  if (lrclibLyrics) {
    displayLyrics(lrclibLyrics);
  }
}
```

## 3. MOBILE CLIENT (Kotlin)

Add to your preferences:

```kotlin
data class AppSettings(
    val jellyfinUrl: String,
    val jellyfinToken: String?,
    // NEW: Lyrics daemon URL
    val lyricsDaemonUrl: String?
)
```

Create a simple client:

```kotlin
class LyricsDaemonClient(private val baseUrl: String) {
    private val client = OkHttpClient()
    private val json = Json { ignoreUnknownKeys = true }
    
    suspend fun getLyrics(itemId: String): ParsedLyrics? = withContext(Dispatchers.IO) {
        val request = Request.Builder()
            .url("$baseUrl/lyrics/$itemId")
            .build()
        
        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) return@withContext null
            
            val body = response.body?.string() ?: return@withContext null
            val result = json.decodeFromString<LyricsResponse>(body)
            
            if (result.found) result.lyrics else null
        }
    }
}
```

## KEY BENEFITS

1. **Lightweight**: The daemon is ~5MB vs the full web backend (~50MB+)
2. **Simple deployment**: Single binary, no database needed
3. **Works with any Jellyfin client**: Desktop, mobile, web - all connect the same way
4. **No proxying needed**: Clients connect directly to Jellyfin AND the daemon
5. **In-memory cache**: Fast responses for frequently played songs
6. **CORS enabled**: Web clients can connect directly

## DEPLOYMENT

### Docker Compose Example:

```yaml
version: '3'
services:
  jellyfin:
    image: jellyfin/jellyfin:latest
    volumes:
      - /media/music:/media/music
      - ./jellyfin-config:/config
    
  aurelia-lyrics:
    image: aurelia/lyrics-daemon:latest
    environment:
      - JELLYFIN_URL=http://jellyfin:8096
      - JELLYFIN_API_KEY=your-api-key
      - MUSIC_PATHS=/media/music
      - PORT=8080
    volumes:
      - /media/music:/media/music:ro
    ports:
      - "8080:8080"
```

### Systemd Service:

```ini
[Unit]
Description=Aurelia Sidecar Lyrics Daemon
After=network.target jellyfin.service

[Service]
Type=simple
User=aurelia
ExecStart=/usr/local/bin/aurelia-sidecar-daemon \
  --jellyfin-url http://localhost:8096 \
  --jellyfin-api-key YOUR_API_KEY \
  --music-paths /media/music
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

*/
