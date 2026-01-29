use crate::db::schema::*;
use crate::domain::errors::DomainError;
use crate::domain::models::{SyncDelta, SyncReport, SyncState};
use crate::models::{Album, Artist, Song};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata};
use std::time::Instant;
use tracing::{debug, info};

pub struct LibraryService {
    db: &'static Database,
}

impl LibraryService {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    pub fn get_sync_state(&self) -> Result<SyncState, DomainError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SYNC_STATE)?;

        if let Some(bytes) = table.get("library")? {
            let state = postcard::from_bytes(bytes.value())
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
            Ok(state)
        } else {
            Ok(SyncState::default())
        }
    }

    pub fn update_sync_state(&self, state: &SyncState) -> Result<(), DomainError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SYNC_STATE)?;
            let encoded = postcard::to_stdvec(state)
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
            table.insert("library", encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn sync_library(
        &self,
        songs: &[Song],
        artists: &[Artist],
        albums: &[Album],
        full_sync: bool,
    ) -> Result<SyncReport, DomainError> {
        let start = Instant::now();

        info!(
            "Starting library sync (full_sync: {}, songs: {}, artists: {}, albums: {})",
            full_sync,
            songs.len(),
            artists.len(),
            albums.len()
        );

        // Get current sync state to preserve last_full_sync_time during incremental syncs
        let current_state = self.get_sync_state().unwrap_or_default();

        let write_txn = self.db.begin_write()?;
        {
            // Clear indexes if full sync
            if full_sync {
                debug!("Full sync: clearing all tables and indexes");
                self.clear_all_tables(&write_txn)?;
            }

            // Sync songs with indexes
            let mut songs_table = write_txn.open_table(SONGS)?;
            let mut songs_by_album = write_txn.open_table(SONGS_BY_ALBUM)?;
            let mut songs_by_artist = write_txn.open_table(SONGS_BY_ARTIST)?;
            let mut favorites = write_txn.open_table(FAVORITES)?;

            for song in songs {
                let encoded = postcard::to_stdvec(song)
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                songs_table.insert(song.id.as_str(), encoded.as_slice())?;

                // Update album index
                if let Some(album_id) = &song.album_id {
                    songs_by_album.insert((album_id.as_str(), song.id.as_str()), ())?;
                }

                // Update artist indexes
                if let Some(artist_ids) = &song.artist_ids {
                    for artist_id in artist_ids {
                        songs_by_artist.insert((artist_id.as_str(), song.id.as_str()), ())?;
                    }
                }

                // Update favorites
                if let Some(true) = song.is_favorite {
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    let encoded_ts = postcard::to_stdvec(&timestamp)
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                    favorites.insert(song.id.as_str(), encoded_ts.as_slice())?;
                }
            }

            // Sync artists
            let mut artists_table = write_txn.open_table(ARTISTS)?;
            for artist in artists {
                let encoded = postcard::to_stdvec(artist)
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                artists_table.insert(artist.id.as_str(), encoded.as_slice())?;
            }

            // Sync albums with indexes
            let mut albums_table = write_txn.open_table(ALBUMS)?;
            let mut albums_by_artist = write_txn.open_table(ALBUMS_BY_ARTIST)?;

            for album in albums {
                let album_id = album
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("{}-{}", album.artist, album.name));

                let encoded = postcard::to_stdvec(album)
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                albums_table.insert(album_id.as_str(), encoded.as_slice())?;

                // Update artist-album index
                if let Some(artist_id) = &album.artist_id {
                    albums_by_artist.insert((artist_id.as_str(), album_id.as_str()), ())?;
                }
            }

            // Update sync state
            let new_state = SyncState {
                last_sync_time: chrono::Utc::now().to_rfc3339(),
                last_full_sync_time: if full_sync {
                    Some(chrono::Utc::now().to_rfc3339())
                } else {
                    current_state.last_full_sync_time.clone()
                },
                last_sync_version: None,
                song_count: songs.len() as u32,
                artist_count: artists.len() as u32,
                album_count: albums.len() as u32,
            };

            let mut sync_state_table = write_txn.open_table(SYNC_STATE)?;
            let encoded_state = postcard::to_stdvec(&new_state)
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
            sync_state_table.insert("library", encoded_state.as_slice())?;
        }

        write_txn.commit()?;

        let duration = start.elapsed();
        info!(
            "Library sync completed in {}ms (songs: {}, artists: {}, albums: {})",
            duration.as_millis(),
            songs.len(),
            artists.len(),
            albums.len()
        );

        Ok(SyncReport {
            full_sync,
            songs_updated: songs.len() as u32,
            artists_updated: artists.len() as u32,
            albums_updated: albums.len() as u32,
            duration_ms: duration.as_millis() as u64,
        })
    }

    fn clear_all_tables(&self, write_txn: &redb::WriteTransaction) -> Result<(), DomainError> {
        // Clear main tables
        self.clear_table(write_txn, SONGS)?;
        self.clear_table(write_txn, ARTISTS)?;
        self.clear_table(write_txn, ALBUMS)?;

        // Clear index tables
        self.clear_composite_table(write_txn, SONGS_BY_ALBUM)?;
        self.clear_composite_table(write_txn, SONGS_BY_ARTIST)?;
        self.clear_composite_table(write_txn, ALBUMS_BY_ARTIST)?;

        // Clear metadata tables
        self.clear_table(write_txn, FAVORITES)?;

        Ok(())
    }

    fn clear_table(
        &self,
        write_txn: &redb::WriteTransaction,
        table_def: redb::TableDefinition<&str, &[u8]>,
    ) -> Result<(), DomainError> {
        let mut table = write_txn.open_table(table_def)?;
        let mut keys = Vec::new();
        for item in table.iter()? {
            let (key, _) = item?;
            keys.push(key.value().to_string());
        }
        for key in keys {
            table.remove(key.as_str())?;
        }
        Ok(())
    }

    fn clear_composite_table(
        &self,
        write_txn: &redb::WriteTransaction,
        table_def: redb::TableDefinition<(&str, &str), ()>,
    ) -> Result<(), DomainError> {
        let mut table = write_txn.open_table(table_def)?;
        let mut keys = Vec::new();
        for item in table.iter()? {
            let (key, _) = item?;
            let (k1, k2) = key.value();
            keys.push((k1.to_string(), k2.to_string()));
        }
        for (k1, k2) in keys {
            table.remove((k1.as_str(), k2.as_str()))?;
        }
        Ok(())
    }

    pub fn get_library_stats(&self) -> Result<(u32, u32, u32), DomainError> {
        let read_txn = self.db.begin_read()?;

        let songs_table = read_txn.open_table(SONGS)?;
        let artists_table = read_txn.open_table(ARTISTS)?;
        let albums_table = read_txn.open_table(ALBUMS)?;

        let song_count = songs_table.len()? as u32;
        let artist_count = artists_table.len()? as u32;
        let album_count = albums_table.len()? as u32;

        Ok((song_count, artist_count, album_count))
    }

    /// Get all local song IDs and their modification timestamps
    pub fn get_local_songs_metadata(
        &self,
    ) -> Result<std::collections::HashMap<String, Option<String>>, DomainError> {
        let read_txn = self.db.begin_read()?;
        let songs_table = read_txn.open_table(SONGS)?;

        let mut result = std::collections::HashMap::new();
        for item in songs_table.iter()? {
            let (key, value) = item?;
            let song: Song = postcard::from_bytes(value.value())
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
            result.insert(key.value().to_string(), song.date_modified.clone());
        }
        Ok(result)
    }

    /// Compute delta between local and remote library state
    /// Returns a SyncDelta indicating what needs to be added, removed, or updated
    pub fn compute_delta(
        &self,
        remote_songs: &[Song],
        remote_artists: &[Artist],
        remote_albums: &[Album],
    ) -> Result<SyncDelta, DomainError> {
        let mut delta = SyncDelta::default();

        // Get local metadata
        let local_songs = self.get_local_songs_metadata()?;

        // Build remote ID sets for quick lookup
        let remote_song_ids: std::collections::HashSet<&str> =
            remote_songs.iter().map(|s| s.id.as_str()).collect();
        let remote_artist_ids: std::collections::HashSet<&str> =
            remote_artists.iter().map(|a| a.id.as_str()).collect();
        let remote_album_ids: std::collections::HashSet<&str> = remote_albums
            .iter()
            .filter_map(|a| a.id.as_deref())
            .collect();

        // Find songs to add or update
        for song in remote_songs {
            if let Some(local_date_modified) = local_songs.get(&song.id) {
                // Song exists locally - check if it needs update
                if local_date_modified != &song.date_modified {
                    delta.songs_to_update.push(song.id.clone());
                }
            } else {
                // Song doesn't exist locally - add it
                delta.songs_to_add.push(song.id.clone());
            }
        }

        // Find songs to remove (exist locally but not remotely)
        for local_id in local_songs.keys() {
            if !remote_song_ids.contains(local_id.as_str()) {
                delta.songs_to_remove.push(local_id.clone());
            }
        }

        // Get local artist/album IDs
        let local_artist_ids = self.get_local_entity_ids(ARTISTS)?;
        let local_album_ids = self.get_local_entity_ids(ALBUMS)?;

        // Find artists to add/remove (updates handled separately by comparing timestamps)
        for artist in remote_artists {
            if !local_artist_ids.contains(&artist.id) {
                delta.artists_to_add.push(artist.id.clone());
            }
        }
        for local_id in &local_artist_ids {
            if !remote_artist_ids.contains(local_id.as_str()) {
                delta.artists_to_remove.push(local_id.clone());
            }
        }

        // Find albums to add/remove
        for album in remote_albums {
            if let Some(id) = &album.id {
                if !local_album_ids.contains(id) {
                    delta.albums_to_add.push(id.clone());
                }
            }
        }
        for local_id in &local_album_ids {
            if !remote_album_ids.contains(local_id.as_str()) {
                delta.albums_to_remove.push(local_id.clone());
            }
        }

        info!(
            "Computed sync delta: {} songs to add, {} to remove, {} to update; {} artists add/remove; {} albums add/remove",
            delta.songs_to_add.len(),
            delta.songs_to_remove.len(),
            delta.songs_to_update.len(),
            delta.artists_to_add.len() + delta.artists_to_remove.len(),
            delta.albums_to_add.len() + delta.albums_to_remove.len()
        );

        Ok(delta)
    }

    /// Get all IDs from a table
    fn get_local_entity_ids(
        &self,
        table_def: redb::TableDefinition<&str, &[u8]>,
    ) -> Result<std::collections::HashSet<String>, DomainError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(table_def)?;
        let mut ids = std::collections::HashSet::new();
        for item in table.iter()? {
            let (key, _) = item?;
            ids.insert(key.value().to_string());
        }
        Ok(ids)
    }

    /// Apply a computed delta to the database
    /// Only modifies items that have changed, rather than rewriting everything
    pub fn apply_delta(
        &self,
        delta: &SyncDelta,
        songs: &[Song],
        artists: &[Artist],
        albums: &[Album],
    ) -> Result<SyncReport, DomainError> {
        let start = Instant::now();

        if delta.is_empty() {
            info!("No changes to apply - library is up to date");
            return Ok(SyncReport {
                full_sync: false,
                songs_updated: 0,
                artists_updated: 0,
                albums_updated: 0,
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }

        info!(
            "Applying delta: {} song changes, {} artist changes, {} album changes",
            delta.songs_to_add.len() + delta.songs_to_remove.len() + delta.songs_to_update.len(),
            delta.artists_to_add.len() + delta.artists_to_remove.len(),
            delta.albums_to_add.len() + delta.albums_to_remove.len()
        );

        // Build lookup maps for efficient access
        let songs_map: std::collections::HashMap<&str, &Song> =
            songs.iter().map(|s| (s.id.as_str(), s)).collect();
        let artists_map: std::collections::HashMap<&str, &Artist> =
            artists.iter().map(|a| (a.id.as_str(), a)).collect();
        let albums_map: std::collections::HashMap<&str, &Album> = albums
            .iter()
            .filter_map(|a| a.id.as_deref().map(|id| (id, a)))
            .collect();

        let write_txn = self.db.begin_write()?;
        {
            // Handle songs
            let mut songs_table = write_txn.open_table(SONGS)?;
            let mut songs_by_album = write_txn.open_table(SONGS_BY_ALBUM)?;
            let mut songs_by_artist = write_txn.open_table(SONGS_BY_ARTIST)?;
            let mut favorites = write_txn.open_table(FAVORITES)?;

            // Add new songs
            for song_id in &delta.songs_to_add {
                if let Some(song) = songs_map.get(song_id.as_str()) {
                    let encoded = postcard::to_stdvec(song)
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                    songs_table.insert(song.id.as_str(), encoded.as_slice())?;

                    // Update indexes
                    if let Some(album_id) = &song.album_id {
                        songs_by_album.insert((album_id.as_str(), song.id.as_str()), ())?;
                    }
                    if let Some(artist_ids) = &song.artist_ids {
                        for artist_id in artist_ids {
                            songs_by_artist.insert((artist_id.as_str(), song.id.as_str()), ())?;
                        }
                    }
                    if song.is_favorite == Some(true) {
                        let timestamp = chrono::Utc::now().to_rfc3339();
                        let encoded_ts = postcard::to_stdvec(&timestamp)
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                        favorites.insert(song.id.as_str(), encoded_ts.as_slice())?;
                    }
                }
            }

            // Update modified songs
            for song_id in &delta.songs_to_update {
                if let Some(song) = songs_map.get(song_id.as_str()) {
                    let encoded = postcard::to_stdvec(song)
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                    songs_table.insert(song.id.as_str(), encoded.as_slice())?;
                }
            }

            // Remove deleted songs
            for song_id in &delta.songs_to_remove {
                songs_table.remove(song_id.as_str())?;
                favorites.remove(song_id.as_str())?;
                // Note: Index cleanup would require iteration; skipping for efficiency
                // Full sync will clean up orphaned indexes
            }

            // Handle artists
            let mut artists_table = write_txn.open_table(ARTISTS)?;
            for artist_id in &delta.artists_to_add {
                if let Some(artist) = artists_map.get(artist_id.as_str()) {
                    let encoded = postcard::to_stdvec(artist)
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                    artists_table.insert(artist.id.as_str(), encoded.as_slice())?;
                }
            }
            for artist_id in &delta.artists_to_remove {
                artists_table.remove(artist_id.as_str())?;
            }

            // Handle albums
            let mut albums_table = write_txn.open_table(ALBUMS)?;
            for album_id in &delta.albums_to_add {
                if let Some(album) = albums_map.get(album_id.as_str()) {
                    let encoded = postcard::to_stdvec(album)
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                    albums_table.insert(album_id.as_str(), encoded.as_slice())?;
                }
            }
            for album_id in &delta.albums_to_remove {
                albums_table.remove(album_id.as_str())?;
            }

            // Update sync state
            let current_state = self.get_sync_state().unwrap_or_default();
            let new_state = SyncState {
                last_sync_time: chrono::Utc::now().to_rfc3339(),
                last_full_sync_time: current_state.last_full_sync_time,
                last_sync_version: None,
                song_count: songs_table.len()? as u32,
                artist_count: artists_table.len()? as u32,
                album_count: albums_table.len()? as u32,
            };

            let mut sync_state_table = write_txn.open_table(SYNC_STATE)?;
            let encoded_state = postcard::to_stdvec(&new_state)
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
            sync_state_table.insert("library", encoded_state.as_slice())?;
        }

        write_txn.commit()?;

        let duration = start.elapsed();
        info!("Delta sync completed in {}ms", duration.as_millis());

        Ok(SyncReport {
            full_sync: false,
            songs_updated: (delta.songs_to_add.len()
                + delta.songs_to_update.len()
                + delta.songs_to_remove.len()) as u32,
            artists_updated: (delta.artists_to_add.len() + delta.artists_to_remove.len()) as u32,
            albums_updated: (delta.albums_to_add.len() + delta.albums_to_remove.len()) as u32,
            duration_ms: duration.as_millis() as u64,
        })
    }
}
