use crate::db::schema::*;
use crate::domain::errors::DomainError;
use crate::domain::models::{SyncReport, SyncState};
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
            let (state, _) =
                bincode::decode_from_slice(bytes.value(), bincode::config::standard())
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
            let encoded = bincode::encode_to_vec(state, bincode::config::standard())
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
                let encoded = bincode::encode_to_vec(song, bincode::config::standard())
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
                    let encoded_ts = bincode::encode_to_vec(&timestamp, bincode::config::standard())
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                    favorites.insert(song.id.as_str(), encoded_ts.as_slice())?;
                }
            }

            // Sync artists
            let mut artists_table = write_txn.open_table(ARTISTS)?;
            for artist in artists {
                let encoded = bincode::encode_to_vec(artist, bincode::config::standard())
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                artists_table.insert(artist.id.as_str(), encoded.as_slice())?;
            }

            // Sync albums with indexes
            let mut albums_table = write_txn.open_table(ALBUMS)?;
            let mut albums_by_artist = write_txn.open_table(ALBUMS_BY_ARTIST)?;

            for album in albums {
                let album_id = album.id.clone().unwrap_or_else(|| {
                    format!("{}-{}", album.artist, album.name)
                });

                let encoded = bincode::encode_to_vec(album, bincode::config::standard())
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
                last_sync_version: None,
                song_count: songs.len() as u32,
                artist_count: artists.len() as u32,
                album_count: albums.len() as u32,
            };

            let mut sync_state_table = write_txn.open_table(SYNC_STATE)?;
            let encoded_state = bincode::encode_to_vec(&new_state, bincode::config::standard())
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

    fn clear_all_tables(
        &self,
        write_txn: &redb::WriteTransaction,
    ) -> Result<(), DomainError> {
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
}
