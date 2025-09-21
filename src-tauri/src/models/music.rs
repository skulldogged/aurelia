//! Music-related data models

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Generic name-ID pair used for artists and other entities
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq, Hash)]
#[specta(rename_all = "camelCase")]
pub struct NameIdPair {
    /// Display name
    pub name: String,
    /// Unique identifier
    pub id: String,
}

/// Wrapper for API responses that contain an Items array
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemsResponse<T> {
    #[serde(rename = "Items")]
    pub items: Vec<T>,
}

/// Song representing a music track or audio file
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq)]
#[specta(rename_all = "camelCase")]
pub struct Song {
    /// Unique identifier
    pub id: String,
    /// Song title
    pub name: String,
    /// Type of item (usually "Audio")
    #[serde(rename = "itemType")]
    pub item_type: String,
    /// Album name
    pub album: Option<String>,
    /// Album ID
    #[serde(rename = "albumId")]
    pub album_id: Option<String>,
    /// List of artist names
    pub artists: Option<Vec<String>>,
    /// List of artist IDs corresponding to artists
    #[serde(rename = "artistIds")]
    pub artist_ids: Option<Vec<String>>,
    /// File path
    pub path: Option<String>,
    /// Duration in seconds
    pub duration: Option<f64>,
    /// URL to album artwork
    #[serde(rename = "albumArtUrl")]
    pub album_art_url: Option<String>,
    /// Release year
    pub year: Option<i32>,
    /// Number of times played
    #[serde(rename = "playCount")]
    pub play_count: Option<i32>,
    /// Whether this item is marked as favorite
    #[serde(rename = "isFavorite")]
    pub is_favorite: Option<bool>,
    /// Track number in album
    #[serde(rename = "trackNumber")]
    pub track_number: Option<i32>,
    /// Audio container/format
    pub container: Option<String>,
    /// Audio bitrate
    #[serde(rename = "bitRate")]
    pub bit_rate: Option<i32>,
    /// Audio sample rate
    #[serde(rename = "sampleRate")]
    pub sample_rate: Option<i32>,
    /// Audio codec
    pub codec: Option<String>,
    /// Music genres
    pub genres: Option<Vec<String>>,
    /// Premiere/release date
    #[serde(rename = "premiereDate")]
    pub premiere_date: Option<String>,
    /// Last played date
    #[serde(rename = "datePlayed")]
    pub date_played: Option<String>,
    /// Date created (when added to server)
    #[serde(rename = "dateCreated")]
    pub date_created: Option<String>,
    /// Album artists (different from track artists)
    #[serde(rename = "albumArtists")]
    pub album_artists: Option<Vec<NameIdPair>>,
    /// Song lyrics
    pub lyrics: Option<String>,
    /// Image tags
    #[serde(rename = "imageTags")]
    pub image_tags: Option<HashMap<String, String>>,
}

impl Eq for Song {}

impl Hash for Song {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Consolidated artist type with all information
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    /// Artist name
    pub name: String,
    /// Artist ID
    pub id: String,
    /// Image tags (metadata about available images)
    pub image_tags: Option<HashMap<String, String>>,
    /// URL to artist image
    pub image_url: Option<String>,
    /// Artist biography/description
    pub overview: Option<String>,
    /// External provider IDs (`MusicBrainz`, etc.)
    pub provider_ids: Option<HashMap<String, String>>,
    /// Community rating
    pub community_rating: Option<f64>,
    /// Number of songs by this artist
    pub song_count: Option<i64>,
    /// Optional list of songs by this artist (only populated when needed)
    pub songs: Option<Vec<Song>>,
}

impl Eq for Artist {}

impl Hash for Artist {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Consolidated album type with all information
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[specta(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct Album {
    /// Album ID from Jellyfin
    pub id: Option<String>,
    /// Album name
    pub name: String,
    /// Primary artist name
    pub artist: String,
    /// Primary artist ID
    pub artist_id: Option<String>,
    /// URL to album artwork
    pub album_art_url: Option<String>,
    /// Number of songs in album
    pub song_count: i64,
    /// Optional list of songs in this album (only populated when needed)
    pub songs: Option<Vec<Song>>,
    /// Image tags
    #[serde(rename = "imageTags")]
    pub image_tags: Option<HashMap<String, String>>,
}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.artist == other.artist
            && self.artist_id == other.artist_id
            && self.album_art_url == other.album_art_url
            && self.song_count == other.song_count
            && self.songs == other.songs
            && self.image_tags == other.image_tags
    }
}

impl Eq for Album {}

impl Hash for Album {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
