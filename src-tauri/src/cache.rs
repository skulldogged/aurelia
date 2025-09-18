//! JSON-based cache system to replace SQLite
use crate::models::{Album, Artist, Song};
use crate::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Cache metadata for versioning and timestamps
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheMetadata {
    version: String,
    created_at: String,
    last_updated: String,
}

/// Main cache structure with in-memory indexes
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonCache {
    /// All cached songs
    songs: Vec<Song>,
    /// All cached artists
    artists: Vec<Artist>,
    /// All cached albums
    albums: Vec<Album>,
    /// Metadata
    metadata: CacheMetadata,
}

/// Thread-safe cache wrapper
pub struct CacheManager {
    cache: RwLock<Option<JsonCache>>,
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new() -> Self {
        let cache_dir = utils::get_app_data_dir()
            .expect("Failed to get app data dir")
            .join("cache");

        Self {
            cache: RwLock::new(None),
            cache_dir,
        }
    }

    /// Get cache directory path
    fn get_cache_dir(&self) -> PathBuf {
        self.cache_dir.clone()
    }

    /// Get path for a specific cache file
    fn get_cache_file(&self, name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", name))
    }

    /// Load cache from disk
    async fn load_cache(&self) -> Result<JsonCache, Box<dyn std::error::Error + Send + Sync>> {
        std::fs::create_dir_all(&self.cache_dir)?;

        let songs_file = self.get_cache_file("songs");
        let artists_file = self.get_cache_file("artists");
        let albums_file = self.get_cache_file("albums");
        let metadata_file = self.get_cache_file("metadata");

        let songs = if songs_file.exists() {
            let data = tokio::fs::read_to_string(&songs_file).await?;
            serde_json::from_str(&data)?
        } else {
            Vec::new()
        };

        let artists = if artists_file.exists() {
            let data = tokio::fs::read_to_string(&artists_file).await?;
            serde_json::from_str(&data)?
        } else {
            Vec::new()
        };

        let albums = if albums_file.exists() {
            let data = tokio::fs::read_to_string(&albums_file).await?;
            serde_json::from_str(&data)?
        } else {
            Vec::new()
        };

        let metadata = if metadata_file.exists() {
            let data = tokio::fs::read_to_string(&metadata_file).await?;
            serde_json::from_str(&data)?
        } else {
            CacheMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            }
        };

        Ok(JsonCache {
            songs,
            artists,
            albums,
            metadata,
        })
    }

    /// Save cache to disk
    async fn save_cache(
        &self,
        cache: &JsonCache,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_metadata = cache.metadata.clone();
        updated_metadata.last_updated = chrono::Utc::now().to_rfc3339();

        let updated_cache = JsonCache {
            metadata: updated_metadata,
            songs: cache.songs.clone(),
            artists: cache.artists.clone(),
            albums: cache.albums.clone(),
        };

        let songs_data = serde_json::to_string_pretty(&updated_cache.songs)?;
        tokio::fs::write(self.get_cache_file("songs"), songs_data).await?;

        let artists_data = serde_json::to_string_pretty(&updated_cache.artists)?;
        tokio::fs::write(self.get_cache_file("artists"), artists_data).await?;

        let albums_data = serde_json::to_string_pretty(&updated_cache.albums)?;
        tokio::fs::write(self.get_cache_file("albums"), albums_data).await?;

        let metadata_data = serde_json::to_string_pretty(&updated_cache.metadata)?;
        tokio::fs::write(self.get_cache_file("metadata"), metadata_data).await?;

        Ok(())
    }

    /// Initialize the cache system
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.load_cache().await?;
        *self.cache.write().await = Some(cache);
        Ok(())
    }

    /// Cache music library
    pub async fn cache_music_library(
        &self,
        items: &[Song],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;
        let cache_data = cache.as_mut().ok_or("Cache not initialized")?;

        cache_data.songs = items.to_vec();
        self.save_cache(cache_data).await?;

        Ok(())
    }

    /// Get cached music library
    pub async fn get_cached_music_library(
        &self,
    ) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let cache_data = cache.as_ref().ok_or("Cache not initialized")?;

        Ok(cache_data.songs.clone())
    }

    /// Cache artists
    pub async fn cache_artists(
        &self,
        artists: &[Artist],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;
        let cache_data = cache.as_mut().ok_or("Cache not initialized")?;

        cache_data.artists = artists.to_vec();
        self.save_cache(cache_data).await?;

        Ok(())
    }

    /// Get cached artists
    pub async fn get_cached_artists(
        &self,
    ) -> Result<Vec<Artist>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let cache_data = cache.as_ref().ok_or("Cache not initialized")?;

        Ok(cache_data.artists.clone())
    }

    /// Get all albums
    pub async fn get_all_albums(
        &self,
    ) -> Result<Vec<Album>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let cache_data = cache.as_ref().ok_or("Cache not initialized")?;

        Ok(cache_data.albums.clone())
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_dir = self.get_cache_dir();

        if cache_dir.exists() {
            tokio::fs::remove_dir_all(&cache_dir).await?;
            std::fs::create_dir_all(&cache_dir)?;
        }

        let mut cache = self.cache.write().await;
        *cache = Some(JsonCache {
            songs: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
            metadata: CacheMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        });

        Ok(())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global cache instance
static CACHE_MANAGER: once_cell::sync::Lazy<CacheManager> =
    once_cell::sync::Lazy::new(CacheManager::new);

/// Initialize the cache system
pub async fn init() -> Result<(), String> {
    CACHE_MANAGER.initialize().await.map_err(|e| e.to_string())
}

/// Cache the music library
pub async fn cache_library(songs: &[Song]) -> Result<(), String> {
    CACHE_MANAGER
        .cache_music_library(songs)
        .await
        .map_err(|e| e.to_string())
}

/// Get cached songs
pub async fn get_songs() -> Result<Vec<Song>, String> {
    CACHE_MANAGER
        .get_cached_music_library()
        .await
        .map_err(|e| e.to_string())
}

/// Cache artists
pub async fn cache_artists(artists: &[Artist]) -> Result<(), String> {
    CACHE_MANAGER
        .cache_artists(artists)
        .await
        .map_err(|e| e.to_string())
}

/// Get cached artists
pub async fn get_artists() -> Result<Vec<Artist>, String> {
    CACHE_MANAGER
        .get_cached_artists()
        .await
        .map_err(|e| e.to_string())
}

/// Get cached albums
pub async fn get_albums() -> Result<Vec<Album>, String> {
    CACHE_MANAGER
        .get_all_albums()
        .await
        .map_err(|e| e.to_string())
}

/// Clear all cache data
pub async fn clear_cache() -> Result<(), String> {
    CACHE_MANAGER.clear_cache().await.map_err(|e| e.to_string())
}
