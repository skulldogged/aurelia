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
        let _ = write_txn.open_table(db::schema::PLAYER_STATE)?;
    }
    write_txn
        .commit()
        .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;

    DB.set(db)
        .map_err(|_| anyhow!("Database already initialized"))?;

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
}
