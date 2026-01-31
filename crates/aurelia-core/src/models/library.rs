use crate::models::{Album, Artist, Song};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct LibraryData {
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub songs: Vec<Song>,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct HomeViewData {
    pub recently_played: Vec<Song>,
    pub recently_added: Vec<Song>,
    pub random_albums: Vec<Album>,
    pub featured_albums: Vec<Album>,
}
