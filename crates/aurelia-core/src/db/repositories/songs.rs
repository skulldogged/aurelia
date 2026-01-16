use crate::db::schema::*;
use crate::models::Song;
use anyhow::Result;
use redb::{Database, ReadableDatabase, ReadableTable};

pub struct SongRepository {
    db: &'static Database,
}

impl SongRepository {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Result<Option<Song>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SONGS)?;

        if let Some(bytes) = table.get(id)? {
            let song = postcard::from_bytes(bytes.value())?;
            Ok(Some(song))
        } else {
            Ok(None)
        }
    }

    pub fn get_all(&self) -> Result<Vec<Song>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SONGS)?;

        let songs: Vec<Song> = table
            .iter()?
            .filter_map(|result| {
                result.ok().and_then(|(_, bytes)| {
                    let song = postcard::from_bytes(bytes.value()).ok()?;
                    Some(song)
                })
            })
            .collect();

        Ok(songs)
    }

    pub fn insert(&self, id: &str, item: &Song) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SONGS)?;
            let encoded = postcard::to_stdvec(item)?;
            table.insert(id, encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn upsert_with_indexes(&self, song: &Song) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            // Insert/update song
            let mut songs_table = write_txn.open_table(SONGS)?;
            let encoded = postcard::to_stdvec(song)?;
            songs_table.insert(song.id.as_str(), encoded.as_slice())?;

            // Update album index
            if let Some(album_id) = &song.album_id {
                let mut album_index = write_txn.open_table(SONGS_BY_ALBUM)?;
                album_index.insert((album_id.as_str(), song.id.as_str()), ())?;
            }

            // Update artist indexes
            if let Some(artist_ids) = &song.artist_ids {
                let mut artist_index = write_txn.open_table(SONGS_BY_ARTIST)?;
                for artist_id in artist_ids {
                    artist_index.insert((artist_id.as_str(), song.id.as_str()), ())?;
                }
            }

            // Update favorite status if applicable
            if let Some(true) = song.is_favorite {
                let mut favorites = write_txn.open_table(FAVORITES)?;
                let timestamp = chrono::Utc::now().to_rfc3339();
                let encoded_ts = postcard::to_stdvec(&timestamp)?;
                favorites.insert(song.id.as_str(), encoded_ts.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_by_album(&self, album_id: &str) -> Result<Vec<Song>> {
        let read_txn = self.db.begin_read()?;
        let index = read_txn.open_table(SONGS_BY_ALBUM)?;
        let songs_table = read_txn.open_table(SONGS)?;

        let mut songs = Vec::new();

        let range_start = (album_id, "");
        let range_end = (album_id, "\u{10ffff}");

        for result in index.range(range_start..=range_end)? {
            let (key, _) = result?;
            let (_, song_id) = key.value();
            if let Some(bytes) = songs_table.get(song_id)? {
                let song = postcard::from_bytes(bytes.value())?;
                songs.push(song);
            }
        }

        Ok(songs)
    }

    pub fn get_by_artist(&self, artist_id: &str) -> Result<Vec<Song>> {
        let read_txn = self.db.begin_read()?;
        let index = read_txn.open_table(SONGS_BY_ARTIST)?;
        let songs_table = read_txn.open_table(SONGS)?;

        let mut songs = Vec::new();

        let range_start = (artist_id, "");
        let range_end = (artist_id, "\u{10ffff}");

        for result in index.range(range_start..=range_end)? {
            let (key, _) = result?;
            let (_, song_id) = key.value();
            if let Some(bytes) = songs_table.get(song_id)? {
                let song = postcard::from_bytes(bytes.value())?;
                songs.push(song);
            }
        }

        Ok(songs)
    }

    pub fn get_favorites(&self) -> Result<Vec<Song>> {
        let read_txn = self.db.begin_read()?;
        let favorites = read_txn.open_table(FAVORITES)?;
        let songs_table = read_txn.open_table(SONGS)?;

        let mut result = Vec::new();
        for item in favorites.iter()? {
            let (id, _) = item?;
            if let Some(bytes) = songs_table.get(id.value())? {
                let song = postcard::from_bytes(bytes.value())?;
                result.push(song);
            }
        }

        Ok(result)
    }

    pub fn update_favorite_status(&self, song_id: &str, is_favorite: bool) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            // Update song's favorite status
            let mut songs_table = write_txn.open_table(SONGS)?;
            let song_bytes = songs_table
                .get(song_id)?
                .map(|bytes| bytes.value().to_vec());

            if let Some(bytes) = song_bytes {
                let mut song: Song = postcard::from_bytes(&bytes)?;
                song.is_favorite = Some(is_favorite);
                let encoded = postcard::to_stdvec(&song)?;
                songs_table.insert(song_id, encoded.as_slice())?;
            }

            // Update favorites index
            let mut favorites = write_txn.open_table(FAVORITES)?;
            if is_favorite {
                let timestamp = chrono::Utc::now().to_rfc3339();
                let encoded_ts = postcard::to_stdvec(&timestamp)?;
                favorites.insert(song_id, encoded_ts.as_slice())?;
            } else {
                favorites.remove(song_id)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut songs_table = write_txn.open_table(SONGS)?;
            let mut album_index = write_txn.open_table(SONGS_BY_ALBUM)?;
            let mut artist_index = write_txn.open_table(SONGS_BY_ARTIST)?;

            // Clear main table
            let mut keys = Vec::new();
            for item in songs_table.iter()? {
                let (key, _) = item?;
                keys.push(key.value().to_string());
            }

            for key in keys {
                songs_table.remove(key.as_str())?;
            }

            // Clear indexes
            let mut album_keys = Vec::new();
            for item in album_index.iter()? {
                let (key, _) = item?;
                let (album_id, song_id) = key.value();
                album_keys.push((album_id.to_string(), song_id.to_string()));
            }

            for (album_id, song_id) in album_keys {
                album_index.remove((album_id.as_str(), song_id.as_str()))?;
            }

            let mut artist_keys = Vec::new();
            for item in artist_index.iter()? {
                let (key, _) = item?;
                let (artist_id, song_id) = key.value();
                artist_keys.push((artist_id.to_string(), song_id.to_string()));
            }

            for (artist_id, song_id) in artist_keys {
                artist_index.remove((artist_id.as_str(), song_id.as_str()))?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
