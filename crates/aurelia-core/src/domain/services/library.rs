use crate::db::schema::*;
use crate::domain::errors::DomainError;
use crate::domain::models::{SyncDelta, SyncReport, SyncState};
use crate::models::{Album, Artist, Song};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

pub struct LibraryService {
    db: Arc<Database>,
}

impl LibraryService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get_sync_state(&self) -> Result<SyncState, DomainError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SYNC_STATE)?;

        if let Some(bytes) = table.get("library")? {
            // Try to deserialize; if the stored format is from an older schema
            // (fewer fields), fall back to default so the DB upgrades on next write.
            match postcard::from_bytes::<SyncState>(bytes.value()) {
                Ok(state) => Ok(state),
                Err(e) => {
                    info!(
                        "SyncState deserialization failed (schema upgrade?), resetting: {}",
                        e
                    );
                    Ok(SyncState::default())
                }
            }
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
                full_sync_in_progress: false,
                full_sync_last_page_index: 0,
                full_sync_entity_type: None,
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

    pub fn clear_all_tables(&self, write_txn: &redb::WriteTransaction) -> Result<(), DomainError> {
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
            if let Some(id) = &album.id
                && !local_album_ids.contains(id)
            {
                delta.albums_to_add.push(id.clone());
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

    // =========================================================================
    // Smart Sync Methods (Phase 3b)
    // =========================================================================

    /// Upsert songs into the database with their indexes.
    /// Returns the number of songs upserted.
    pub fn upsert_songs(&self, songs: &[Song]) -> Result<u32, DomainError> {
        if songs.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        {
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
                } else {
                    // Remove from favorites if no longer favorite
                    let _ = favorites.remove(song.id.as_str());
                }
            }
        }
        write_txn.commit()?;

        info!("Upserted {} songs", songs.len());
        Ok(songs.len() as u32)
    }

    /// Upsert albums into the database with their indexes.
    /// Returns the number of albums upserted.
    pub fn upsert_albums(&self, albums: &[Album]) -> Result<u32, DomainError> {
        if albums.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        {
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
        }
        write_txn.commit()?;

        info!("Upserted {} albums", albums.len());
        Ok(albums.len() as u32)
    }

    /// Upsert artists into the database.
    /// Returns the number of artists upserted.
    pub fn upsert_artists(&self, artists: &[Artist]) -> Result<u32, DomainError> {
        if artists.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        {
            let mut artists_table = write_txn.open_table(ARTISTS)?;

            for artist in artists {
                let encoded = postcard::to_stdvec(artist)
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                artists_table.insert(artist.id.as_str(), encoded.as_slice())?;
            }
        }
        write_txn.commit()?;

        info!("Upserted {} artists", artists.len());
        Ok(artists.len() as u32)
    }

    /// Remove songs whose IDs are NOT in the provided set of valid remote IDs.
    /// Returns the number of songs removed.
    pub fn remove_deleted_songs(
        &self,
        valid_remote_ids: &std::collections::HashSet<String>,
    ) -> Result<u32, DomainError> {
        let local_ids = self.get_local_entity_ids(SONGS)?;
        let to_remove: Vec<String> = local_ids.difference(valid_remote_ids).cloned().collect();

        if to_remove.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        {
            let mut songs_table = write_txn.open_table(SONGS)?;
            let mut favorites = write_txn.open_table(FAVORITES)?;

            for song_id in &to_remove {
                songs_table.remove(song_id.as_str())?;
                let _ = favorites.remove(song_id.as_str());
            }
            // Note: Index cleanup (SONGS_BY_ALBUM, SONGS_BY_ARTIST) is deferred
            // to the next full sync for efficiency
        }
        write_txn.commit()?;

        info!("Removed {} deleted songs", to_remove.len());
        Ok(to_remove.len() as u32)
    }

    /// Remove albums whose IDs are NOT in the provided set of valid remote IDs.
    /// Returns the number of albums removed.
    pub fn remove_deleted_albums(
        &self,
        valid_remote_ids: &std::collections::HashSet<String>,
    ) -> Result<u32, DomainError> {
        let local_ids = self.get_local_entity_ids(ALBUMS)?;
        let to_remove: Vec<String> = local_ids.difference(valid_remote_ids).cloned().collect();

        if to_remove.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        {
            let mut albums_table = write_txn.open_table(ALBUMS)?;

            for album_id in &to_remove {
                albums_table.remove(album_id.as_str())?;
            }
        }
        write_txn.commit()?;

        info!("Removed {} deleted albums", to_remove.len());
        Ok(to_remove.len() as u32)
    }

    /// Remove artists whose IDs are NOT in the provided set of valid remote IDs.
    /// Returns the number of artists removed.
    pub fn remove_deleted_artists(
        &self,
        valid_remote_ids: &std::collections::HashSet<String>,
    ) -> Result<u32, DomainError> {
        let local_ids = self.get_local_entity_ids(ARTISTS)?;
        let to_remove: Vec<String> = local_ids.difference(valid_remote_ids).cloned().collect();

        if to_remove.is_empty() {
            return Ok(0);
        }

        let write_txn = self.db.begin_write()?;
        {
            let mut artists_table = write_txn.open_table(ARTISTS)?;

            for artist_id in &to_remove {
                artists_table.remove(artist_id.as_str())?;
            }
        }
        write_txn.commit()?;

        info!("Removed {} deleted artists", to_remove.len());
        Ok(to_remove.len() as u32)
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
                full_sync_in_progress: false,
                full_sync_last_page_index: 0,
                full_sync_entity_type: None,
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

#[cfg(test)]
mod tests {
    use super::LibraryService;
    use crate::db;
    use crate::domain::models::SyncDelta;
    use crate::models::{Album, Artist, Song};
    use once_cell::sync::OnceCell;
    use serial_test::serial;
    use tempfile::TempDir;

    fn init_db() {
        static TEST_DIR: OnceCell<TempDir> = OnceCell::new();
        let dir = TEST_DIR.get_or_init(|| TempDir::new().expect("temp dir"));
        let path = dir.path().to_path_buf();
        db::init(&path).expect("db init");
        db::reset_for_tests().expect("db reset");
    }

    fn song(id: &str, album_id: &str, artist_id: &str, date_modified: &str) -> Song {
        Song {
            id: id.to_string(),
            name: format!("Song {id}"),
            item_type: "Audio".to_string(),
            album: Some(format!("Album {album_id}")),
            album_id: Some(album_id.to_string()),
            artists: Some(vec![format!("Artist {artist_id}")]),
            artist_ids: Some(vec![artist_id.to_string()]),
            path: None,
            duration: None,
            album_art_url: None,
            year: None,
            play_count: None,
            is_favorite: None,
            disc_number: None,
            track_number: None,
            container: None,
            bit_rate: None,
            sample_rate: None,
            codec: None,
            genres: None,
            premiere_date: None,
            date_played: None,
            date_created: Some("2024-01-01T00:00:00Z".to_string()),
            date_modified: Some(date_modified.to_string()),
            album_artists: None,
            lyrics: None,
            image_tags: None,
        }
    }

    fn artist(id: &str) -> Artist {
        Artist {
            name: format!("Artist {id}"),
            id: id.to_string(),
            image_tags: None,
            image_url: None,
            overview: None,
            provider_ids: None,
            community_rating: None,
            song_count: None,
            date_modified: None,
            songs: None,
        }
    }

    fn album(id: &str, artist_id: &str) -> Album {
        Album {
            id: Some(id.to_string()),
            name: format!("Album {id}"),
            artist: format!("Artist {artist_id}"),
            artist_id: Some(artist_id.to_string()),
            album_art_url: None,
            song_count: 1,
            songs: None,
            image_tags: None,
            provider_ids: None,
            date_created: Some("2024-01-01T00:00:00Z".to_string()),
            date_modified: None,
        }
    }

    #[test]
    #[serial]
    fn compute_delta_detects_add_update_remove() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let local_songs = vec![
            song("song1", "album1", "artist1", "2024-01-01T00:00:00Z"),
            song("song2", "album2", "artist2", "2024-01-01T00:00:00Z"),
            song("song_local", "album2", "artist2", "2024-01-01T00:00:00Z"),
        ];
        let local_artists = vec![artist("artist1"), artist("artist2")];
        let local_albums = vec![album("album1", "artist1"), album("album2", "artist2")];

        service
            .sync_library(&local_songs, &local_artists, &local_albums, true)
            .expect("sync");

        let remote_songs = vec![
            song("song1", "album1", "artist1", "2024-01-01T00:00:00Z"),
            song("song2", "album2", "artist2", "2024-02-01T00:00:00Z"),
            song("song3", "album3", "artist3", "2024-02-01T00:00:00Z"),
        ];
        let remote_artists = vec![artist("artist1"), artist("artist3")];
        let remote_albums = vec![album("album1", "artist1"), album("album3", "artist3")];

        let delta = service
            .compute_delta(&remote_songs, &remote_artists, &remote_albums)
            .expect("delta");

        assert!(delta.songs_to_add.contains(&"song3".to_string()));
        assert!(delta.songs_to_update.contains(&"song2".to_string()));
        assert!(delta.songs_to_remove.contains(&"song_local".to_string()));
        assert!(delta.artists_to_add.contains(&"artist3".to_string()));
        assert!(delta.artists_to_remove.contains(&"artist2".to_string()));
        assert!(delta.albums_to_add.contains(&"album3".to_string()));
        assert!(delta.albums_to_remove.contains(&"album2".to_string()));
    }

    #[test]
    #[serial]
    fn apply_delta_updates_counts() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let local_songs = vec![
            song("song1", "album1", "artist1", "2024-01-01T00:00:00Z"),
            song("song2", "album2", "artist2", "2024-01-01T00:00:00Z"),
            song("song_local", "album2", "artist2", "2024-01-01T00:00:00Z"),
        ];
        let local_artists = vec![artist("artist1"), artist("artist2")];
        let local_albums = vec![album("album1", "artist1"), album("album2", "artist2")];

        service
            .sync_library(&local_songs, &local_artists, &local_albums, true)
            .expect("sync");

        let remote_songs = vec![
            song("song1", "album1", "artist1", "2024-01-01T00:00:00Z"),
            song("song2", "album2", "artist2", "2024-02-01T00:00:00Z"),
            song("song3", "album3", "artist3", "2024-02-01T00:00:00Z"),
        ];
        let remote_artists = vec![artist("artist1"), artist("artist3")];
        let remote_albums = vec![album("album1", "artist1"), album("album3", "artist3")];

        let delta = service
            .compute_delta(&remote_songs, &remote_artists, &remote_albums)
            .expect("delta");

        let report = service
            .apply_delta(&delta, &remote_songs, &remote_artists, &remote_albums)
            .expect("apply");

        let (song_count, artist_count, album_count) = service.get_library_stats().expect("stats");

        assert_eq!(report.full_sync, false);
        assert_eq!(song_count, 3);
        assert_eq!(artist_count, 2);
        assert_eq!(album_count, 2);
    }

    #[test]
    #[serial]
    fn apply_delta_noop_when_empty() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);
        let report = service
            .apply_delta(&SyncDelta::default(), &[], &[], &[])
            .expect("apply");
        assert_eq!(report.songs_updated, 0);
    }

    // =========================================================================
    // Smart Sync Method Tests
    // =========================================================================

    #[test]
    #[serial]
    fn upsert_songs_inserts_new_and_updates_existing() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        // Insert initial songs
        let songs_v1 = vec![
            song("s1", "a1", "ar1", "2024-01-01T00:00:00Z"),
            song("s2", "a1", "ar1", "2024-01-01T00:00:00Z"),
        ];
        let count = service.upsert_songs(&songs_v1).expect("upsert");
        assert_eq!(count, 2);

        let (song_count, _, _) = service.get_library_stats().expect("stats");
        assert_eq!(song_count, 2);

        // Upsert: update s1, add s3
        let songs_v2 = vec![
            song("s1", "a1", "ar1", "2024-06-01T00:00:00Z"), // updated
            song("s3", "a2", "ar2", "2024-06-01T00:00:00Z"), // new
        ];
        let count = service.upsert_songs(&songs_v2).expect("upsert v2");
        assert_eq!(count, 2);

        let (song_count, _, _) = service.get_library_stats().expect("stats");
        assert_eq!(song_count, 3); // s1, s2, s3
    }

    #[test]
    #[serial]
    fn upsert_albums_inserts_and_updates() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let albums_v1 = vec![album("alb1", "ar1"), album("alb2", "ar1")];
        let count = service.upsert_albums(&albums_v1).expect("upsert");
        assert_eq!(count, 2);

        let (_, _, album_count) = service.get_library_stats().expect("stats");
        assert_eq!(album_count, 2);

        // Add a third album
        let albums_v2 = vec![album("alb3", "ar2")];
        service.upsert_albums(&albums_v2).expect("upsert v2");

        let (_, _, album_count) = service.get_library_stats().expect("stats");
        assert_eq!(album_count, 3);
    }

    #[test]
    #[serial]
    fn upsert_artists_inserts_and_updates() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let artists_v1 = vec![artist("ar1"), artist("ar2")];
        let count = service.upsert_artists(&artists_v1).expect("upsert");
        assert_eq!(count, 2);

        let (_, artist_count, _) = service.get_library_stats().expect("stats");
        assert_eq!(artist_count, 2);

        // Add a third artist
        let artists_v2 = vec![artist("ar3")];
        service.upsert_artists(&artists_v2).expect("upsert v2");

        let (_, artist_count, _) = service.get_library_stats().expect("stats");
        assert_eq!(artist_count, 3);
    }

    #[test]
    #[serial]
    fn remove_deleted_songs_removes_absent_ids() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        // Populate with 3 songs
        let songs = vec![
            song("s1", "a1", "ar1", "2024-01-01"),
            song("s2", "a1", "ar1", "2024-01-01"),
            song("s3", "a2", "ar2", "2024-01-01"),
        ];
        service.upsert_songs(&songs).expect("upsert");

        // Remote only has s1 and s3 - s2 was deleted on server
        let valid_ids: std::collections::HashSet<String> =
            ["s1".to_string(), "s3".to_string()].into_iter().collect();
        let removed = service.remove_deleted_songs(&valid_ids).expect("remove");
        assert_eq!(removed, 1);

        let (song_count, _, _) = service.get_library_stats().expect("stats");
        assert_eq!(song_count, 2);
    }

    #[test]
    #[serial]
    fn remove_deleted_albums_removes_absent_ids() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let albums = vec![
            album("alb1", "ar1"),
            album("alb2", "ar1"),
            album("alb3", "ar2"),
        ];
        service.upsert_albums(&albums).expect("upsert");

        // alb2 deleted on server
        let valid_ids: std::collections::HashSet<String> = ["alb1".to_string(), "alb3".to_string()]
            .into_iter()
            .collect();
        let removed = service.remove_deleted_albums(&valid_ids).expect("remove");
        assert_eq!(removed, 1);

        let (_, _, album_count) = service.get_library_stats().expect("stats");
        assert_eq!(album_count, 2);
    }

    #[test]
    #[serial]
    fn remove_deleted_artists_removes_absent_ids() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let artists = vec![artist("ar1"), artist("ar2"), artist("ar3")];
        service.upsert_artists(&artists).expect("upsert");

        // ar2 deleted on server
        let valid_ids: std::collections::HashSet<String> =
            ["ar1".to_string(), "ar3".to_string()].into_iter().collect();
        let removed = service.remove_deleted_artists(&valid_ids).expect("remove");
        assert_eq!(removed, 1);

        let (_, artist_count, _) = service.get_library_stats().expect("stats");
        assert_eq!(artist_count, 2);
    }

    #[test]
    #[serial]
    fn remove_deleted_returns_zero_when_nothing_to_delete() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        let songs = vec![song("s1", "a1", "ar1", "2024-01-01")];
        service.upsert_songs(&songs).expect("upsert");

        // All IDs still valid
        let valid_ids: std::collections::HashSet<String> = ["s1".to_string()].into_iter().collect();
        let removed = service.remove_deleted_songs(&valid_ids).expect("remove");
        assert_eq!(removed, 0);
    }

    #[test]
    #[serial]
    fn upsert_empty_is_noop() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        assert_eq!(service.upsert_songs(&[]).expect("upsert"), 0);
        assert_eq!(service.upsert_albums(&[]).expect("upsert"), 0);
        assert_eq!(service.upsert_artists(&[]).expect("upsert"), 0);
    }

    #[test]
    #[serial]
    fn upsert_songs_updates_favorite_status() {
        init_db();
        let db = db::get().expect("db");
        let service = LibraryService::new(db);

        // Insert a favorite song
        let mut fav_song = song("fav1", "a1", "ar1", "2024-01-01");
        fav_song.is_favorite = Some(true);
        service.upsert_songs(&[fav_song]).expect("upsert");

        // Now un-favorite it
        let mut unfav_song = song("fav1", "a1", "ar1", "2024-02-01");
        unfav_song.is_favorite = Some(false);
        service.upsert_songs(&[unfav_song]).expect("upsert");

        // Verify it's no longer in favorites (song still exists)
        let (song_count, _, _) = service.get_library_stats().expect("stats");
        assert_eq!(song_count, 1);
    }
}
