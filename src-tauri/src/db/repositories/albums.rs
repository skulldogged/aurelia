use crate::db::schema::*;
use crate::models::Album;
use anyhow::Result;
use redb::{Database, ReadableDatabase, ReadableTable};

pub struct AlbumRepository {
    db: &'static Database,
}

impl AlbumRepository {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Result<Option<Album>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ALBUMS)?;
        
        if let Some(bytes) = table.get(id)? {
            let (album, _) = bincode::decode_from_slice(bytes.value(), bincode::config::standard())?;
            Ok(Some(album))
        } else {
            Ok(None)
        }
    }

    pub fn get_all(&self) -> Result<Vec<Album>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ALBUMS)?;
        
        let albums: Vec<Album> = table
            .iter()?
            .filter_map(|result| {
                result.ok().and_then(|(_, bytes)| {
                    let (album, _) = bincode::decode_from_slice(bytes.value(), bincode::config::standard()).ok()?;
                    Some(album)
                })
            })
            .collect();
        
        Ok(albums)
    }

    pub fn insert(&self, id: &str, item: &Album) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ALBUMS)?;
            let encoded = bincode::encode_to_vec(item, bincode::config::standard())?;
            table.insert(id, encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn upsert_with_indexes(&self, album: &Album) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut albums_table = write_txn.open_table(ALBUMS)?;
            let encoded = bincode::encode_to_vec(album, bincode::config::standard())?;
            let album_id = album.id.clone().unwrap_or_else(|| album.name.clone());
            albums_table.insert(album_id.as_str(), encoded.as_slice())?;

            if let Some(artist_id) = &album.artist_id {
                let mut albums_by_artist = write_txn.open_table(ALBUMS_BY_ARTIST)?;
                albums_by_artist.insert((artist_id.as_str(), album_id.as_str()), ())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_by_artist(&self, artist_id: &str) -> Result<Vec<Album>> {
        let read_txn = self.db.begin_read()?;
        let index = read_txn.open_table(ALBUMS_BY_ARTIST)?;
        let albums_table = read_txn.open_table(ALBUMS)?;
        
        let mut albums = Vec::new();
        let range_start = (artist_id, "");
        let range_end = (artist_id, "\u{10ffff}");
        
        for result in index.range(range_start..=range_end)? {
            let (key, _) = result?;
            let (_, album_id) = key.value();
            if let Some(bytes) = albums_table.get(album_id)? {
                let (album, _) = bincode::decode_from_slice(bytes.value(), bincode::config::standard())?;
                albums.push(album);
            }
        }
        
        Ok(albums)
    }

    pub fn clear(&self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ALBUMS)?;
            let mut index = write_txn.open_table(ALBUMS_BY_ARTIST)?;
            
            let mut keys = Vec::new();
            for item in table.iter()? {
                let (key, _) = item?;
                keys.push(key.value().to_string());
            }
            
            for key in keys {
                table.remove(key.as_str())?;
            }
            
            let mut index_keys = Vec::new();
            for item in index.iter()? {
                let (key, _) = item?;
                let (artist_id, album_id) = key.value();
                index_keys.push((artist_id.to_string(), album_id.to_string()));
            }
            
            for (artist_id, album_id) in index_keys {
                index.remove((artist_id.as_str(), album_id.as_str()))?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
