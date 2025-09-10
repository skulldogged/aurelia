//! Jellyfin API service client

use crate::error::{AppError, AppResult};
use crate::models::{
    auth::{JellyfinAuthResponse, LoginResponse},
    jellyfin::JellyfinLyrics,
    Artist, NameIdPair, Song,
};
use crate::utils;
use reqwest::Client;
use serde_json;
use std::collections::HashMap;

/// Jellyfin API client
pub struct JellyfinClient {
    client: Client,
    server_url: String,
    token: Option<String>,
}

impl JellyfinClient {
    /// Create a new Jellyfin client
    pub fn new(server_url: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            token: None,
        }
    }

    /// Create a new authenticated Jellyfin client
    pub fn with_auth(server_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            token: Some(token),
        }
    }

    /// Set the authentication token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Get the server URL
    pub fn get_server_url(&self) -> &str {
        &self.server_url
    }

    /// Get the HTTP client (for internal use)
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Get the authorization header value
    pub fn get_auth_header(&self) -> String {
        match &self.token {
            Some(token) => format!("MediaBrowser Token=\"{}\"", token),
            None => utils::build_jellyfin_auth_header(),
        }
    }

    /// Authenticate user with Jellyfin server
    pub async fn authenticate(&self, username: &str, password: &str) -> AppResult<LoginResponse> {
        let login_url = utils::build_jellyfin_url(&self.server_url, "/Users/AuthenticateByName");

        let response = self
            .client
            .post(&login_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "Username": username,
                "Pw": password
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Auth(format!(
                "Login failed: HTTP {}",
                response.status()
            )));
        }

        let auth_response: JellyfinAuthResponse = response.json().await?;
        Ok(LoginResponse {
            token: auth_response.access_token,
            user_id: auth_response.user.id,
        })
    }

    /// Get music library items
    pub async fn get_music_library(&self, user_id: &str) -> AppResult<Vec<Song>> {
        let library_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/Users/{}/Items?IncludeItemTypes=Audio&Recursive=true&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate,AlbumArtists,MediaStreams,DateCreated,DateLastSaved,DateLastMediaAdded,ArtistItems", user_id)
        );

        let response = self
            .client
            .get(&library_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch library: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_music_items(response_json)
    }

    /// Get recently played music items
    pub async fn get_recently_played(&self, user_id: &str) -> AppResult<Vec<Song>> {
        let library_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/Users/{}/Items?IncludeItemTypes=Audio&Recursive=true&Filters=IsPlayed&SortBy=DatePlayed&SortOrder=Descending&Limit=20&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate,AlbumArtists,MediaStreams,DateCreated,DateLastSaved,DateLastMediaAdded", user_id)
        );

        let response = self
            .client
            .get(&library_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch recently played: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_music_items(response_json)
    }

    /// Parse music items from Jellyfin API response
    fn parse_music_items(&self, response_json: serde_json::Value) -> AppResult<Vec<Song>> {
        let items = response_json["Items"]
            .as_array()
            .ok_or_else(|| AppError::ApiParse("Invalid response format".to_string()))?;

        let mut music_items = Vec::new();

        for item in items {
            let music_item = self.parse_single_music_item(item)?;
            music_items.push(music_item);
        }

        Ok(music_items)
    }

    /// Parse a single music item from JSON with manual field mapping
    fn parse_single_music_item(&self, item: &serde_json::Value) -> AppResult<Song> {
        // Extract all the fields manually with proper camelCase naming
        let id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing item ID".to_string()))?
            .to_string();

        let name = item["Name"].as_str().unwrap_or("").to_string();

        let item_type = item["Type"].as_str().unwrap_or("").to_string();

        let album = item["Album"].as_str().map(|s| s.to_string());
        let album_id = item["AlbumId"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| item["ParentId"].as_str().map(|s| s.to_string()));
        let path = item["Path"].as_str().map(|s| s.to_string());

        // Handle RunTimeTicks -> duration conversion
        let duration = item["RunTimeTicks"]
            .as_i64()
            .map(|ticks| ticks as f64 / 10_000_000.0);

        // Handle UserData fields
        let user_data = &item["UserData"];
        let play_count = user_data["PlayCount"]
            .as_i64()
            .and_then(|n| n.try_into().ok());
        let is_favorite = user_data["IsFavorite"].as_bool();
        let date_played = user_data["LastPlayedDate"].as_str().map(String::from);

        // Handle other fields
        let year = item["ProductionYear"]
            .as_i64()
            .and_then(|n| n.try_into().ok());
        let track_number = item["IndexNumber"].as_i64().and_then(|n| n.try_into().ok());
        let premiere_date = item["PremiereDate"].as_str().map(|s| s.to_string());
        let date_created = item["DateCreated"].as_str().map(|s| s.to_string());

        // Handle MediaStreams for audio properties
        let (bit_rate, sample_rate, codec) = if let Some(streams) = item["MediaStreams"].as_array()
        {
            if let Some(audio_stream) = streams.iter().find(|s| s["Type"] == "Audio") {
                (
                    audio_stream["BitRate"]
                        .as_i64()
                        .and_then(|n| n.try_into().ok()),
                    audio_stream["SampleRate"]
                        .as_i64()
                        .and_then(|n| n.try_into().ok()),
                    audio_stream["Codec"].as_str().map(|s| s.to_string()),
                )
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        // Extract complex fields that need special handling
        let album_art_url = self.extract_album_art_url(item, &id);
        let artists = self.extract_artists(item);
        let artist_ids = self.extract_artist_ids(item);
        let genres = self.extract_genres(item);
        let album_artists = self.extract_album_artists(item);

        let container = item["Path"].as_str().and_then(|p| {
            std::path::Path::new(p)
                .extension()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_lowercase())
        });

        // Create the Song struct with camelCase field names
        let song = Song {
            id,
            name: name.clone(),
            item_type,
            album,
            album_id,
            artists,
            artist_ids,
            path,
            duration,
            album_art_url: album_art_url.clone(),
            year,
            play_count,
            is_favorite,
            track_number,
            container,
            bit_rate,
            sample_rate,
            codec,
            genres,
            premiere_date,
            date_played,
            date_created,
            album_artists,
            lyrics: None,
        };

        Ok(song)
    }

    /// Extract album artwork URL from item
    fn extract_album_art_url(&self, item: &serde_json::Value, item_id: &str) -> Option<String> {
        let image_id = item["AlbumId"].as_str().unwrap_or(item_id);

        if let Some(tags) = item["ImageTags"].as_object() {
            if tags.contains_key("Primary") {
                let base_url = format!(
                    "{}/Items/{}/Images/Primary",
                    self.server_url.trim_end_matches('/'),
                    image_id
                );

                // Only include api_key parameter if we have a token
                if let Some(token) = &self.token {
                    return Some(format!("{}?api_key={}", base_url, token));
                } else {
                    // If no token, try without api_key parameter (may work for public images)
                    return Some(base_url);
                }
            }
        }
        None
    }

    /// Extract artists from item
    fn extract_artists(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        item["Artists"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
    }

    /// Extract artist IDs from item
    fn extract_artist_ids(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        item["ArtistItems"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v["Id"].as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
    }

    /// Extract genres from item
    fn extract_genres(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        item["Genres"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
    }

    /// Extract album artists from item
    fn extract_album_artists(&self, item: &serde_json::Value) -> Option<Vec<NameIdPair>> {
        item["AlbumArtists"]
            .as_array()
            .and_then(|arr| serde_json::from_value(serde_json::Value::Array(arr.clone())).ok())
            .filter(|v: &Vec<NameIdPair>| !v.is_empty())
    }

    /// Get artist details
    pub async fn get_artist_details(&self, user_id: &str, artist_id: &str) -> AppResult<Artist> {
        let artist_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Users/{}/Items/{}?Fields=ImageTags,Overview,ProviderIds,CommunityRating",
                user_id, artist_id
            ),
        );

        let response = self
            .client
            .get(&artist_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch artist details: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_single_artist(&response_json)
    }

    /// Get all artists
    pub async fn get_all_artists(&self) -> AppResult<Vec<Artist>> {
        let artists_url = utils::build_jellyfin_url(
            &self.server_url,
            "/Artists?Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,SongCount",
        );

        let response = self
            .client
            .get(&artists_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch artists: HTTP {}",
                response.status()
            )));
        }

        // First, get the raw response text to debug the casing issue
        let response_text = response.text().await?;

        // Parse the JSON manually to handle field mapping
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AppError::ApiParse(format!("Failed to parse artists JSON: {}", e)))?;

        let items = response_json["Items"]
            .as_array()
            .ok_or_else(|| AppError::ApiParse("Invalid artists response format".to_string()))?;

        let mut artists = Vec::new();

        for item in items {
            let artist = self.parse_single_artist(item)?;
            artists.push(artist);
        }

        Ok(artists)
    }

    /// Parse a single artist from JSON with manual field mapping
    fn parse_single_artist(&self, item: &serde_json::Value) -> AppResult<Artist> {
        let id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing artist ID".to_string()))?
            .to_string();

        let name = item["Name"].as_str().unwrap_or("").to_string();

        // Extract image tags for later processing
        let image_tags = item["ImageTags"].clone();

        // Generate image URL if available
        let image_url = if let Some(tags) = item["ImageTags"].as_object() {
            if tags.contains_key("Primary") {
                let base_url = format!(
                    "{}/Items/{}/Images/Primary",
                    self.server_url.trim_end_matches('/'),
                    id
                );

                // Only include api_key parameter if we have a token
                if let Some(token) = &self.token {
                    Some(format!("{}?api_key={}", base_url, token))
                } else {
                    // If no token, try without api_key parameter (may work for public images)
                    Some(base_url)
                }
            } else {
                None
            }
        } else {
            None
        };

        let overview = item["Overview"].as_str().map(|s| s.to_string());

        let provider_ids = item["ProviderIds"].as_object().map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<String, String>>()
        });

        let community_rating = item["CommunityRating"].as_f64().map(|f| f as f32);
        let song_count = item["SongCount"].as_i64().map(|n| n as i32);

        let artist = Artist {
            name,
            id,
            image_tags: Some(image_tags),
            image_url,
            overview,
            provider_ids,
            community_rating,
            song_count,
            songs: None,
        };

        Ok(artist)
    }

    /// Get lyrics for a track
    pub async fn get_lyrics(&self, item_id: &str) -> AppResult<Option<JellyfinLyrics>> {
        let lyrics_url =
            utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{}/Lyrics", item_id));

        let response = self
            .client
            .get(&lyrics_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None); // No lyrics available
        }

        let lyrics: JellyfinLyrics = response.json().await?;
        Ok(Some(lyrics))
    }

    /// Toggle favorite status for an item
    pub async fn toggle_favorite(
        &self,
        user_id: &str,
        item_id: &str,
        is_favorite: bool,
    ) -> AppResult<()> {
        let fav_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/Users/{}/FavoriteItems/{}", user_id, item_id),
        );

        let response = if is_favorite {
            self.client
                .post(&fav_url)
                .header("Authorization", self.get_auth_header())
                .send()
                .await?
        } else {
            self.client
                .delete(&fav_url)
                .header("Authorization", self.get_auth_header())
                .send()
                .await?
        };

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to toggle favorite status: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Get audio stream URL
    pub fn get_audio_stream_url(&self, item_id: &str, container: Option<&str>) -> String {
        let supports_seeking = utils::supports_seeking(container);
        let token_param = self.token.as_deref().unwrap_or("");

        println!("Item ID: {}", item_id);
        println!("Container: {:?}", container);
        println!("Supports seeking: {}", supports_seeking);

        if supports_seeking {
            format!(
                "{}/Audio/{}/stream?api_key={}&static=true",
                self.server_url.trim_end_matches('/'),
                item_id,
                token_param
            )
        } else {
            format!(
                "{}/Audio/{}/stream.aac?api_key={}",
                self.server_url.trim_end_matches('/'),
                item_id,
                token_param
            )
        }
    }
}
