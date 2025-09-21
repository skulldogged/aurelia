use crate::models::{Album, Artist, Song};
use crate::utils;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::info;

type ImageMetadata = HashMap<String, HashMap<String, String>>;

/// Cache metadata for versioning and timestamps
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheMetadata {
    version: String,
    created_at: String,
    last_updated: String,
}

/// Main cache structure with in-memory indexes
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    #[must_use]
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
        self.cache_dir.join(format!("{name}.json"))
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

    /// Load image metadata from disk
    async fn load_image_metadata(
        &self,
    ) -> Result<ImageMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let metadata_path = self.get_cache_dir().join("image_metadata.json");
        if metadata_path.exists() {
            let data = tokio::fs::read_to_string(metadata_path).await?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Save image metadata to disk
    async fn save_image_metadata(
        &self,
        metadata: &ImageMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metadata_path = self.get_cache_dir().join("image_metadata.json");
        let data = serde_json::to_string_pretty(metadata)?;
        tokio::fs::write(metadata_path, data).await?;
        Ok(())
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

    /// Overwrite the music library in the cache
    pub async fn overwrite_music_library(
        &self,
        items: &[Song],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_data = {
            let mut cache = self.cache.write().await;
            let cache_data = cache.as_mut().ok_or("Cache not initialized")?;
            cache_data.songs = items.to_vec();
            cache_data.clone()
        };
        self.save_cache(&cache_data).await?;

        Ok(())
    }

    /// Sync the music library in the cache
    pub fn sync_songs(
        cache_data: &mut JsonCache,
        fetched_songs: &[Song],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting song sync. Cached: {}, Fetched: {}",
            cache_data.songs.len(),
            fetched_songs.len()
        );

        let existing_song_map: HashMap<String, Song> = cache_data
            .songs
            .iter()
            .map(|s| (s.id.clone(), s.clone()))
            .collect();

        let mut new_count = 0;
        let mut updated_count = 0;
        for song in fetched_songs {
            if let Some(existing_song) = existing_song_map.get(&song.id) {
                if existing_song != song {
                    updated_count += 1;
                }
            } else {
                new_count += 1;
            }
        }

        let fetched_song_ids: HashSet<String> =
            fetched_songs.iter().map(|s| s.id.clone()).collect();
        let removed_count = existing_song_map
            .keys()
            .filter(|&id| !fetched_song_ids.contains(id))
            .count();

        let mut final_song_map = existing_song_map;
        final_song_map.extend(fetched_songs.iter().map(|s| (s.id.clone(), s.clone())));
        final_song_map.retain(|id, _| fetched_song_ids.contains(id));
        cache_data.songs = final_song_map.values().cloned().collect();

        info!(
            "Song sync complete. Added: {}, Updated: {}, Removed: {}, Total: {}",
            new_count,
            updated_count,
            removed_count,
            cache_data.songs.len()
        );

        Ok(())
    }

    fn sync_artists(cache_data: &mut JsonCache, fetched_artists: &[Artist]) {
        info!(
            "Starting artist sync. Cached: {}, Fetched: {}",
            cache_data.artists.len(),
            fetched_artists.len()
        );

        let existing_artist_map: HashMap<String, Artist> = cache_data
            .artists
            .iter()
            .map(|a| (a.id.clone(), a.clone()))
            .collect();

        let mut new_count = 0;
        let mut updated_count = 0;
        for artist in fetched_artists {
            if let Some(existing_artist) = existing_artist_map.get(&artist.id) {
                if existing_artist != artist {
                    updated_count += 1;
                }
            } else {
                new_count += 1;
            }
        }

        let fetched_artist_ids: HashSet<String> =
            fetched_artists.iter().map(|a| a.id.clone()).collect();
        let removed_count = existing_artist_map
            .keys()
            .filter(|&id| !fetched_artist_ids.contains(id))
            .count();

        let mut final_artist_map = existing_artist_map;
        final_artist_map.extend(fetched_artists.iter().map(|a| (a.id.clone(), a.clone())));
        final_artist_map.retain(|id, _| fetched_artist_ids.contains(id));
        cache_data.artists = final_artist_map.values().cloned().collect();

        info!(
            "Artist sync complete. Added: {}, Updated: {}, Removed: {}, Total: {}",
            new_count,
            updated_count,
            removed_count,
            cache_data.artists.len()
        );
    }

    fn sync_albums(cache_data: &mut JsonCache, fetched_albums: &[Album]) {
        info!(
            "Starting album sync. Cached: {}, Fetched: {}",
            cache_data.albums.len(),
            fetched_albums.len()
        );

        let existing_album_map: HashMap<String, Album> = cache_data
            .albums
            .iter()
            .filter_map(|a| a.id.as_ref().map(|id| (id.clone(), a.clone())))
            .collect();

        let mut new_count = 0;
        let mut updated_count = 0;
        for album in fetched_albums {
            if let Some(id) = &album.id {
                if let Some(existing_album) = existing_album_map.get(id) {
                    if existing_album != album {
                        updated_count += 1;
                    }
                } else {
                    new_count += 1;
                }
            }
        }

        let fetched_album_ids: HashSet<String> =
            fetched_albums.iter().filter_map(|a| a.id.clone()).collect();
        let removed_count = existing_album_map
            .keys()
            .filter(|&id| !fetched_album_ids.contains(id.as_str()))
            .count();

        let mut final_album_map = existing_album_map;
        final_album_map.extend(
            fetched_albums
                .iter()
                .filter_map(|a| a.id.as_ref().map(|id| (id.clone(), a.clone()))),
        );
        final_album_map.retain(|id, _| fetched_album_ids.contains(id));
        cache_data.albums = final_album_map.values().cloned().collect();

        info!(
            "Album sync complete. Added: {}, Updated: {}, Removed: {}, Total: {}",
            new_count,
            updated_count,
            removed_count,
            cache_data.albums.len()
        );
    }

    pub async fn sync_library(
        &self,
        songs: &[Song],
        artists: &[Artist],
        albums: &[Album],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_to_save = {
            let mut cache = self.cache.write().await;
            let cache_data = cache.as_mut().ok_or("Cache not initialized")?;

            Self::sync_songs(cache_data, songs)?;
            Self::sync_artists(cache_data, artists);
            Self::sync_albums(cache_data, albums);
            self.sync_images(songs, artists, albums).await?;
            cache_data.clone()
        };

        self.save_cache(&cache_to_save).await?;

        Ok(())
    }

    /// Get cached music library
    pub async fn get_cached_music_library(
        &self,
    ) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let songs = {
            let cache = self.cache.read().await;
            let cache_data = cache.as_ref().ok_or("Cache not initialized")?;
            cache_data.songs.clone()
        };

        Ok(songs)
    }

    /// Cache artists
    pub async fn cache_artists(
        &self,
        artists: &[Artist],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_to_save = {
            let mut cache = self.cache.write().await;
            let cache_data = cache.as_mut().ok_or("Cache not initialized")?;
            cache_data.artists = artists.to_vec();
            cache_data.clone()
        };
        self.save_cache(&cache_to_save).await?;

        Ok(())
    }

    /// Get cached artists
    pub async fn get_cached_artists(
        &self,
    ) -> Result<Vec<Artist>, Box<dyn std::error::Error + Send + Sync>> {
        let artists = {
            let cache = self.cache.read().await;
            let cache_data = cache.as_ref().ok_or("Cache not initialized")?;
            cache_data.artists.clone()
        };

        Ok(artists)
    }

    /// Cache albums
    pub async fn cache_albums(
        &self,
        albums: &[Album],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_to_save = {
            let mut cache = self.cache.write().await;
            let cache_data = cache.as_mut().ok_or("Cache not initialized")?;
            cache_data.albums = albums.to_vec();
            cache_data.clone()
        };
        self.save_cache(&cache_to_save).await?;

        Ok(())
    }

    /// Get all albums
    pub async fn get_all_albums(
        &self,
    ) -> Result<Vec<Album>, Box<dyn std::error::Error + Send + Sync>> {
        let albums = {
            let cache = self.cache.read().await;
            let cache_data = cache.as_ref().ok_or("Cache not initialized")?;
            cache_data.albums.clone()
        };
        Ok(albums)
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_dir = self.get_cache_dir();

        if cache_dir.exists() {
            tokio::fs::remove_dir_all(&cache_dir).await?;
            std::fs::create_dir_all(&cache_dir)?;
        }

        *self.cache.write().await = Some(JsonCache {
            songs: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
            metadata: CacheMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        });

        info!("Local music library cache cleared.");

        Ok(())
    }

    async fn sync_images(
        &self,
        songs: &[Song],
        artists: &[Artist],
        albums: &[Album],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut image_metadata = self.load_image_metadata().await?;
        let mut updated = false;

        let mut process_item = |id: &str, tags: &Option<HashMap<String, String>>| {
            if let Some(tags) = tags
                && image_metadata.get(id) != Some(tags)
            {
                image_metadata.insert(id.to_string(), tags.clone());
                updated = true;
            }
        };

        for song in songs {
            process_item(&song.id, &song.image_tags);
        }
        for artist in artists {
            process_item(&artist.id, &artist.image_tags);
        }
        for album in albums {
            if let Some(id) = &album.id {
                process_item(id, &album.image_tags);
            }
        }

        if updated {
            self.save_image_metadata(&image_metadata).await?;
            info!("Image metadata sync complete.");
        } else {
            info!("No image changes detected.");
        }

        Ok(())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global cache instance
static CACHE_MANAGER: std::sync::LazyLock<CacheManager> =
    std::sync::LazyLock::new(CacheManager::new);

/// Initialize the cache system
pub async fn init() -> Result<(), String> {
    CACHE_MANAGER.initialize().await.map_err(|e| e.to_string())
}

/// Cache the music library (overwrite)
pub async fn cache_library(songs: &[Song]) -> Result<(), String> {
    CACHE_MANAGER
        .overwrite_music_library(songs)
        .await
        .map_err(|e| e.to_string())
}

/// Sync the music library
pub async fn sync_library(
    songs: &[Song],
    artists: &[Artist],
    albums: &[Album],
) -> Result<(), String> {
    CACHE_MANAGER
        .sync_library(songs, artists, albums)
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

/// Cache albums
pub async fn cache_albums(albums: &[Album]) -> Result<(), String> {
    CACHE_MANAGER
        .cache_albums(albums)
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
