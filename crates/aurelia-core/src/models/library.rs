use crate::models::{Album, Song};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HomeViewData {
    pub recently_played: Vec<Song>,
    pub recently_added: Vec<Album>,
    pub random_albums: Vec<Album>,
    pub featured_albums: Vec<Album>,
}

/// Home view sections used by mobile clients.
#[derive(Serialize, Deserialize, Clone, Debug, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct MobileHomeData {
    pub most_played: Vec<Song>,
    pub recently_played: Vec<Song>,
    pub recently_added: Vec<Album>,
    pub random_albums: Vec<Album>,
    pub featured_albums: Vec<Album>,
}
