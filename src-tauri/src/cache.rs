use crate::database;
use crate::models::{Album, Artist, Song};
use anyhow::Result;

pub fn sync_library(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<()> {
    database::sync_all(songs, artists, albums)
}

pub fn get_songs() -> Result<Vec<Song>> {
    database::songs::get_all()
}

pub fn get_artists() -> Result<Vec<Artist>> {
    database::artists::get_all()
}

pub fn get_albums() -> Result<Vec<Album>> {
    database::albums::get_all()
}

pub fn clear_cache() -> Result<()> {
    database::songs::clear()?;
    database::artists::clear()?;
    database::albums::clear()?;
    Ok(())
}

pub fn update_song_favorite_status(song_id: &str, is_favorite: bool) -> Result<()> {
    database::songs::update_favorite_status(song_id, is_favorite)
}