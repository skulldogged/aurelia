use crate::models::{Album, Artist, Song};
use anyhow::Result;
use bincode::config::standard;
use bincode::{Decode, Encode};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use sled::{Db, Tree};
use tauri::{AppHandle, Manager};
use tracing::{debug, info, warn};

pub static DB: OnceCell<Db> = OnceCell::new();

fn open_tree(name: &str) -> Result<Tree> {
    debug!("Opening tree: {}", name);
    let db = DB.get().ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
    Ok(db.open_tree(name)?)
}

pub fn flush_db() -> Result<()> {
    if let Some(db) = DB.get() {
        debug!("Flushing database...");
        db.flush()?;
        info!("Database flushed successfully.");
    }
    Ok(())
}

pub fn init(app: &AppHandle) -> Result<()> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| anyhow::anyhow!("Failed to get app data directory: {}", e))?;

    info!("Database path: {:?}", app_data_dir);

    let db_path = app_data_dir.join("sled_db");
    debug!("Full database path: {:?}", db_path);

    // Ensure the directory exists
    std::fs::create_dir_all(&app_data_dir).map_err(|e| anyhow::anyhow!("Failed to create app data directory: {}", e))?;

    let db = sled::Config::default()
        .path(db_path)
        .flush_every_ms(Some(1000))
        .open()
        .map_err(|e| anyhow::anyhow!("Failed to open sled database: {}", e))?;

    DB.set(db).map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    open_tree("songs")?;
    open_tree("artists")?;
    open_tree("albums")?;
    info!("Database initialized successfully");
    Ok(())
}

fn upsert_item<T: Serialize + Encode>(tree: &Tree, item_id: &str, item: &T) -> Result<()> {
    let encoded = bincode::encode_to_vec(item, standard())?;
    tree.insert(item_id.as_bytes(), encoded)?;
    Ok(())
}

fn get_item<T: DeserializeOwned + for<'a> Decode<()>>(
    tree: &Tree,
    item_id: &str,
) -> Result<Option<T>> {
    match tree.get(item_id.as_bytes())? {
        Some(bytes) => {
            let (item, _) = bincode::decode_from_slice(&bytes, standard())?;
            Ok(Some(item))
        }
        None => Ok(None),
    }
}

fn get_all_items<T: DeserializeOwned + for<'a> Decode<()>>(tree: &Tree) -> Result<Vec<T>> {
    debug!("Getting all items from tree");
    let items: Vec<T> = tree.iter().map(|res| {
        let (_, bytes) = res?;
        let (item, _) = bincode::decode_from_slice(&bytes, standard())?;
        Ok(item)
    }).collect::<Result<Vec<T>>>()?;
    debug!("Retrieved {} items from tree", items.len());
    Ok(items)
}

fn clear_tree(tree: &Tree) -> Result<()> {
    tree.clear()?;
    Ok(())
}

pub fn sync_items<T: Serialize + Encode + Send + Sync + Clone>(
    tree_name: &str,
    items: &[T],
    get_id: impl Fn(&T) -> String + Send + Sync,
) -> Result<()> {
    debug!("Syncing {} items to tree '{}'", items.len(), tree_name);
    let tree = open_tree(tree_name)?;
    tree.clear()?;
    let mut batch = sled::Batch::default();
    for item in items {
        let id = get_id(item);
        let encoded = bincode::encode_to_vec(item, standard())?;
        batch.insert(id.as_bytes(), encoded);
    }
    tree.apply_batch(batch)?;
    info!("Synced {} items to tree '{}'", items.len(), tree_name);
    Ok(())
}

pub mod songs {
    use super::*;
    pub fn sync(songs: &[Song]) -> Result<()> {
        sync_items("songs", songs, |s| s.id.clone())
    }

    pub fn get_all() -> Result<Vec<Song>> {
        let tree = open_tree("songs")?;
        get_all_items(&tree)
    }

    pub fn update_favorite_status(song_id: &str, is_favorite: bool) -> Result<()> {
        let tree = open_tree("songs")?;
        if let Some(mut song) = get_item::<Song>(&tree, song_id)? {
            song.is_favorite = Some(is_favorite);
            upsert_item(&tree, song_id, &song)?;
        }
        Ok(())
    }

    pub fn clear() -> Result<()> {
        let tree = open_tree("songs")?;
        clear_tree(&tree)
    }
}

pub mod artists {
    use super::*;
    pub fn sync(artists: &[Artist]) -> Result<()> {
        sync_items("artists", artists, |a| a.id.clone())
    }

    pub fn get_all() -> Result<Vec<Artist>> {
        let tree = open_tree("artists")?;
        get_all_items(&tree)
    }

    pub fn clear() -> Result<()> {
        let tree = open_tree("artists")?;
        clear_tree(&tree)
    }
}

pub mod albums {
    use super::*;

    pub fn sync(albums: &[Album]) -> Result<()> {
        info!("Syncing {} albums to database", albums.len());

        let mut albums_processed: Vec<Album> = Vec::new();
        for album in albums {
            let mut new_album = album.clone();
            if new_album.id.is_none() {
                let generated_id = format!("{}-{}", new_album.artist, new_album.name);
                warn!("Album '{}' has no ID, generating one: {}", new_album.name, generated_id);
                new_album.id = Some(generated_id);
            }
            albums_processed.push(new_album);
        }

        sync_items("albums", &albums_processed, |a| a.id.clone().unwrap())
    }

    pub fn get_all() -> Result<Vec<Album>> {
        let tree = open_tree("albums")?;
        let albums = get_all_items(&tree)?;
        info!("Retrieved {} albums from database", albums.len());
        Ok(albums)
    }

    pub fn clear() -> Result<()> {
        let tree = open_tree("albums")?;
        clear_tree(&tree)
    }
}
