use crate::database;
use crate::db;
use crate::models::{Album, Artist, Credentials, Song};
use anyhow::{Result, anyhow};
use redb::ReadableDatabase;
use serde_json;
use std::path::PathBuf;

fn init_db(app_data_dir: &PathBuf) -> Result<()> {
    if database::DB.get().is_none() {
        database::init(app_data_dir)?;
    }
    Ok(())
}

pub fn sync_library(
    app_data_dir: PathBuf,
    songs: &[Song],
    artists: &[Artist],
    albums: &[Album],
) -> Result<()> {
    init_db(&app_data_dir)?;
    database::sync_all(songs, artists, albums)
}

pub fn get_songs(app_data_dir: PathBuf) -> Result<Vec<Song>> {
    init_db(&app_data_dir)?;
    database::songs::get_all()
}

pub fn get_artists(app_data_dir: PathBuf) -> Result<Vec<Artist>> {
    init_db(&app_data_dir)?;
    database::artists::get_all()
}

pub fn get_albums(app_data_dir: PathBuf) -> Result<Vec<Album>> {
    init_db(&app_data_dir)?;
    database::albums::get_all()
}

pub fn clear_cache(app_data_dir: PathBuf) -> Result<()> {
    init_db(&app_data_dir)?;
    database::songs::clear()?;
    database::artists::clear()?;
    database::albums::clear()?;
    Ok(())
}

pub fn update_song_favorite_status(
    app_data_dir: PathBuf,
    song_id: &str,
    is_favorite: bool,
) -> Result<()> {
    init_db(&app_data_dir)?;
    database::songs::update_favorite_status(song_id, is_favorite)
}

pub fn get_sync_state(app_data_dir: PathBuf) -> Result<String> {
    init_db(&app_data_dir)?;
    let db = database::DB
        .get()
        .ok_or_else(|| anyhow!("Database not initialized"))?;
    let service = crate::domain::services::LibraryService::new(db);
    let state = service
        .get_sync_state()
        .map_err(|err| anyhow!("Failed to read sync state: {err}"))?;
    let encoded = serde_json::to_string(&state)?;
    Ok(encoded)
}

pub fn set_sync_state(app_data_dir: PathBuf, state_json: &str) -> Result<()> {
    init_db(&app_data_dir)?;
    let db = database::DB
        .get()
        .ok_or_else(|| anyhow!("Database not initialized"))?;
    let service = crate::domain::services::LibraryService::new(db);
    let state = serde_json::from_str(state_json)?;
    service
        .update_sync_state(&state)
        .map_err(|err| anyhow!("Failed to update sync state: {err}"))?;
    Ok(())
}

const CREDENTIALS_KEY: &str = "current";

pub fn save_credentials(app_data_dir: PathBuf, credentials: &Credentials) -> Result<()> {
    init_db(&app_data_dir)?;
    let db = database::DB
        .get()
        .ok_or_else(|| anyhow!("Database not initialized"))?;

    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(db::schema::CREDENTIALS)?;
        let encoded = postcard::to_stdvec(credentials)
            .map_err(|e| anyhow!("Failed to encode credentials: {e}"))?;
        table.insert(CREDENTIALS_KEY, encoded.as_slice())?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn load_credentials(app_data_dir: PathBuf) -> Result<Option<Credentials>> {
    init_db(&app_data_dir)?;
    let db = database::DB
        .get()
        .ok_or_else(|| anyhow!("Database not initialized"))?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(db::schema::CREDENTIALS)?;

    match table.get(CREDENTIALS_KEY)? {
        Some(guard) => {
            let bytes: &[u8] = guard.value();
            let credentials: Credentials = postcard::from_bytes(bytes)
                .map_err(|e| anyhow!("Failed to decode credentials: {e}"))?;
            Ok(Some(credentials))
        }
        None => Ok(None),
    }
}

pub fn clear_credentials(app_data_dir: PathBuf) -> Result<()> {
    init_db(&app_data_dir)?;
    let db = database::DB
        .get()
        .ok_or_else(|| anyhow!("Database not initialized"))?;

    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(db::schema::CREDENTIALS)?;
        let _ = table.remove(CREDENTIALS_KEY);
    }
    write_txn.commit()?;
    Ok(())
}
