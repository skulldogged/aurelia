use crate::models::{Album, Artist, Song};
use crate::utils;
use anyhow::Result;
use bincode::config::standard;
use bincode::{Decode, Encode};
use once_cell::sync::Lazy;
use serde::{Serialize, de::DeserializeOwned};
use sled::{Db, Tree};
use tracing::info;

static DB: Lazy<Db> = Lazy::new(|| {
    let cache_dir = utils::get_app_data_dir().expect("Failed to get app data dir");
    let db_path = cache_dir.join("sled_db");
    sled::open(db_path).expect("Failed to open sled database")
});

fn open_tree(name: &str) -> Result<Tree> {
    Ok(DB.open_tree(name)?)
}

pub fn init() -> Result<()> {
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
    tree.iter()
        .map(|res| {
            let (_, bytes) = res?;
            let (item, _) = bincode::decode_from_slice(&bytes, standard())?;
            Ok(item)
        })
        .collect()
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
        sync_items("albums", albums, |a| a.id.clone().unwrap_or_default())
    }

    pub fn get_all() -> Result<Vec<Album>> {
        let tree = open_tree("albums")?;
        get_all_items(&tree)
    }

    pub fn clear() -> Result<()> {
        let tree = open_tree("albums")?;
        clear_tree(&tree)
    }
}
