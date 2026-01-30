pub mod repositories;
pub mod schema;

pub use repositories::*;
pub use schema::*;

use crate::models::{Album, Artist, Song};
use anyhow::{Result, anyhow};
use once_cell::sync::OnceCell;
use redb::{Database, ReadOnlyTable, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use tracing::{debug, info};

pub static DB: OnceCell<Database> = OnceCell::new();

// Table definitions
const SONGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("songs");
const ARTISTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("artists");
const ALBUMS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("albums");

pub fn init(app_data_dir: &PathBuf) -> Result<()> {
    // If database is already initialized, just return Ok
    if DB.get().is_some() {
        debug!("Database already initialized, skipping");
        return Ok(());
    }

    info!("Database path: {:?}", app_data_dir);

    let db_path = app_data_dir.join("aurelia.redb");
    debug!("Full database path: {:?}", db_path);

    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| anyhow!("Failed to create app data directory: {}", e))?;

    let db = Database::create(&db_path).map_err(|e| anyhow!("Failed to create database: {}", e))?;

    // Initialize all tables
    let write_txn = db
        .begin_write()
        .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
    {
        // Primary tables
        let _ = write_txn.open_table(schema::SONGS)?;
        let _ = write_txn.open_table(schema::ARTISTS)?;
        let _ = write_txn.open_table(schema::ALBUMS)?;
        let _ = write_txn.open_table(schema::PLAYLISTS)?;

        // Index tables
        let _ = write_txn.open_table(schema::SONGS_BY_ALBUM)?;
        let _ = write_txn.open_table(schema::SONGS_BY_ARTIST)?;
        let _ = write_txn.open_table(schema::ALBUMS_BY_ARTIST)?;

        // Metadata tables
        let _ = write_txn.open_table(schema::FAVORITES)?;
        let _ = write_txn.open_table(schema::SYNC_STATE)?;
        let _ = write_txn.open_table(schema::CREDENTIALS)?;
    }
    write_txn.commit()?;

    // Use set() but ignore if already set (race condition handling)
    let _ = DB.set(db);

    info!("Database initialized successfully");
    Ok(())
}

pub fn get() -> Result<&'static Database> {
    DB.get().ok_or_else(|| anyhow!("Database not initialized"))
}

// ============================================================================
// Sync functions
// ============================================================================

fn get_all_items<T: DeserializeOwned>(table: &ReadOnlyTable<&str, &[u8]>) -> Result<Vec<T>> {
    debug!("Getting all items from table");
    let items: Vec<T> = table
        .iter()
        .map_err(|e| anyhow!("Failed to iterate over table: {}", e))?
        .map(|res| {
            let (_, bytes) = res.map_err(|e| anyhow!("Failed to get table item: {}", e))?;
            let item = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode table item: {}", e))?;
            Ok(item)
        })
        .collect::<Result<Vec<T>>>()?;
    debug!("Retrieved {} items from table", items.len());
    Ok(items)
}

pub(crate) fn clear_table(table: &mut redb::Table<&str, &[u8]>) -> Result<()> {
    let mut keys = Vec::new();
    for result in table
        .iter()
        .map_err(|e| anyhow!("Failed to iterate over table: {}", e))?
    {
        let (key, _) = result.map_err(|e| anyhow!("Failed to get table item: {}", e))?;
        keys.push(key.value().to_string());
    }
    for key in keys {
        table
            .remove(key.as_str())
            .map_err(|e| anyhow!("Failed to remove table item: {}", e))?;
    }
    Ok(())
}

pub fn sync_all(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<()> {
    let db = get()?;

    let service = crate::domain::services::LibraryService::new(db);
    service
        .sync_library(songs, artists, albums, true)
        .map_err(|e| anyhow!("Sync failed: {}", e))?;

    Ok(())
}

/// Incremental sync - only updates items that have changed
/// Falls back to full sync if no previous sync data exists
pub fn sync_incremental(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<bool> {
    let db = get()?;
    let service = crate::domain::services::LibraryService::new(db);

    let (song_count, artist_count, _album_count) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    if song_count == 0 && artist_count == 0 {
        info!("No existing library data found, performing full sync");
        service
            .sync_library(songs, artists, albums, true)
            .map_err(|e| anyhow!("Full sync failed: {}", e))?;
        return Ok(true);
    }

    let delta = service
        .compute_delta(songs, artists, albums)
        .map_err(|e| anyhow!("Failed to compute sync delta: {}", e))?;

    let total_items = songs.len() + artists.len() + albums.len();
    let threshold = total_items / 2;
    if delta.total_changes() > threshold {
        info!(
            "Delta too large ({} changes out of {} items), performing full sync",
            delta.total_changes(),
            total_items
        );
        service
            .sync_library(songs, artists, albums, true)
            .map_err(|e| anyhow!("Full sync failed: {}", e))?;
        return Ok(true);
    }

    if delta.is_empty() {
        info!("No changes detected, library is up to date");
    } else {
        info!(
            "Applying incremental sync: {} changes",
            delta.total_changes()
        );
        service
            .apply_delta(&delta, songs, artists, albums)
            .map_err(|e| anyhow!("Delta sync failed: {}", e))?;
    }

    Ok(false)
}

/// Songs-only incremental sync - for hybrid lazy-load approach
pub fn sync_songs_only(songs: &[Song]) -> Result<bool> {
    let db = get()?;
    let service = crate::domain::services::LibraryService::new(db);

    let (song_count, _, _) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    if song_count == 0 {
        info!("No existing song data found, performing full songs sync");
        service
            .sync_library(songs, &[], &[], true)
            .map_err(|e| anyhow!("Songs sync failed: {}", e))?;
        return Ok(true);
    }

    let delta = service
        .compute_delta(songs, &[], &[])
        .map_err(|e| anyhow!("Failed to compute song delta: {}", e))?;

    if delta.songs_to_add.is_empty()
        && delta.songs_to_remove.is_empty()
        && delta.songs_to_update.is_empty()
    {
        info!("No song changes detected, library is up to date");
    } else {
        info!(
            "Applying incremental song sync: {} adds, {} removes, {} updates",
            delta.songs_to_add.len(),
            delta.songs_to_remove.len(),
            delta.songs_to_update.len()
        );
        service
            .apply_delta(&delta, songs, &[], &[])
            .map_err(|e| anyhow!("Song delta sync failed: {}", e))?;
    }

    Ok(false)
}

// ============================================================================
// Songs submodule
// ============================================================================

pub mod songs {
    use super::*;
    use redb::ReadableDatabase;

    pub fn get_all() -> Result<Vec<Song>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
        get_all_items(&table)
    }

    pub fn get_by_id(song_id: &str) -> Result<Option<Song>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;

        if let Some(bytes) = table.get(song_id)? {
            let song: Song = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode song: {}", e))?;
            Ok(Some(song))
        } else {
            Ok(None)
        }
    }

    pub fn update_favorite_status(song_id: &str, is_favorite: bool) -> Result<()> {
        let db = crate::db::get()?;
        let repo = crate::db::repositories::SongRepository::new(db);
        repo.update_favorite_status(song_id, is_favorite)
    }

    pub fn clear() -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(SONGS_TABLE)
                .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
            clear_table(&mut table)?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Artists submodule
// ============================================================================

pub mod artists {
    use super::*;
    use redb::ReadableDatabase;

    pub fn get_all() -> Result<Vec<Artist>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
        get_all_items(&table)
    }

    pub fn clear() -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ARTISTS_TABLE)
                .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
            clear_table(&mut table)?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        Ok(())
    }

    pub fn get_by_id(artist_id: &str) -> Result<Option<Artist>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;

        if let Some(bytes) = table.get(artist_id)? {
            let artist: Artist = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode artist: {}", e))?;
            Ok(Some(artist))
        } else {
            Ok(None)
        }
    }

    pub fn cache(artist: &Artist) -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ARTISTS_TABLE)
                .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
            let encoded = postcard::to_stdvec(artist)
                .map_err(|e| anyhow!("Failed to encode artist: {}", e))?;
            table
                .insert(artist.id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert artist: {}", e))?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        debug!("Cached artist: {}", artist.name);
        Ok(())
    }
}

// ============================================================================
// Albums submodule
// ============================================================================

pub mod albums {
    use super::*;
    use redb::ReadableDatabase;

    pub fn get_all() -> Result<Vec<Album>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ALBUMS_TABLE)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
        let albums = get_all_items(&table)?;
        info!("Retrieved {} albums from database", albums.len());
        Ok(albums)
    }

    pub fn clear() -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ALBUMS_TABLE)
                .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
            clear_table(&mut table)?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        Ok(())
    }

    pub fn get_by_id(album_id: &str) -> Result<Option<Album>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ALBUMS_TABLE)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;

        if let Some(bytes) = table.get(album_id)? {
            let album: Album = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode album: {}", e))?;
            Ok(Some(album))
        } else {
            Ok(None)
        }
    }

    pub fn cache(album: &Album) -> Result<()> {
        let db = crate::db::get()?;
        let album_id = album.id.as_ref().ok_or(anyhow!("Album has no ID"))?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ALBUMS_TABLE)
                .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
            let encoded =
                postcard::to_stdvec(album).map_err(|e| anyhow!("Failed to encode album: {}", e))?;
            table
                .insert(album_id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert album: {}", e))?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        debug!("Cached album: {}", album.name);
        Ok(())
    }
}
