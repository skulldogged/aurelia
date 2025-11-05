use crate::db::schema::*;
use crate::models::Artist;
use anyhow::Result;
use redb::{Database, ReadableDatabase, ReadableTable};

pub struct ArtistRepository {
    db: &'static Database,
}

impl ArtistRepository {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Result<Option<Artist>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ARTISTS)?;
        
        if let Some(bytes) = table.get(id)? {
            let (artist, _) = bincode::decode_from_slice(bytes.value(), bincode::config::standard())?;
            Ok(Some(artist))
        } else {
            Ok(None)
        }
    }

    pub fn get_all(&self) -> Result<Vec<Artist>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ARTISTS)?;
        
        let artists: Vec<Artist> = table
            .iter()?
            .filter_map(|result| {
                result.ok().and_then(|(_, bytes)| {
                    let (artist, _) = bincode::decode_from_slice(bytes.value(), bincode::config::standard()).ok()?;
                    Some(artist)
                })
            })
            .collect();
        
        Ok(artists)
    }

    pub fn insert(&self, id: &str, item: &Artist) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ARTISTS)?;
            let encoded = bincode::encode_to_vec(item, bincode::config::standard())?;
            table.insert(id, encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn upsert(&self, artist: &Artist) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ARTISTS)?;
            let encoded = bincode::encode_to_vec(artist, bincode::config::standard())?;
            table.insert(artist.id.as_str(), encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ARTISTS)?;
            let keys: Vec<String> = table
                .iter()?
                .filter_map(|result| result.ok())
                .map(|(key, _)| key.value().to_string())
                .collect();
            
            for key in keys {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
