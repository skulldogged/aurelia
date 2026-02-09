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

pub fn save_setting(app_data_dir: PathBuf, key: &str, value: &str) -> Result<()> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let write_txn = database.begin_write()?;
    {
        let mut table = write_txn.open_table(crate::db::schema::SETTINGS)?;
        table.insert(key, value)?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn load_setting(app_data_dir: PathBuf, key: &str) -> Result<Option<String>> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let read_txn = database.begin_read()?;
    let table = read_txn.open_table(crate::db::schema::SETTINGS)?;
    Ok(table.get(key)?.map(|guard| guard.value().to_string()))
}

pub fn delete_setting(app_data_dir: PathBuf, key: &str) -> Result<()> {
    init_db(&app_data_dir)?;
    let database = db::get()?;
    let write_txn = database.begin_write()?;
    {
        let mut table = write_txn.open_table(crate::db::schema::SETTINGS)?;
        let _ = table.remove(key);
    }
    write_txn.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Album, Artist, Credentials, Song};
    use once_cell::sync::OnceCell;
    use serial_test::serial;
    use tempfile::TempDir;

    fn init_db() -> PathBuf {
        static TEST_DIR: OnceCell<TempDir> = OnceCell::new();
        let dir = TEST_DIR.get_or_init(|| TempDir::new().expect("temp dir"));
        let path = dir.path().to_path_buf();
        db::init(&path).expect("db init");
        db::reset_for_tests().expect("db reset");
        path
    }

    fn song(id: &str) -> Song {
        Song {
            id: id.to_string(),
            name: format!("Song {id}"),
            item_type: "Audio".to_string(),
            album: None,
            album_id: None,
            artists: None,
            artist_ids: None,
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
            date_created: None,
            date_modified: None,
            album_artists: None,
            lyrics: None,
            image_tags: None,
        }
    }

    #[test]
    #[serial]
    fn sync_and_load_songs() {
        let app_dir = init_db();
        let songs = vec![song("s1"), song("s2")];
        let artists: Vec<Artist> = Vec::new();
        let albums: Vec<Album> = Vec::new();

        sync_library(app_dir.clone(), &songs, &artists, &albums).expect("sync");
        let loaded = get_songs(app_dir).expect("load");
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    #[serial]
    fn save_and_clear_credentials() {
        let app_dir = init_db();
        let creds = Credentials {
            server_url: "http://localhost:8096".to_string(),
            username: "user".to_string(),
            token: "token".to_string(),
            user_id: "user-id".to_string(),
        };

        save_credentials(app_dir.clone(), &creds).expect("save");
        let loaded = load_credentials(app_dir.clone()).expect("load");
        let loaded = loaded.expect("credentials present");
        assert_eq!(loaded.server_url, creds.server_url);
        assert_eq!(loaded.username, creds.username);
        assert_eq!(loaded.token, creds.token);
        assert_eq!(loaded.user_id, creds.user_id);

        clear_credentials(app_dir.clone()).expect("clear");
        let cleared = load_credentials(app_dir).expect("load after clear");
        assert!(cleared.is_none());
    }
}
