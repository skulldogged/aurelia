//! Music-related data models

use serde::{Deserialize, Serialize};
use specta::Type;

/// Generic name-ID pair used for artists and other entities
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NameIdPair {
    /// Display name
    #[serde(rename = "Name")]
    pub name: String,
    /// Unique identifier
    #[serde(rename = "Id")]
    pub id: String,
}

/// Music item representing a song or audio file
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct MusicItem {
    /// Unique identifier
    pub id: String,
    /// Song title
    pub name: String,
    /// Type of item (usually "Audio")
    pub item_type: String,
    /// Album name
    pub album: Option<String>,
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
}

/// Artist information
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct ArtistInfo {
    /// Artist name
    #[serde(rename = "Name")]
    pub name: String,
    /// Artist ID
    #[serde(rename = "Id")]
    pub id: String,
    /// Image tags (metadata about available images)
    #[serde(rename = "ImageTags")]
    #[specta(skip)]
    pub image_tags: Option<serde_json::Value>,
    /// URL to artist image
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    /// Artist biography/description
    #[serde(rename = "Overview")]
    pub overview: Option<String>,
    /// External provider IDs (MusicBrainz, etc.)
    #[serde(rename = "ProviderIds")]
    #[specta(skip)]
    pub provider_ids: Option<serde_json::Value>,
    /// Community rating
    #[serde(rename = "CommunityRating")]
    pub community_rating: Option<f32>,
}

/// Collection of artists
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct ArtistItem {
    /// List of artists
    #[serde(rename = "Items")]
    pub items: Vec<ArtistInfo>,
}

/// Album information with metadata
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct AlbumInfo {
    /// Album name
    pub name: String,
    /// Primary artist name
    pub artist: String,
    /// Primary artist ID
    #[serde(rename = "artistId")]
    pub artist_id: Option<String>,
    /// URL to album artwork
    #[serde(rename = "albumArtUrl")]
    pub album_art_url: Option<String>,
    /// Number of songs in album
    #[serde(rename = "songCount")]
    pub song_count: i32,
}

/// Album with all its songs
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct AlbumWithSongs {
    /// Album name
    pub name: String,
    /// Primary artist name
    pub artist: String,
    /// Primary artist ID
    #[serde(rename = "artistId")]
    pub artist_id: Option<String>,
    /// URL to album artwork
    #[serde(rename = "albumArtUrl")]
    pub album_art_url: Option<String>,
    /// Number of songs in album
    #[serde(rename = "songCount")]
    pub song_count: i32,
    /// List of songs in this album
    pub songs: Vec<MusicItem>,
}

/// Artist with all their songs
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct ArtistWithSongs {
    /// Artist ID
    pub id: String,
    /// Artist name
    pub name: String,
    /// Number of songs by this artist
    #[serde(rename = "songCount")]
    pub song_count: i32,
    /// URL to artist image
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    /// List of songs by this artist
    pub songs: Vec<MusicItem>,
}
