use crate::db;
use crate::models::{Album, Artist, Song};
use anyhow::{Result, anyhow};

use once_cell::sync::OnceCell;
use redb::{Database, ReadOnlyTable, ReadableDatabase, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use tracing::{debug, info, warn};

pub static DB: OnceCell<Database> = OnceCell::new();

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

    let write_txn = db
        .begin_write()
        .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
    {
        let _ = write_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
        let _ = write_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
        let _ = write_txn
            .open_table(ALBUMS_TABLE)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;

        // Initialize new index tables
        let _ = write_txn.open_table(db::schema::SONGS_BY_ALBUM)?;
        let _ = write_txn.open_table(db::schema::SONGS_BY_ARTIST)?;
        let _ = write_txn.open_table(db::schema::ALBUMS_BY_ARTIST)?;
        let _ = write_txn.open_table(db::schema::FAVORITES)?;
        let _ = write_txn.open_table(db::schema::SYNC_STATE)?;
        let _ = write_txn.open_table(db::schema::PLAYLISTS)?;
    }
    write_txn
        .commit()
        .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;

    // Use set() but ignore if already set (race condition handling)
    let _ = DB.set(db);

    info!("Database initialized successfully");
    Ok(())
}

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
    let db = DB.get().ok_or(anyhow!("Database not initialized"))?;

    // Use the LibraryService for sync operations
    let service = crate::domain::services::LibraryService::new(db);
    service
        .sync_library(songs, artists, albums, true)
        .map_err(|e| anyhow!("Sync failed: {}", e))?;

    Ok(())
}

/// Incremental sync - only updates items that have changed
/// Falls back to full sync if no previous sync data exists
pub fn sync_incremental(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<bool> {
    let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
    let service = crate::domain::services::LibraryService::new(db);

    // Check if we have any existing data to compare against
    let (song_count, artist_count, _album_count) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    // If database is empty, fall back to full sync
    if song_count == 0 && artist_count == 0 {
        info!("No existing library data found, performing full sync");
        service
            .sync_library(songs, artists, albums, true)
            .map_err(|e| anyhow!("Full sync failed: {}", e))?;
        return Ok(true); // true = was full sync
    }

    // Compute what changed
    let delta = service
        .compute_delta(songs, artists, albums)
        .map_err(|e| anyhow!("Failed to compute sync delta: {}", e))?;

    // If changes are too large (>50% of library), do full sync instead
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

    // Apply incremental changes
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

    Ok(false) // false = was incremental sync
}

/// Songs-only incremental sync - for hybrid lazy-load approach
/// Artists/albums are fetched on-demand when user visits detail pages
pub fn sync_songs_only(songs: &[Song]) -> Result<bool> {
    let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
    let service = crate::domain::services::LibraryService::new(db);

    // Check if we have any existing data
    let (song_count, _, _) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    // If database is empty, do full songs sync
    if song_count == 0 {
        info!("No existing song data found, performing full songs sync");
        service
            .sync_library(songs, &[], &[], true)
            .map_err(|e| anyhow!("Songs sync failed: {}", e))?;
        return Ok(true);
    }

    // Compute song-only delta
    let delta = service
        .compute_delta(songs, &[], &[])
        .map_err(|e| anyhow!("Failed to compute song delta: {}", e))?;

    // Apply song changes only
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

// Legacy implementation kept for reference (can be removed later)
#[allow(dead_code)]
fn sync_all_legacy(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<()> {
    let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
    let write_txn = db
        .begin_write()
        .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
    {
        // Sync songs
        let mut songs_table = write_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
        clear_table(&mut songs_table)?;

        // Sync and build indexes
        let mut songs_by_album = write_txn.open_table(db::schema::SONGS_BY_ALBUM)?;
        let mut songs_by_artist = write_txn.open_table(db::schema::SONGS_BY_ARTIST)?;
        let mut favorites = write_txn.open_table(db::schema::FAVORITES)?;

        for item in songs {
            let id = item.id.clone();
            let encoded =
                postcard::to_stdvec(item).map_err(|e| anyhow!("Failed to encode song: {}", e))?;
            songs_table
                .insert(id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert song into table: {}", e))?;

            // Build indexes
            if let Some(album_id) = &item.album_id {
                songs_by_album.insert((album_id.as_str(), id.as_str()), ())?;
            }

            if let Some(artist_ids) = &item.artist_ids {
                for artist_id in artist_ids {
                    songs_by_artist.insert((artist_id.as_str(), id.as_str()), ())?;
                }
            }

            if let Some(true) = item.is_favorite {
                let timestamp = chrono::Utc::now().to_rfc3339();
                let encoded_ts = postcard::to_stdvec(&timestamp)?;
                favorites.insert(id.as_str(), encoded_ts.as_slice())?;
            }
        }

        // Sync artists
        let mut artists_table = write_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
        clear_table(&mut artists_table)?;
        for item in artists {
            let id = item.id.clone();
            let encoded =
                postcard::to_stdvec(item).map_err(|e| anyhow!("Failed to encode artist: {}", e))?;
            artists_table
                .insert(id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert artist into table: {}", e))?;
        }

        // Sync albums
        let mut albums_table = write_txn
            .open_table(ALBUMS_TABLE)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
        clear_table(&mut albums_table)?;

        let mut albums_by_artist = write_txn.open_table(db::schema::ALBUMS_BY_ARTIST)?;

        let mut albums_processed: Vec<Album> = Vec::new();
        for album in albums {
            let mut new_album = album.clone();
            if new_album.id.is_none() {
                let generated_id = format!("{}-{}", new_album.artist, new_album.name);
                warn!(
                    "Album '{}' has no ID, generating one: {}",
                    new_album.name, generated_id
                );
                new_album.id = Some(generated_id);
            }
            albums_processed.push(new_album);
        }
        for item in &albums_processed {
            let id = item.id.clone().unwrap_or_default();
            let encoded =
                postcard::to_stdvec(item).map_err(|e| anyhow!("Failed to encode album: {}", e))?;
            albums_table
                .insert(id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert album into table: {}", e))?;

            // Build album-artist index
            if let Some(artist_id) = &item.artist_id {
                albums_by_artist.insert((artist_id.as_str(), id.as_str()), ())?;
            }
        }
    }
    write_txn
        .commit()
        .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
    Ok(())
}

pub mod songs {
    use super::*;

    pub fn get_all() -> Result<Vec<Song>> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
        get_all_items(&table)
    }

    pub fn update_favorite_status(song_id: &str, is_favorite: bool) -> Result<()> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
        let repo = crate::db::repositories::SongRepository::new(db);
        repo.update_favorite_status(song_id, is_favorite)
    }

    pub fn clear() -> Result<()> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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

pub mod artists {
    use super::*;

    pub fn get_all() -> Result<Vec<Artist>> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
        get_all_items(&table)
    }

    pub fn clear() -> Result<()> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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

    /// Get a single artist by ID from cache
    pub fn get_by_id(artist_id: &str) -> Result<Option<Artist>> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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

    /// Cache a single artist (upsert)
    pub fn cache(artist: &Artist) -> Result<()> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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

pub mod albums {
    use super::*;

    pub fn get_all() -> Result<Vec<Album>> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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

    /// Get a single album by ID from cache
    pub fn get_by_id(album_id: &str) -> Result<Option<Album>> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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

    /// Cache a single album (upsert)
    pub fn cache(album: &Album) -> Result<()> {
        let db = DB.get().ok_or(anyhow!("Database not initialized"))?;
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
