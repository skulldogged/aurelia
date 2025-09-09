//! Jellyfin API service client

use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::utils;
use reqwest::Client;
use serde_json;

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

    /// Get the authorization header value
    fn get_auth_header(&self) -> String {
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
    pub async fn get_music_library(&self, user_id: &str) -> AppResult<Vec<MusicItem>> {
        let library_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/Users/{}/Items?IncludeItemTypes=Audio&Recursive=true&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate,AlbumArtists,MediaStreams,DateCreated", user_id)
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

    /// Parse music items from Jellyfin API response
    fn parse_music_items(&self, response_json: serde_json::Value) -> AppResult<Vec<MusicItem>> {
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

    /// Parse a single music item from JSON
    fn parse_single_music_item(&self, item: &serde_json::Value) -> AppResult<MusicItem> {
        let duration_ticks = item["RunTimeTicks"].as_i64();
        let duration_seconds = duration_ticks.map(|ticks| ticks as f64 / 10_000_000.0);

        let item_id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing item ID".to_string()))?
            .to_string();

        let album_art_url = self.extract_album_art_url(item, &item_id);

        // Extract artists
        let artists_vec = self.extract_artists(item);
        let artist_ids_vec = self.extract_artist_ids(item);
        let genres_vec = self.extract_genres(item);
        let album_artists = self.extract_album_artists(item);

        let container = item["Path"].as_str().and_then(|p| {
            std::path::Path::new(p)
                .extension()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_lowercase())
        });

        let (bit_rate, sample_rate, codec) = if let Some(streams) = item["MediaStreams"].as_array()
        {
            if let Some(stream) = streams.iter().find(|s| s["Type"] == "Audio") {
                let bit_rate = stream["BitRate"].as_i64().and_then(|n| n.try_into().ok());
                let sample_rate = stream["SampleRate"]
                    .as_i64()
                    .and_then(|n| n.try_into().ok());
                let codec = stream["Codec"].as_str().map(|s| s.to_string());
                (bit_rate, sample_rate, codec)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        Ok(MusicItem {
            id: item_id,
            name: item["Name"].as_str().unwrap_or("").to_string(),
            item_type: item["Type"].as_str().unwrap_or("").to_string(),
            album: item["Album"].as_str().map(|s| s.to_string()),
            artists: artists_vec,
            artist_ids: artist_ids_vec,
            path: item["Path"].as_str().map(|s| s.to_string()),
            duration: duration_seconds,
            album_art_url,
            year: item["ProductionYear"]
                .as_i64()
                .and_then(|n| n.try_into().ok()),
            play_count: item["UserData"]["PlayCount"]
                .as_i64()
                .and_then(|n| n.try_into().ok()),
            is_favorite: item["UserData"]["IsFavorite"].as_bool(),
            track_number: item["IndexNumber"].as_i64().and_then(|n| n.try_into().ok()),
            container,
            bit_rate,
            sample_rate,
            codec,
            genres: genres_vec,
            premiere_date: item["PremiereDate"].as_str().map(|s| s.to_string()),
            date_played: item["UserData"]["LastPlayedDate"]
                .as_str()
                .map(|s| s.to_string()),
            date_created: item["DateCreated"].as_str().map(|s| s.to_string()),
            album_artists,
            lyrics: None,
        })
    }

    /// Extract album artwork URL from item
    fn extract_album_art_url(&self, item: &serde_json::Value, item_id: &str) -> Option<String> {
        let image_id = item["AlbumId"].as_str().unwrap_or(item_id);

        if let Some(tags) = item["ImageTags"].as_object() {
            if tags.contains_key("Primary") {
                return Some(format!(
                    "{}/Items/{}/Images/Primary",
                    self.server_url.trim_end_matches('/'),
                    image_id
                ));
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
    pub async fn get_artist_details(
        &self,
        user_id: &str,
        artist_id: &str,
    ) -> AppResult<ArtistInfo> {
        let artist_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Users/{}/Items/{}?Fields=Overview,ProviderIds,CommunityRating",
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

        let mut artist_info: ArtistInfo = response.json().await?;

        // Add image URL if available
        if let Some(tags) = &artist_info.image_tags {
            if tags.as_object().is_some_and(|t| t.contains_key("Primary")) {
                artist_info.image_url = Some(format!(
                    "{}/Items/{}/Images/Primary",
                    self.server_url.trim_end_matches('/'),
                    artist_info.id
                ));
            }
        }

        Ok(artist_info)
    }

    /// Get all artists
    pub async fn get_all_artists(&self) -> AppResult<Vec<ArtistInfo>> {
        let artists_url = utils::build_jellyfin_url(&self.server_url, "/Artists?Recursive=true");

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

        let response_json: ArtistItem = response.json().await?;

        let artists_with_urls = response_json
            .items
            .into_iter()
            .map(|mut artist| {
                if let Some(tags) = &artist.image_tags {
                    if tags.as_object().is_some_and(|t| t.contains_key("Primary")) {
                        artist.image_url = Some(format!(
                            "{}/Items/{}/Images/Primary",
                            self.server_url.trim_end_matches('/'),
                            artist.id
                        ));
                    }
                }
                artist
            })
            .collect();

        Ok(artists_with_urls)
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
