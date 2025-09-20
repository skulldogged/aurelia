//! Jellyfin API service client

use crate::error::{AppError, AppResult};
use crate::models::{
    Artist, NameIdPair, Song,
    auth::{JellyfinAuthResponse, LoginResponse},
    jellyfin::{ClientCapabilities, JellyfinLyrics},
};
use crate::utils;
use reqwest::Client;
use serde_json;
use std::collections::HashMap;
use tracing::debug;

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

    /// Get album artists only
    pub async fn get_album_artists(&self) -> AppResult<Vec<Artist>> {
        let artists_url = utils::build_jellyfin_url(
            &self.server_url,
            "/Artists/AlbumArtists?Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,SongCount",
        );

        let response = self
            .client
            .get(&artists_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch album artists: HTTP {}",
                response.status()
            )));
        }

        let response_text = response.text().await?;
        let response_json: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                AppError::ApiParse(format!("Failed to parse album artists JSON: {}", e))
            })?;

        let items = response_json["Items"].as_array().ok_or_else(|| {
            AppError::ApiParse("Invalid album artists response format".to_string())
        })?;

        let mut artists = Vec::new();

        for item in items {
            let artist = self.parse_single_artist(item)?;
            artists.push(artist);
        }

        debug!("Fetched {} album artists from Jellyfin API", artists.len());
        Ok(artists)
    }

    /// Get albums directly from Jellyfin
    pub async fn get_albums(&self, user_id: &str) -> AppResult<Vec<crate::models::Album>> {
        let albums_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Users/{}/Items?IncludeItemTypes=MusicAlbum&Recursive=true&Fields=ImageTags,Overview,ProductionYear,CommunityRating,SongCount,Artists,ArtistItems",
                user_id
            ),
        );

        let response = self
            .client
            .get(&albums_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch albums: HTTP {}",
                response.status()
            )));
        }

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AppError::ApiParse(format!("Failed to parse albums JSON: {}", e)))?;

        let items = response_json["Items"]
            .as_array()
            .ok_or_else(|| AppError::ApiParse("Invalid albums response format".to_string()))?;

        let mut albums = Vec::new();

        for item in items {
            let album = self.parse_single_album(item)?;
            albums.push(album);
        }

        debug!("Fetched {} albums directly from Jellyfin API", albums.len());
        Ok(albums)
    }

    /// Parse a single album from JSON
    fn parse_single_album(&self, item: &serde_json::Value) -> AppResult<crate::models::Album> {
        let id = item["Id"].as_str().map(|s| s.to_string());
        let name = item["Name"].as_str().unwrap_or("Unknown Album").to_string();

        let artist = if let Some(artists) = item["Artists"].as_array() {
            artists
                .first()
                .and_then(|a| a.as_str())
                .unwrap_or("Unknown Artist")
        } else {
            "Unknown Artist"
        };

        let artist_id = item["AlbumArtists"]
            .as_array()
            .and_then(|artists| artists.first())
            .and_then(|artist| artist["Id"].as_str())
            .map(|s| s.to_string());

        let album_art_url = if let Some(tags) = item["ImageTags"].as_object() {
            if tags.contains_key("Primary") {
                let base_url = format!(
                    "{}/Items/{}/Images/Primary",
                    self.server_url.trim_end_matches('/'),
                    item["Id"].as_str().unwrap_or("")
                );

                if let Some(token) = &self.token {
                    Some(format!("{}?api_key={}", base_url, token))
                } else {
                    Some(base_url)
                }
            } else {
                None
            }
        } else {
            None
        };

        let song_count = item["SongCount"].as_i64().unwrap_or(0) as i32;

        Ok(crate::models::Album {
            id,
            name,
            artist: artist.to_string(),
            artist_id,
            album_art_url,
            song_count,
            songs: None,
        })
    }

    /// Get music library items
    pub async fn get_music_library(&self, user_id: &str) -> AppResult<Vec<Song>> {
        let library_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Users/{}/Items?IncludeItemTypes=Audio&Recursive=true&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate,AlbumArtists,MediaStreams,DateCreated,DateLastSaved,DateLastMediaAdded,ArtistItems,MediaSources,Width,Height,Container",
                user_id
            ),
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
            &format!(
                "/Users/{}/Items?IncludeItemTypes=Audio&Recursive=true&Filters=IsPlayed&SortBy=DatePlayed&SortOrder=Descending&Limit=20&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate,AlbumArtists,MediaStreams,DateCreated,DateLastSaved,DateLastMediaAdded",
                user_id
            ),
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

        let duration = item["RunTimeTicks"]
            .as_i64()
            .map(|ticks| ticks as f64 / 10_000_000.0);

        let user_data = &item["UserData"];
        let play_count = user_data["PlayCount"]
            .as_i64()
            .and_then(|n| n.try_into().ok());
        let is_favorite = user_data["IsFavorite"].as_bool();
        let date_played = user_data["LastPlayedDate"].as_str().map(String::from);

        let year = item["ProductionYear"]
            .as_i64()
            .and_then(|n| n.try_into().ok());
        let track_number = item["IndexNumber"].as_i64().and_then(|n| n.try_into().ok());
        let premiere_date = item["PremiereDate"].as_str().map(|s| s.to_string());
        let date_created = item["DateCreated"].as_str().map(|s| s.to_string());

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

        let album_art_url = self.extract_album_art_url(item, &id);
        let artists = self.extract_artists(item);
        let artist_ids = self.extract_artist_ids(item);
        let genres = self.extract_genres(item);
        let album_artists = self.extract_album_artists(item);

        let container = item["Container"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| {
                item["Path"].as_str().and_then(|p| {
                    std::path::Path::new(p)
                        .extension()
                        .and_then(|os_str| os_str.to_str())
                        .map(|s| s.to_lowercase())
                })
            });

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

        if let Some(tags) = item["ImageTags"].as_object()
            && tags.contains_key("Primary")
        {
            let base_url = format!(
                "{}/Items/{}/Images/Primary",
                self.server_url.trim_end_matches('/'),
                image_id
            );

            if let Some(token) = &self.token {
                return Some(format!("{}?api_key={}", base_url, token));
            } else {
                return Some(base_url);
            }
        }
        None
    }

    /// Extract artists from item
    fn extract_artists(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        let artists_value = &item["Artists"];

        if let Some(arr) = artists_value.as_array() {
            let mut artists = Vec::new();
            for artist_value in arr.iter() {
                if let Some(artist_str) = artist_value.as_str() {
                    if artist_str.contains('\x1F') {
                        for split_artist in artist_str.split('\x1F').filter(|s| !s.is_empty()) {
                            artists.push(split_artist.to_string());
                        }
                    } else {
                        artists.push(artist_str.to_string());
                    }
                }
            }
            if !artists.is_empty() {
                return Some(artists);
            }
        }

        if let Some(artists_str) = artists_value.as_str() {
            let artists = artists_str
                .split('\x1F')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            if !artists.is_empty() {
                return Some(artists);
            }
        }
        None
    }

    /// Extract artist IDs from item
    fn extract_artist_ids(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        let artist_items = &item["ArtistItems"];

        if let Some(arr) = artist_items.as_array() {
            let mut artist_ids = Vec::new();
            for item_value in arr.iter() {
                if let Some(id_str) = item_value["Id"].as_str() {
                    if id_str.contains('\x1F') {
                        for split_id in id_str.split('\x1F').filter(|s| !s.is_empty()) {
                            artist_ids.push(split_id.to_string());
                        }
                    } else {
                        artist_ids.push(id_str.to_string());
                    }
                }
            }
            if !artist_ids.is_empty() {
                return Some(artist_ids);
            }
        }

        if let Some(arr) = artist_items.as_array() {
            let artist_ids = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>();
            if !artist_ids.is_empty() {
                let mut final_ids = Vec::new();
                for id_str in artist_ids {
                    if id_str.contains('\x1F') {
                        for split_id in id_str.split('\x1F').filter(|s| !s.is_empty()) {
                            final_ids.push(split_id.to_string());
                        }
                    } else {
                        final_ids.push(id_str);
                    }
                }
                return Some(final_ids);
            }
        }
        None
    }

    /// Extract genres from item
    fn extract_genres(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        let genres_value = &item["Genres"];

        if let Some(arr) = genres_value.as_array() {
            let mut genres = Vec::new();
            for genre_value in arr {
                if let Some(genre_str) = genre_value.as_str() {
                    if genre_str.contains('\x1F') {
                        for split_genre in genre_str.split('\x1F').filter(|s| !s.is_empty()) {
                            genres.push(split_genre.to_string());
                        }
                    } else {
                        genres.push(genre_str.to_string());
                    }
                }
            }
            if !genres.is_empty() {
                return Some(genres);
            }
        }

        if let Some(genres_str) = genres_value.as_str() {
            let genres = genres_str
                .split('\x1F')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            if !genres.is_empty() {
                return Some(genres);
            }
        }

        None
    }

    /// Extract album artists from item
    fn extract_album_artists(&self, item: &serde_json::Value) -> Option<Vec<NameIdPair>> {
        let album_artists_value = &item["AlbumArtists"];

        if let Some(arr) = album_artists_value.as_array() {
            let mut album_artists = Vec::new();
            for artist_value in arr.iter() {
                if let (Some(id), Some(name)) =
                    (artist_value["Id"].as_str(), artist_value["Name"].as_str())
                {
                    album_artists.push(NameIdPair {
                        id: id.to_string(),
                        name: name.to_string(),
                    });
                }
            }
            if !album_artists.is_empty() {
                return Some(album_artists);
            }
        }

        if let Some(_album_artists_str) = album_artists_value.as_str() {
            // This is more complex since we need to parse name-id pairs from a string
            // For now, return None and let it fall back to track artists
        }

        None
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

        let response_text = response.text().await?;

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

    /// Get all artists for a user via Items endpoint (more complete Overview support)
    pub async fn get_all_artists_for_user(&self, user_id: &str) -> AppResult<Vec<Artist>> {
        let mut all_artists = Vec::new();
        let mut start_index = 0;
        let limit = 100; // Jellyfin's default page size

        loop {
            let url = utils::build_jellyfin_url(
                &self.server_url,
                &format!(
                    "/Users/{}/Items?IncludeItemTypes=MusicArtist&Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,SongCount&StartIndex={}&Limit={}",
                    user_id, start_index, limit
                ),
            );

            let response = self
                .client
                .get(&url)
                .header("Authorization", self.get_auth_header())
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(AppError::Network(format!(
                    "Failed to fetch artists (Items): HTTP {}",
                    response.status()
                )));
            }

            let response_json: serde_json::Value = response.json().await?;

            let items = response_json["Items"]
                .as_array()
                .ok_or_else(|| AppError::ApiParse("Invalid artists response format".to_string()))?;

            if items.is_empty() {
                // No more artists to fetch
                break;
            }

            for item in items {
                let artist = self.parse_single_artist(item)?;
                all_artists.push(artist);
            }

            start_index += items.len();
        }

        Ok(all_artists)
    }

    /// Parse a single artist from JSON with manual field mapping
    fn parse_single_artist(&self, item: &serde_json::Value) -> AppResult<Artist> {
        let id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing artist ID".to_string()))?
            .to_string();

        let name = item["Name"].as_str().unwrap_or("").to_string();

        let image_tags = item["ImageTags"].clone();

        let image_url = if let Some(tags) = item["ImageTags"].as_object() {
            if tags.contains_key("Primary") {
                let base_url = format!(
                    "{}/Items/{}/Images/Primary",
                    self.server_url.trim_end_matches('/'),
                    id
                );

                if let Some(token) = &self.token {
                    Some(format!("{}?api_key={}", base_url, token))
                } else {
                    Some(base_url)
                }
            } else {
                None
            }
        } else {
            None
        };

        let overview = item["Overview"].as_str().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

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

        debug!(
            "Audio stream - Item ID: {}, Container: {:?}, Supports seeking: {}",
            item_id, container, supports_seeking
        );

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

    /// Register client capabilities with the Jellyfin server
    pub async fn register_capabilities(&self, capabilities: &ClientCapabilities) -> AppResult<()> {
        let capabilities_url =
            utils::build_jellyfin_url(&self.server_url, "/Sessions/Capabilities");

        let response = self
            .client
            .post(&capabilities_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(capabilities)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to register capabilities: HTTP {}",
                response.status()
            )));
        }

        debug!("Successfully registered client capabilities with Jellyfin server");
        Ok(())
    }

    /// Report playback start to the Jellyfin server
    pub async fn report_playback_start(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
    ) -> AppResult<()> {
        let playing_url = utils::build_jellyfin_url(&self.server_url, "/Sessions/Playing");

        let mut request_body = serde_json::json!({
            "ItemId": item_id
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        let response = self
            .client
            .post(&playing_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "No response body".to_string());
            return Err(AppError::Network(format!(
                "Failed to report playback start: HTTP {} - {}",
                status, body
            )));
        }

        debug!("Successfully reported playback start for item: {}", item_id);
        Ok(())
    }

    /// Report playback progress to the Jellyfin server
    pub async fn report_playback_progress(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
        event_name: Option<&str>,
        is_paused: Option<bool>,
    ) -> AppResult<()> {
        let progress_url =
            utils::build_jellyfin_url(&self.server_url, "/Sessions/Playing/Progress");

        let mut request_body = serde_json::json!({
            "ItemId": item_id
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        if let Some(event) = event_name {
            request_body["EventName"] = serde_json::json!(event);
        }

        if let Some(paused) = is_paused {
            request_body["IsPaused"] = serde_json::json!(paused);
        }

        let response = self
            .client
            .post(&progress_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to report playback progress: HTTP {}",
                response.status()
            )));
        }

        debug!(
            "Successfully reported playback progress for item: {}",
            item_id
        );
        Ok(())
    }

    /// Report playback stop to the Jellyfin server
    pub async fn report_playback_stop(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
    ) -> AppResult<()> {
        let stopped_url = utils::build_jellyfin_url(&self.server_url, "/Sessions/Playing/Stopped");

        let mut request_body = serde_json::json!({
            "ItemId": item_id,
            "IsPaused": true
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        let response = self
            .client
            .post(&stopped_url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to report playback stop: HTTP {}",
                response.status()
            )));
        }

        debug!("Successfully reported playback stop for item: {}", item_id);
        Ok(())
    }

    /// Mark item as played (update play count and last played date)
    pub async fn mark_item_played(&self, user_id: &str, item_id: &str) -> AppResult<()> {
        let played_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("/Users/{}/PlayedItems/{}", user_id, item_id),
        );

        let response = self
            .client
            .post(&played_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to mark item as played: HTTP {}",
                response.status()
            )));
        }

        debug!(
            "Successfully marked item {} as played for user {}",
            item_id, user_id
        );
        Ok(())
    }
}
