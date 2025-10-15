use crate::models::{Album, Artist, Song};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct LibraryData {
    pub songs: Vec<Song>,
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct HomeViewData {
    pub recently_added: Vec<Album>,
    pub random_albums: Vec<Album>,
    pub featured_albums: Vec<Album>,
    pub recently_played: Vec<Song>,
}
