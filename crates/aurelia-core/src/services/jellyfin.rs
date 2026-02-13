//! Jellyfin API service client

use crate::error::{AppError, AppResult};
use crate::models::{
    Artist, NameIdPair, Song,
    auth::{JellyfinAuthResponse, LoginResponse},
    jellyfin::{ClientCapabilities, JellyfinLyrics},
};
use crate::utils;
use crate::utils::error_handling;
use reqwest::Client;
use serde_json;
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Paginated response from Jellyfin Items endpoint
pub struct PaginatedResponse {
    pub items: Vec<serde_json::Value>,
    pub total_record_count: usize,
    /// The Date header from the server response (for clock-skew-safe sync)
    pub server_date: Option<String>,
}

/// Jellyfin API client
pub struct JellyfinClient {
    client: Client,
    server_url: String,
    token: Option<String>,
}

impl JellyfinClient {
    /// Create a new Jellyfin client
    #[must_use]
    pub fn new(server_url: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            token: None,
        }
    }

    /// Create a new authenticated Jellyfin client
    #[must_use]
    pub fn with_auth(server_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            token: Some(token),
        }
    }

    /// Get the HTTP client (for internal use)
    #[must_use]
    pub const fn get_client(&self) -> &Client {
        &self.client
    }

    /// Get the authorization header value
    #[must_use]
    pub fn get_auth_header(&self) -> String {
        self.token.as_ref().map_or_else(
            || utils::build_jellyfin_auth_header(None),
            |token| format!("MediaBrowser Token=\"{token}\""),
        )
    }

    /// Authenticate user with Jellyfin server
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        device_id: &str,
    ) -> AppResult<LoginResponse> {
        let login_url = utils::build_jellyfin_url(&self.server_url, "/Users/AuthenticateByName");

        let response = self
            .client
            .post(&login_url)
            .header(
                "Authorization",
                utils::build_jellyfin_auth_header(Some(device_id)),
            )
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "Username": username,
                "Pw": password
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::auth_error_with_context(
                format!("HTTP {status}: {message}"),
                &format!("Authentication failed for user '{username}'"),
            ));
        }

        let auth_response: JellyfinAuthResponse = response.json().await.map_err(|e| {
            error_handling::api_parse_error_with_context(
                e,
                "Failed to parse authentication response from server",
            )
        })?;
        Ok(LoginResponse {
            token: auth_response.access_token,
            user_id: auth_response.user.id,
        })
    }

    /// Get album artists only
    pub async fn get_album_artists(&self) -> AppResult<Vec<Artist>> {
        let artists_url = utils::build_jellyfin_url(
            &self.server_url,
            "/Artists/AlbumArtists?Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,DateLastModified",
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
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AppError::ApiParse(format!("Failed to parse album artists JSON: {e}")))?;

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
                "/Items?userId={user_id}&IncludeItemTypes=MusicAlbum&Recursive=true&Fields=ImageTags,Overview,ProductionYear,CommunityRating,Artists,ProviderIds,DateCreated,DateLastModified",
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
            .map_err(|e| AppError::ApiParse(format!("Failed to parse albums JSON: {e}")))?;

        let items = response_json["Items"]
            .as_array()
            .ok_or_else(|| AppError::ApiParse("Invalid albums response format".to_string()))?;

        let mut albums = Vec::new();

        for item in items {
            let album = self.parse_single_album(item);
            albums.push(album);
        }

        debug!("Fetched {} albums directly from Jellyfin API", albums.len());
        Ok(albums)
    }

    /// Parse a single album from JSON
    pub fn parse_single_album(&self, item: &serde_json::Value) -> crate::models::Album {
        let id = item["Id"].as_str().map(std::string::ToString::to_string);
        let name = item["Name"].as_str().unwrap_or("Unknown Album").to_string();

        let artist = item["Artists"]
            .as_array()
            .map_or("Unknown Artist", |artists| {
                artists
                    .first()
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Artist")
            });

        let artist_id = item["AlbumArtists"]
            .as_array()
            .and_then(|artists| artists.first())
            .and_then(|artist| artist["Id"].as_str())
            .map(std::string::ToString::to_string);

        let (album_art_url, image_tags) = self.extract_image_info(item, id.as_deref());

        let song_count = item["SongCount"].as_i64().unwrap_or(0);

        let provider_ids = item["ProviderIds"].as_object().map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<String, String>>()
        });

        let date_created = item["DateCreated"]
            .as_str()
            .map(std::string::ToString::to_string);

        let date_modified = item["DateLastModified"]
            .as_str()
            .map(std::string::ToString::to_string);

        crate::models::Album {
            id,
            name,
            artist: artist.to_string(),
            artist_id,
            album_art_url,
            song_count,
            songs: None,
            image_tags,
            provider_ids,
            date_created,
            date_modified,
        }
    }

    /// Get albums for a specific artist
    /// Handles compilation vs non-compilation albums like Feishin does
    /// - For compilations: uses ContributingArtistIds (artist contributed to the album)
    /// - For non-compilations: uses AlbumArtistIds (artist is the album artist)
    /// - If neither specified: uses ArtistIds (all credits)
    pub async fn get_albums_for_artist(
        &self,
        user_id: &str,
        artist_id: &str,
        compilation: Option<bool>,
    ) -> AppResult<Vec<crate::models::Album>> {
        let artist_filter = match compilation {
            Some(true) => format!("ContributingArtistIds={artist_id}"),
            Some(false) => format!("AlbumArtistIds={artist_id}"),
            None => format!("ArtistIds={artist_id}"),
        };

        let albums_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items?userId={user_id}&IncludeItemTypes=MusicAlbum&Recursive=true&{artist_filter}&Fields=People,Tags,ImageTags,Overview,ProductionYear,CommunityRating,Artists,ProviderIds,DateCreated",
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
                "Failed to fetch albums for artist: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        let items = response_json["Items"]
            .as_array()
            .ok_or_else(|| AppError::ApiParse("Invalid albums response format".to_string()))?;

        let albums = items
            .iter()
            .map(|item| self.parse_single_album(item))
            .collect();
        Ok(albums)
    }

    /// Get music library items
    pub async fn get_music_library(&self, user_id: &str) -> AppResult<Vec<Song>> {
        let library_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&Fields=Genres,DateCreated,DateLastModified,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams"
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
        self.parse_music_items(&response_json)
    }

    /// Get recently played music items
    pub async fn get_recently_played(&self, user_id: &str) -> AppResult<Vec<Song>> {
        let library_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&Filters=IsPlayed&SortBy=DatePlayed&SortOrder=Descending&Limit=20&Fields=Genres,DateCreated,DateLastModified,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams"
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
        self.parse_music_items(&response_json)
    }

    /// Get songs for a specific album artist using server-side filtering
    /// This uses the AlbumArtistIds query parameter to let Jellyfin do the filtering
    pub async fn get_songs_for_album_artist(
        &self,
        user_id: &str,
        artist_id: &str,
    ) -> AppResult<Vec<Song>> {
        let songs_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&AlbumArtistIds={artist_id}&Fields=Genres,DateCreated,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams&SortBy=Album,SortName"
            ),
        );

        let response = self
            .client
            .get(&songs_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch songs for artist: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_music_items(&response_json)
    }

    /// Get songs for a specific album using server-side filtering
    /// Includes workaround for Jellyfin bug where AlbumIds filter also matches album names
    pub async fn get_songs_for_album(&self, user_id: &str, album_id: &str) -> AppResult<Vec<Song>> {
        let songs_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&AlbumIds={album_id}&Fields=Genres,DateCreated,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams&SortBy=ParentIndexNumber,IndexNumber,SortName"
            ),
        );

        let response = self
            .client
            .get(&songs_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch songs for album: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        let mut songs = self.parse_music_items(&response_json)?;

        // Workaround for Jellyfin bug: AlbumIds filter searches for both:
        // 1. Matching album ID
        // 2. An album with the NAME of the album
        // Filter client-side to ensure we only get songs from the requested album
        songs.retain(|song| song.album_id.as_ref() == Some(&album_id.to_string()));

        Ok(songs)
    }

    /// Get instant mix (similar songs) for a given item
    pub async fn get_instant_mix(&self, item_id: &str) -> AppResult<Vec<Song>> {
        let instant_mix_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items/{}/InstantMix?Fields=Genres,DateCreated,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams",
                item_id
            ),
        );

        let response = self
            .client
            .get(&instant_mix_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch instant mix: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_music_items(&response_json)
    }

    /// Parse music items from Jellyfin API response
    fn parse_music_items(&self, response_json: &serde_json::Value) -> AppResult<Vec<Song>> {
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
    #[allow(clippy::too_many_lines)]
    pub fn parse_single_music_item(&self, item: &serde_json::Value) -> AppResult<Song> {
        let id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing item ID".to_string()))?
            .to_string();

        let name = item["Name"].as_str().unwrap_or("").to_string();

        let item_type = item["Type"].as_str().unwrap_or("").to_string();

        let album = item["Album"].as_str().map(std::string::ToString::to_string);
        let album_id = item["AlbumId"]
            .as_str()
            .map(std::string::ToString::to_string)
            .or_else(|| {
                item["ParentId"]
                    .as_str()
                    .map(std::string::ToString::to_string)
            });

        #[allow(clippy::cast_precision_loss)]
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
        let disc_number = item["ParentIndexNumber"]
            .as_i64()
            .and_then(|n| n.try_into().ok());
        let track_number = item["IndexNumber"].as_i64().and_then(|n| n.try_into().ok());
        let premiere_date = item["PremiereDate"]
            .as_str()
            .map(std::string::ToString::to_string);
        let date_created = item["DateCreated"]
            .as_str()
            .map(std::string::ToString::to_string);

        let (bit_rate, sample_rate, codec): (Option<i32>, Option<i32>, Option<String>) =
            item["MediaStreams"]
                .as_array()
                .map_or((None, None, None), |streams| {
                    streams.iter().find(|s| s["Type"] == "Audio").map_or(
                        (None, None, None),
                        |audio_stream| {
                            (
                                audio_stream["BitRate"]
                                    .as_i64()
                                    .and_then(|n| n.try_into().ok()),
                                audio_stream["SampleRate"]
                                    .as_i64()
                                    .and_then(|n| n.try_into().ok()),
                                audio_stream["Codec"]
                                    .as_str()
                                    .map(std::string::ToString::to_string),
                            )
                        },
                    )
                });

        let (album_art_url, image_tags) = self.extract_image_info(item, Some(&id));
        let artists = Self::extract_artists(item);
        let artist_ids = Self::extract_artist_ids(item);
        let album_artists = Self::extract_album_artists(item);
        let genres = Self::extract_genres(item);

        let container = item["Container"]
            .as_str()
            .map(std::string::ToString::to_string)
            .or_else(|| {
                item["Path"].as_str().and_then(|p| {
                    std::path::Path::new(p)
                        .extension()
                        .and_then(|os_str| os_str.to_str())
                        .map(str::to_lowercase)
                })
            });

        let date_modified = item["DateLastModified"]
            .as_str()
            .map(std::string::ToString::to_string);

        let song = Song {
            id,
            name,
            item_type,
            album,
            album_id,
            artists,
            artist_ids,
            path: item["Path"].as_str().map(std::string::ToString::to_string),
            duration,
            album_art_url,
            year,
            play_count,
            is_favorite,
            disc_number,
            track_number,
            container,
            bit_rate,
            sample_rate,
            codec,
            genres,
            premiere_date,
            date_played,
            date_created,
            date_modified,
            album_artists,
            lyrics: None,
            image_tags,
        };

        Ok(song)
    }

    /// Extract album artwork URL from item
    fn extract_image_info(
        &self,
        item: &serde_json::Value,
        item_id: Option<&str>,
    ) -> (Option<String>, Option<HashMap<String, String>>) {
        let image_id = item["AlbumId"]
            .as_str()
            .or(item_id)
            .unwrap_or_else(|| item["Id"].as_str().unwrap_or(""));

        // Build image tags map if available
        let image_tags = item["ImageTags"].as_object().map(|tags| {
            tags.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<String, String>>()
        });

        // For songs, always try to build the album art URL using AlbumId (if present)
        // For albums/artists, only build URL if they have a Primary image tag
        let has_album_id = item["AlbumId"].as_str().is_some();
        let has_primary_tag = image_tags
            .as_ref()
            .is_some_and(|tags| tags.contains_key("Primary"));

        let url = if has_album_id || has_primary_tag {
            let base_url = utils::build_jellyfin_url(
                &self.server_url,
                &format!("/Items/{}/Images/Primary", image_id),
            );
            if let Some(token) = &self.token {
                Some(format!("{base_url}?api_key={token}"))
            } else {
                Some(base_url)
            }
        } else {
            None
        };

        (url, image_tags)
    }

    /// Parse a single playlist item from JSON
    fn parse_single_playlist_item(
        &self,
        item: &serde_json::Value,
    ) -> AppResult<crate::models::music::Playlist> {
        let id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing playlist ID".to_string()))?
            .to_string();

        let name = item["Name"]
            .as_str()
            .unwrap_or("Untitled Playlist")
            .to_string();

        let server_id = item["ServerId"].as_str().unwrap_or("").to_string();

        let can_delete = item["CanDelete"].as_bool();
        let sort_name = item["SortName"].as_str().map(String::from);
        let is_folder = item["IsFolder"].as_bool().unwrap_or(true);
        let item_type = item["Type"].as_str().unwrap_or("Playlist").to_string();

        let user_data_json = item["UserData"].as_object();
        let is_favorite = user_data_json
            .and_then(|ud| ud.get("IsFavorite").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        let user_data = user_data_json.map(|ud| crate::models::music::UserData {
            playback_position_ticks: ud
                .get("PlaybackPositionTicks")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            play_count: ud.get("PlayCount").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            is_favorite,
            played: ud.get("Played").and_then(|v| v.as_bool()).unwrap_or(false),
            last_played_date: ud
                .get("LastPlayedDate")
                .and_then(|v| v.as_str())
                .map(String::from),
        });

        let run_time_ticks = item["RunTimeTicks"].as_i64();
        let child_count = item["ChildCount"].as_i64().map(|n| n as i32);
        let date_created = item["DateCreated"].as_str().map(String::from);
        let date_last_saved = item["DateLastSaved"].as_str().map(String::from);

        // Extract image info
        let (image_tags, backdrop_image_tags, image_blur_hashes) =
            if let Some(image_tags_obj) = item["ImageTags"].as_object() {
                let image_tags = image_tags_obj
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>();

                let backdrop_tags = item["BackdropImageTags"].as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });

                let blur_hashes = item["ImageBlurHashes"].as_object().map(|blur_obj| {
                    blur_obj
                        .iter()
                        .filter_map(|(k, v)| {
                            v.as_object().map(|inner| {
                                (
                                    k.clone(),
                                    inner
                                        .iter()
                                        .filter_map(|(ik, iv)| {
                                            iv.as_str().map(|s| (ik.clone(), s.to_string()))
                                        })
                                        .collect(),
                                )
                            })
                        })
                        .collect()
                });

                (Some(image_tags), backdrop_tags, blur_hashes)
            } else {
                (None, None, None)
            };

        let location_type = item["LocationType"]
            .as_str()
            .unwrap_or("Virtual")
            .to_string();
        let media_type = item["MediaType"].as_str().map(String::from);

        Ok(crate::models::music::Playlist {
            name,
            server_id,
            id,
            can_delete,
            sort_name,
            is_folder,
            item_type,
            user_data,
            run_time_ticks,
            child_count,
            image_tags,
            backdrop_image_tags,
            image_blur_hashes,
            location_type,
            media_type,
            date_created,
            date_last_saved,
            is_favorite: Some(is_favorite),
            description: None, // Playlists don't seem to have descriptions in the basic response
            songs: None,       // Songs would need a separate API call
        })
    }

    /// Extract artists from item
    fn extract_artists(item: &serde_json::Value) -> Option<Vec<String>> {
        // Use ArtistItems (array of objects with Name+Id) to ensure alignment with IDs
        let artist_items = &item["ArtistItems"];

        if let Some(arr) = artist_items.as_array() {
            let mut artists = Vec::new();
            for item_value in arr {
                if let Some(name_str) = item_value["Name"].as_str() {
                    if name_str.contains('\x1F') {
                        for split_name in name_str.split('\x1F').filter(|s| !s.is_empty()) {
                            artists.push(split_name.to_string());
                        }
                    } else {
                        artists.push(name_str.to_string());
                    }
                }
            }
            if !artists.is_empty() {
                return Some(artists);
            }
        }

        // Fallback to Artists array if ArtistItems not available
        let artists_value = &item["Artists"];
        if let Some(arr) = artists_value.as_array() {
            let mut artists = Vec::new();
            for artist_value in arr {
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
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
            if !artists.is_empty() {
                return Some(artists);
            }
        }
        None
    }

    /// Extract artist IDs from item
    fn extract_artist_ids(item: &serde_json::Value) -> Option<Vec<String>> {
        let artist_items = &item["ArtistItems"];

        if let Some(arr) = artist_items.as_array() {
            let mut artist_ids = Vec::new();
            for item_value in arr {
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
                .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
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
    fn extract_genres(item: &serde_json::Value) -> Option<Vec<String>> {
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
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
            if !genres.is_empty() {
                return Some(genres);
            }
        }

        None
    }

    /// Extract album artists from item
    fn extract_album_artists(item: &serde_json::Value) -> Option<Vec<NameIdPair>> {
        let album_artists_value = &item["AlbumArtists"];

        if let Some(arr) = album_artists_value.as_array() {
            let mut album_artists = Vec::new();
            for artist_value in arr {
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

        None
    }

    /// Get artist details
    pub async fn get_artist_details(&self, user_id: &str, artist_id: &str) -> AppResult<Artist> {
        let artist_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items/{artist_id}?userId={user_id}&Fields=ImageTags,Overview,ProviderIds,CommunityRating,DateLastModified"
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

    /// Get album details
    pub async fn get_album_details(
        &self,
        user_id: &str,
        album_id: &str,
    ) -> AppResult<crate::models::Album> {
        let album_url = utils::build_jellyfin_url(
            &self.server_url,
            &format!(
                "/Items/{album_id}?userId={user_id}&Fields=ImageTags,Overview,ProductionYear,CommunityRating,Artists,ProviderIds,DateCreated,DateLastModified"
            ),
        );

        let response = self
            .client
            .get(&album_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch album details: HTTP {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        Ok(self.parse_single_album(&response_json))
    }

    /// Get all artists
    pub async fn get_all_artists(&self) -> AppResult<Vec<Artist>> {
        let artists_url = utils::build_jellyfin_url(
            &self.server_url,
            "/Artists?Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating",
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
            .map_err(|e| AppError::ApiParse(format!("Failed to parse artists JSON: {e}")))?;

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
                    "/Items?userId={user_id}&IncludeItemTypes=MusicArtist&Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating&StartIndex={start_index}&Limit={limit}"
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
    pub fn parse_single_artist(&self, item: &serde_json::Value) -> AppResult<Artist> {
        let id = item["Id"]
            .as_str()
            .ok_or_else(|| AppError::ApiParse("Missing artist ID".to_string()))?
            .to_string();

        let name = item["Name"].as_str().unwrap_or("").to_string();

        let image_tags = item["ImageTags"].as_object().map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<String, String>>()
        });

        let image_url = item["ImageTags"].as_object().and_then(|tags| {
            if tags.contains_key("Primary") {
                let base_url = utils::build_jellyfin_url(
                    &self.server_url,
                    &format!("/Items/{}/Images/Primary", id),
                );

                if let Some(token) = &self.token {
                    Some(format!("{base_url}?api_key={token}"))
                } else {
                    Some(base_url)
                }
            } else {
                None
            }
        });

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

        let community_rating = item["CommunityRating"].as_f64();
        let song_count = item["SongCount"].as_i64();

        let date_modified = item["DateLastModified"]
            .as_str()
            .map(std::string::ToString::to_string);

        let artist = Artist {
            name,
            id,
            image_tags,
            image_url,
            overview,
            provider_ids,
            community_rating,
            song_count,
            date_modified,
            songs: None,
        };

        Ok(artist)
    }

    /// Fetch just the filesystem `Path` of an item from Jellyfin.
    ///
    /// Uses a lightweight `GET /Items/{id}?Fields=Path` request.
    pub async fn get_item_path(&self, item_id: &str) -> AppResult<Option<String>> {
        let url =
            utils::build_jellyfin_url(&self.server_url, &format!("/Items/{item_id}?Fields=Path"));

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["Path"].as_str().map(ToString::to_string))
    }

    /// Get lyrics for a song
    pub async fn get_lyrics(&self, item_id: &str) -> AppResult<Option<JellyfinLyrics>> {
        let lyrics_url =
            utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{item_id}/Lyrics"));

        tracing::info!("[Lyrics] GET {}", lyrics_url);

        let response = self
            .client
            .get(&lyrics_url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        let status = response.status();
        tracing::info!("[Lyrics] Response status: {}", status);

        if !status.is_success() {
            return Ok(None); // No lyrics available
        }

        // Read body as text first so we can log it on parse failure
        let body = response.text().await?;
        tracing::info!(
            "[Lyrics] Response body length: {} bytes, preview: {}",
            body.len(),
            &body[..body.len().min(300)]
        );

        match serde_json::from_str::<JellyfinLyrics>(&body) {
            Ok(lyrics) => Ok(Some(lyrics)),
            Err(e) => {
                tracing::error!("[Lyrics] Failed to parse Jellyfin lyrics JSON: {}", e);
                Ok(None)
            }
        }
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
            &format!("/UserFavoriteItems/{item_id}?userId={user_id}"),
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

    /// Get audio stream URL for desktop (raw HTTP streaming with byte-range or startTimeTicks seeking).
    ///
    /// For seekable containers, returns a direct static stream.
    /// For non-seekable containers (ALAC, etc.), returns a transcoded AAC stream.
    /// The desktop player handles seeking via startTimeTicks on the raw stream.
    pub fn get_audio_stream_url(&self, item_id: &str, container: Option<&str>) -> String {
        let token = self.token.as_deref().unwrap_or("");
        if utils::supports_seeking(container) {
            format!(
                "{}?api_key={}&static=true",
                utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{}/stream", item_id)),
                token
            )
        } else {
            format!(
                "{}?api_key={}",
                utils::build_jellyfin_url(
                    &self.server_url,
                    &format!("/Audio/{}/stream.aac", item_id)
                ),
                token
            )
        }
    }

    /// Get audio stream URL for mobile (ExoPlayer/Media3).
    ///
    /// For seekable containers, returns a direct static stream.
    /// For non-seekable containers (ALAC, etc.), uses the `/universal` endpoint which
    /// transcodes to AAC. Uses `transcodingProtocol=http` so ExoPlayer receives a
    /// progressive stream it can parse directly (not HLS which requires a special MediaSource).
    pub fn get_mobile_audio_stream_url(&self, item_id: &str, container: Option<&str>) -> String {
        let token = self.token.as_deref().unwrap_or("");
        if utils::supports_seeking(container) {
            format!(
                "{}?api_key={}&static=true",
                utils::build_jellyfin_url(&self.server_url, &format!("/Audio/{}/stream", item_id)),
                token
            )
        } else {
            format!(
                "{}?api_key={}\
                 &container=mp3,aac,m4a|aac,flac,ogg\
                 &transcodingContainer=aac\
                 &transcodingProtocol=http\
                 &audioCodec=aac\
                 &maxStreamingBitrate=999999999",
                utils::build_jellyfin_url(
                    &self.server_url,
                    &format!("/Audio/{}/universal", item_id)
                ),
                token
            )
        }
    }

    /// Register client capabilities with the Jellyfin server
    pub async fn register_capabilities(&self, capabilities: &ClientCapabilities) -> AppResult<()> {
        let capabilities_url =
            utils::build_jellyfin_url(&self.server_url, "/Sessions/Capabilities/Full");

        let request_body = serde_json::json!({
            "capabilities": capabilities
        });

        let response = self
            .client
            .post(&capabilities_url)
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
            error!(
                "Failed to register capabilities: HTTP {} - Response body: {}",
                status, body
            );
            return Err(AppError::Network(format!(
                "Failed to register capabilities: HTTP {} - {}",
                status, body
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
            "ItemId": item_id,
            "CanSeek": true,
            "IsPaused": false,
            "IsMuted": false
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
                "Failed to report playback start: HTTP {status} - {body}"
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
            "ItemId": item_id,
            "IsMuted": false
        });

        if let Some(position) = position_ticks {
            request_body["PositionTicks"] = serde_json::json!(position);
        }

        if let Some(event) = event_name {
            request_body["EventName"] = serde_json::json!(event);
        }

        if let Some(paused) = is_paused {
            request_body["IsPaused"] = serde_json::json!(paused);
        } else {
            request_body["IsPaused"] = serde_json::json!(false);
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
            "IsPaused": false,
            "IsMuted": false
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
            &format!("/UserPlayedItems/{item_id}?userId={user_id}"),
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

impl JellyfinClient {
    /// Get all playlists for a user
    pub async fn get_playlists(
        &self,
        user_id: &str,
    ) -> AppResult<Vec<crate::models::music::Playlist>> {
        let url = utils::build_jellyfin_url(&self.server_url, &format!("Users/{}/Items", user_id));

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .query(&[("IncludeItemTypes", "Playlist"), ("Recursive", "true")])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                "Failed to get playlists from server",
            ));
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            error_handling::network_error_with_context(
                e,
                "Failed to parse playlists response from server",
            )
        })?;

        let items = response_json["Items"]
            .as_array()
            .ok_or_else(|| AppError::ApiParse("Invalid response format".to_string()))?;

        let mut playlists = Vec::new();

        for item in items {
            let playlist = self.parse_single_playlist_item(item)?;
            playlists.push(playlist);
        }

        debug!(
            "Retrieved {} playlists for user {}",
            playlists.len(),
            user_id
        );
        for playlist in &playlists {
            debug!(
                "Playlist: {} (ID: {}, Type: {}, ChildCount: {:?})",
                playlist.name, playlist.id, playlist.item_type, playlist.child_count
            );
        }
        Ok(playlists)
    }

    /// Create a new playlist
    pub async fn create_playlist(
        &self,
        data: &crate::models::music::PlaylistCreateData,
    ) -> AppResult<crate::models::music::Playlist> {
        let url = utils::build_jellyfin_url(&self.server_url, "Playlists");

        // Construct the request JSON with PascalCase field names as expected by Jellyfin
        let request_body = serde_json::json!({
            "Name": data.name,
            "Ids": data.ids,
            "UserId": data.user_id,
            "IsPublic": data.is_public.unwrap_or(false)
        });

        debug!("Creating playlist with JSON: {}", request_body);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                "Failed to create playlist on server",
            ));
        }

        // Debug: log the raw response
        let response_text = response.text().await.map_err(|e| {
            error_handling::network_error_with_context(
                e,
                "Failed to read playlist creation response",
            )
        })?;

        debug!("Playlist creation response: {}", response_text);

        // Try to parse as full Playlist first
        let playlist = match serde_json::from_str::<crate::models::music::Playlist>(&response_text)
        {
            Ok(p) => p,
            Err(_) => {
                // If that fails, try to parse as a simple object with just Id and Name
                let simple_response: serde_json::Value = serde_json::from_str(&response_text)
                    .map_err(|e| {
                        error!("Failed to parse any playlist JSON: {}", e);
                        error_handling::api_parse_error_with_context(
                            format!("JSON parse error: {}", e),
                            "Failed to parse created playlist response from server",
                        )
                    })?;

                // Create a minimal playlist from the response
                crate::models::music::Playlist {
                    name: simple_response["Name"]
                        .as_str()
                        .unwrap_or("Untitled Playlist")
                        .to_string(),
                    server_id: self.server_url.clone(),
                    id: simple_response["Id"]
                        .as_str()
                        .ok_or_else(|| {
                            error_handling::api_parse_error_with_context(
                                "Missing playlist ID in response".to_string(),
                                "Failed to parse created playlist response from server",
                            )
                        })?
                        .to_string(),
                    can_delete: Some(true),
                    sort_name: None,
                    is_folder: true,
                    item_type: "Playlist".to_string(),
                    user_data: None,
                    run_time_ticks: None,
                    child_count: Some(data.ids.as_ref().map_or(0, |ids| ids.len() as i32)),
                    image_tags: None,
                    backdrop_image_tags: None,
                    image_blur_hashes: None,
                    location_type: "Virtual".to_string(),
                    media_type: None,
                    date_created: None,
                    date_last_saved: None,
                    is_favorite: None,
                    description: None,
                    songs: None,
                }
            }
        };

        info!("Successfully created playlist: {}", playlist.name);
        Ok(playlist)
    }

    /// Update an existing playlist
    pub async fn update_playlist(
        &self,
        playlist_id: &str,
        updates: &crate::models::music::PlaylistUpdateData,
    ) -> AppResult<crate::models::music::Playlist> {
        let url =
            utils::build_jellyfin_url(&self.server_url, &format!("Playlists/{}", playlist_id));

        // Construct the request JSON with PascalCase field names as expected by Jellyfin
        let request_body = serde_json::json!({
            "Name": updates.name,
            "Ids": updates.ids,
            "UserId": updates.user_id,
            "IsPublic": updates.is_public
        });

        debug!("Updating playlist with JSON: {}", request_body);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                &format!("Failed to update playlist {}", playlist_id),
            ));
        }

        // Debug: log the raw response
        let response_text = response.text().await.map_err(|e| {
            error_handling::network_error_with_context(e, "Failed to read playlist update response")
        })?;

        debug!(
            "Playlist update response: '{}' (length: {})",
            response_text,
            response_text.len()
        );

        // For updates, Jellyfin may return empty response or just success status
        let playlist = if response_text.trim().is_empty() {
            debug!("Empty response from playlist update, constructing playlist from input");
            // If response is empty, construct a playlist from the input data
            crate::models::music::Playlist {
                name: updates
                    .name
                    .clone()
                    .unwrap_or_else(|| "Untitled Playlist".to_string()),
                server_id: self.server_url.clone(),
                id: playlist_id.to_string(),
                can_delete: Some(true),
                sort_name: None,
                is_folder: true,
                item_type: "Playlist".to_string(),
                user_data: None,
                run_time_ticks: None,
                child_count: Some(updates.ids.as_ref().map_or(0, |ids| ids.len() as i32)),
                image_tags: None,
                backdrop_image_tags: None,
                image_blur_hashes: None,
                location_type: "Virtual".to_string(),
                media_type: None,
                date_created: None,
                date_last_saved: None,
                is_favorite: None,
                description: None,
                songs: None,
            }
        } else {
            // Try to parse as full Playlist first
            match serde_json::from_str::<crate::models::music::Playlist>(&response_text) {
                Ok(p) => p,
                Err(_) => {
                    // If that fails, try to parse as a simple object with just Id and Name
                    let simple_response: serde_json::Value = serde_json::from_str(&response_text)
                        .map_err(|e| {
                        error!("Failed to parse any updated playlist JSON: {}", e);
                        error_handling::api_parse_error_with_context(
                            format!("JSON parse error: {}", e),
                            "Failed to parse updated playlist response from server",
                        )
                    })?;

                    // Create a minimal playlist from the response
                    crate::models::music::Playlist {
                        name: simple_response["Name"]
                            .as_str()
                            .unwrap_or("Untitled Playlist")
                            .to_string(),
                        server_id: self.server_url.clone(),
                        id: simple_response["Id"]
                            .as_str()
                            .unwrap_or(playlist_id)
                            .to_string(),
                        can_delete: Some(true),
                        sort_name: None,
                        is_folder: true,
                        item_type: "Playlist".to_string(),
                        user_data: None,
                        run_time_ticks: None,
                        child_count: Some(updates.ids.as_ref().map_or(0, |ids| ids.len() as i32)),
                        image_tags: None,
                        backdrop_image_tags: None,
                        image_blur_hashes: None,
                        location_type: "Virtual".to_string(),
                        media_type: None,
                        date_created: None,
                        date_last_saved: None,
                        is_favorite: None,
                        description: None,
                        songs: None,
                    }
                }
            }
        };

        info!("Successfully updated playlist: {}", playlist.name);
        Ok(playlist)
    }

    /// Delete a playlist
    pub async fn delete_playlist(&self, playlist_id: &str) -> AppResult<()> {
        let url = utils::build_jellyfin_url(&self.server_url, &format!("Items/{}", playlist_id));

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                &format!("Failed to delete playlist {}", playlist_id),
            ));
        }

        info!("Successfully deleted playlist {}", playlist_id);
        Ok(())
    }

    /// Add items to a playlist
    pub async fn add_playlist_items(
        &self,
        playlist_id: &str,
        item_ids: &[String],
    ) -> AppResult<()> {
        let url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("Playlists/{}/Items", playlist_id),
        );

        let ids_param = item_ids.join(",");

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .query(&[("Ids", &ids_param)])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                &format!("Failed to add items to playlist {}", playlist_id),
            ));
        }

        info!(
            "Successfully added {} items to playlist {}",
            item_ids.len(),
            playlist_id
        );
        Ok(())
    }

    /// Remove items from a playlist
    pub async fn remove_playlist_items(
        &self,
        playlist_id: &str,
        item_ids: &[String],
    ) -> AppResult<()> {
        let url = utils::build_jellyfin_url(
            &self.server_url,
            &format!("Playlists/{}/Items", playlist_id),
        );

        let ids_param = item_ids.join(",");

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.get_auth_header())
            .query(&[("EntryIds", &ids_param)])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                &format!("Failed to remove items from playlist {}", playlist_id),
            ));
        }

        info!(
            "Successfully removed {} items from playlist {}",
            item_ids.len(),
            playlist_id
        );
        Ok(())
    }

    /// Get items in a playlist
    pub async fn get_playlist_items(
        &self,
        playlist_id: &str,
    ) -> AppResult<Vec<crate::models::music::Song>> {
        // Use Items endpoint with ParentId to get playlist contents
        let url = utils::build_jellyfin_url(&self.server_url, "Items");

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .query(&[
                ("ParentId", playlist_id),
                ("IncludeItemTypes", "Audio"),
                ("Recursive", "false"),
                ("Fields", "Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,Genres,PremiereDate,AlbumArtists,MediaStreams,DateCreated,DateLastSaved,DateLastMediaAdded,MediaSources,Width,Height,Container"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error");
            return Err(error_handling::network_error_with_context(
                format!("HTTP {}: {}", status, message),
                &format!("Failed to get playlist items for {}", playlist_id),
            ));
        }

        // Debug: log the raw response first
        let response_text = response.text().await.map_err(|e| {
            error_handling::network_error_with_context(e, "Failed to read playlist items response")
        })?;

        debug!("Playlist items raw response: {}", response_text);

        // Parse using the same method as library items
        let json_value: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
            error_handling::network_error_with_context(
                format!("Failed to parse JSON: {}", e),
                "Failed to parse playlist items response from server",
            )
        })?;

        let songs = self.parse_music_items(&json_value)?;

        debug!(
            "Retrieved {} items from playlist {}",
            songs.len(),
            playlist_id
        );
        Ok(songs)
    }

    // =========================================================================
    // Smart Sync API Methods
    // =========================================================================

    /// Fetch a single page of items from the Jellyfin Items endpoint.
    /// Returns raw JSON items, total count, and the server's Date header.
    pub async fn fetch_items_page(
        &self,
        base_query: &str,
        start_index: usize,
        limit: usize,
    ) -> AppResult<PaginatedResponse> {
        let query = format!(
            "{base_query}&StartIndex={start_index}&Limit={limit}&enableTotalRecordCount=true"
        );
        let url = utils::build_jellyfin_url(&self.server_url, &query);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch items page (startIndex={start_index}): HTTP {}",
                response.status()
            )));
        }

        // Capture the server's Date header for clock-skew-safe sync timestamps
        let server_date = response
            .headers()
            .get("date")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let response_json: serde_json::Value = response.json().await?;

        let total_record_count = response_json["TotalRecordCount"]
            .as_u64()
            .unwrap_or(0) as usize;

        let items = response_json["Items"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        Ok(PaginatedResponse {
            items,
            total_record_count,
            server_date,
        })
    }

    /// Fetch songs with pagination and optional incremental date filter.
    /// When `since_date` is Some, only items modified since that date are returned.
    /// Returns (songs, server_date_header).
    pub async fn get_songs_paginated(
        &self,
        user_id: &str,
        since_date: Option<&str>,
        page_size: usize,
    ) -> AppResult<(Vec<Song>, Option<String>)> {
        let mut base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&Fields=Genres,DateCreated,DateLastModified,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams"
        );

        append_incremental_date_filter(&mut base_query, since_date);

        let mut all_songs = Vec::new();
        let mut start_index = 0;
        let mut server_date = None;

        loop {
            let page = self.fetch_items_page(&base_query, start_index, page_size).await?;

            if server_date.is_none() {
                server_date = page.server_date;
            }

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                let song = self.parse_single_music_item(item)?;
                all_songs.push(song);
            }

            start_index += page_count;

            info!(
                "Fetched songs page: {}/{} (page_size={})",
                start_index, page.total_record_count, page_size
            );

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Total songs fetched: {}", all_songs.len());
        Ok((all_songs, server_date))
    }

    /// Fetch albums with pagination and optional incremental date filter.
    /// Returns (albums, server_date_header).
    pub async fn get_albums_paginated(
        &self,
        user_id: &str,
        since_date: Option<&str>,
        page_size: usize,
    ) -> AppResult<(Vec<crate::models::Album>, Option<String>)> {
        let mut base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes=MusicAlbum&Recursive=true&Fields=ImageTags,Overview,ProductionYear,CommunityRating,Artists,ProviderIds,DateCreated,DateLastModified"
        );

        append_incremental_date_filter(&mut base_query, since_date);

        let mut all_albums = Vec::new();
        let mut start_index = 0;
        let mut server_date = None;

        loop {
            let page = self.fetch_items_page(&base_query, start_index, page_size).await?;

            if server_date.is_none() {
                server_date = page.server_date;
            }

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                let album = self.parse_single_album(item);
                all_albums.push(album);
            }

            start_index += page_count;

            info!(
                "Fetched albums page: {}/{} (page_size={})",
                start_index, page.total_record_count, page_size
            );

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Total albums fetched: {}", all_albums.len());
        Ok((all_albums, server_date))
    }

    /// Fetch artists with pagination and optional incremental date filter.
    /// Returns (artists, server_date_header).
    pub async fn get_artists_paginated(
        &self,
        user_id: &str,
        since_date: Option<&str>,
        page_size: usize,
    ) -> AppResult<(Vec<Artist>, Option<String>)> {
        let mut base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes=MusicArtist&Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,DateLastModified"
        );

        append_incremental_date_filter(&mut base_query, since_date);

        let mut all_artists = Vec::new();
        let mut start_index = 0;
        let mut server_date = None;

        loop {
            let page = self.fetch_items_page(&base_query, start_index, page_size).await?;

            if server_date.is_none() {
                server_date = page.server_date;
            }

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                let artist = self.parse_single_artist(item)?;
                all_artists.push(artist);
            }

            start_index += page_count;

            info!(
                "Fetched artists page: {}/{} (page_size={})",
                start_index, page.total_record_count, page_size
            );

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Total artists fetched: {}", all_artists.len());
        Ok((all_artists, server_date))
    }

    /// Fetch only IDs for a given item type (lightweight, for deletion detection).
    /// Uses large pages since each item is just an ID string (~36 bytes).
    pub async fn get_all_item_ids(
        &self,
        user_id: &str,
        item_type: &str,
    ) -> AppResult<Vec<String>> {
        // Request no fields - just IDs (Jellyfin always returns Id)
        let base_query = format!(
            "/Items?userId={user_id}&IncludeItemTypes={item_type}&Recursive=true&Fields="
        );

        let mut all_ids = Vec::new();
        let mut start_index = 0;
        let page_size = 2000; // Large page since payloads are tiny

        loop {
            let page = self.fetch_items_page(&base_query, start_index, page_size).await?;

            if page.items.is_empty() {
                break;
            }

            let page_count = page.items.len();
            for item in &page.items {
                if let Some(id) = item["Id"].as_str() {
                    all_ids.push(id.to_string());
                }
            }

            start_index += page_count;

            if start_index >= page.total_record_count {
                break;
            }
        }

        info!("Fetched {} {} IDs for deletion check", all_ids.len(), item_type);
        Ok(all_ids)
    }

    /// Parse the HTTP Date header into an RFC3339 timestamp.
    /// Falls back to local UTC time if parsing fails.
    pub fn parse_server_date(date_header: Option<&str>) -> String {
        if let Some(header) = date_header {
            // HTTP Date format: "Wed, 09 Jun 2021 10:18:14 GMT"
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc2822(header) {
                return parsed.to_rfc3339();
            }
        }
        // Fallback to local clock
        chrono::Utc::now().to_rfc3339()
    }
}

fn append_incremental_date_filter(base_query: &mut String, since_date: Option<&str>) {
    if let Some(date) = since_date {
        let encoded = urlencoding::encode(date);
        base_query.push_str(&format!(
            "&minDateLastSaved={encoded}&minDateLastSavedForUser={encoded}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::append_incremental_date_filter;

    #[test]
    fn append_incremental_date_filter_encodes_timezone_offset() {
        let mut query = "/Items?userId=test".to_string();
        append_incremental_date_filter(&mut query, Some("2026-02-12T18:10:22+00:00"));

        assert!(query.contains("minDateLastSaved=2026-02-12T18%3A10%3A22%2B00%3A00"));
        assert!(query.contains("minDateLastSavedForUser=2026-02-12T18%3A10%3A22%2B00%3A00"));
    }

    #[test]
    fn append_incremental_date_filter_noop_without_date() {
        let mut query = "/Items?userId=test".to_string();
        append_incremental_date_filter(&mut query, None);
        assert_eq!(query, "/Items?userId=test");
    }
}
