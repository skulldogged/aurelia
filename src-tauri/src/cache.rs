use crate::database;
use crate::models::{Album, Artist, Song};
use anyhow::Result;

pub fn sync_library(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<()> {
    database::songs::sync(songs)?;
    database::artists::sync(artists)?;
    database::albums::sync(albums)?;
    Ok(())
}

pub fn cache_library(songs: &[Song]) -> Result<()> {
    database::songs::sync(songs)
}

pub fn cache_artists(artists: &[Artist]) -> Result<()> {
    database::artists::sync(artists)
}

pub fn cache_albums(albums: &[Album]) -> Result<()> {
    database::albums::sync(albums)
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
