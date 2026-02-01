use crate::db;
use crate::models::{Album, Artist, Credentials, Song};
use anyhow::{anyhow, Result};
use redb::ReadableDatabase;
use serde_json;
use std::path::PathBuf;

fn init_db(app_data_dir: &PathBuf) -> Result<()> {
    if db::DB.get().is_none() {
        db::init(app_data_dir)?;
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
    db::sync_all(songs, artists, albums)
}

pub fn get_songs(app_data_dir: PathBuf) -> Result<Vec<Song>> {
    init_db(&app_data_dir)?;
    db::songs::get_all()
}

pub fn clear_cache(app_data_dir: PathBuf) -> Result<()> {
    init_db(&app_data_dir)?;
    db::songs::clear()?;
    db::artists::clear()?;
    db::albums::clear()?;
    Ok(())
}

pub fn update_song_favorite_status(
    app_data_dir: PathBuf,
    song_id: &str,
    is_favorite: bool,
) -> Result<()> {
    init_db(&app_data_dir)?;
    db::songs::update_favorite_status(song_id, is_favorite)
}

pub fn get_sync_state(app_data_dir: PathBuf) -> Result<String> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let service = crate::domain::services::LibraryService::new(database);
    let state = service
        .get_sync_state()
        .map_err(|err| anyhow!("Failed to read sync state: {err}"))?;
    let encoded = serde_json::to_string(&state)?;
    Ok(encoded)
}

pub fn set_sync_state(app_data_dir: PathBuf, state_json: &str) -> Result<()> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let service = crate::domain::services::LibraryService::new(database);
    let state = serde_json::from_str(state_json)?;
    service
        .update_sync_state(&state)
        .map_err(|err| anyhow!("Failed to update sync state: {err}"))?;
    Ok(())
}

pub fn save_credentials(app_data_dir: PathBuf, credentials: &Credentials) -> Result<()> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let write_txn = database.begin_write()?;
    {
        let mut table = write_txn.open_table(crate::db::schema::CREDENTIALS)?;
        let json = serde_json::to_vec(credentials)?;
        table.insert("main", json.as_slice())?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn load_credentials(app_data_dir: PathBuf) -> Result<Option<Credentials>> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let read_txn = database.begin_read()?;
    let table = read_txn.open_table(crate::db::schema::CREDENTIALS)?;
    if let Some(guard) = table.get("main")? {
        let credentials: Credentials = serde_json::from_slice(guard.value())?;
        Ok(Some(credentials))
    } else {
        Ok(None)
    }
}

pub fn clear_credentials(app_data_dir: PathBuf) -> Result<()> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let write_txn = database.begin_write()?;
    {
        let mut table = write_txn.open_table(crate::db::schema::CREDENTIALS)?;
        table.remove("main")?;
    }
    write_txn.commit()?;
    Ok(())
}
