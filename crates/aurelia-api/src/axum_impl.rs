//! Axum API implementation
//!
//! This module provides the Axum-specific implementation of the Api trait
//! for the web backend.

use std::path::PathBuf;
use std::sync::Arc;
use crate::{Api, ApiResult, AppError, Credentials, Song, Album, Artist, Playlist, LibraryData, HomeViewData, PlaylistCreateData, PlaylistUpdateData};
use aurelia_core::listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen};
use std::collections::HashMap;

/// Application state for Axum
#[derive(Clone)]
pub struct AppState {
    pub app_data_dir: PathBuf,
}

/// Helper to get cached credentials
fn get_credentials(state: &AppState) -> ApiResult<Option<Credentials>> {
    aurelia_core::load_credentials(state.app_data_dir.to_string_lossy().to_string())
}

/// Axum API implementation
pub struct AxumApiImpl {
    state: Arc<AppState>,
}

impl AxumApiImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl Api for AxumApiImpl {
    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>> {
        get_credentials(&self.state)
    }

    async fn authenticate(&self, server_url: String, username: String, password: String) -> ApiResult<Credentials> {
        let login_resp = aurelia_core::authenticate(server_url.clone(), username.clone(), password).await?;
        let creds = Credentials {
            server_url: server_url.clone(),
            username: username.clone(),
            token: login_resp.token.clone(),
            user_id: login_resp.user_id.clone(),
        };
        
        // Save to disk
        let _ = aurelia_core::save_credentials(
            self.state.app_data_dir.to_string_lossy().to_string(),
            creds.clone(),
        );
        
        Ok(creds)
    }

    async fn logout(&self) -> ApiResult<()> {
        aurelia_core::clear_credentials(self.state.app_data_dir.to_string_lossy().to_string())
    }

    async fn get_library(&self) -> ApiResult<LibraryData> {
        let songs = aurelia_core::load_cached_songs(self.state.app_data_dir.to_string_lossy().to_string())?;
        
        // Derive albums and artists from songs
        let mut album_map: HashMap<String, Vec<Song>> = HashMap::new();
        let mut artist_map: HashMap<String, Artist> = HashMap::new();
        
        for song in &songs {
            if let Some(album_id) = &song.album_id {
                album_map.entry(album_id.clone()).or_default().push(song.clone());
            }
            
            if let Some(artist_ids) = &song.artist_ids {
                for (i, artist_id) in artist_ids.iter().enumerate() {
                    if !artist_map.contains_key(artist_id) {
                        let name = song.artists.as_ref()
                            .and_then(|a| a.get(i))
                            .cloned()
                            .unwrap_or_else(|| "Unknown Artist".to_string());
                        artist_map.insert(artist_id.clone(), Artist {
                            name,
                            id: artist_id.clone(),
                            image_tags: None,
                            image_url: None,
                            overview: None,
                            provider_ids: None,
                            community_rating: None,
                            song_count: None,
                            date_modified: None,
                            songs: None,
                        });
                    }
                }
            }
        }
        
        let albums: Vec<Album> = album_map.iter()
            .filter_map(|(album_id, album_songs)| {
                let first_song = album_songs.iter().max_by_key(|s| s.date_created.as_deref().unwrap_or(""))?;
                Some(Album {
                    id: Some(album_id.clone()),
                    name: first_song.album.clone().unwrap_or_else(|| "Unknown Album".to_string()),
                    artist: first_song.artists.as_ref()?.first()?.clone(),
                    artist_id: first_song.artist_ids.as_ref()?.first().cloned(),
                    album_art_url: first_song.album_art_url.clone(),
                    song_count: album_songs.len() as i64,
                    songs: None,
                    image_tags: None,
                    provider_ids: None,
                    date_created: first_song.date_created.clone(),
                    date_modified: None,
                })
            })
            .collect();
        
        let artists: Vec<Artist> = artist_map.into_values().collect();
        
        Ok(LibraryData {
            albums,
            artists,
            songs,
        })
    }

    async fn sync_library(&self) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        
        // Fetch songs from Jellyfin (result is saved to disk by fetch_songs)
        let _songs = aurelia_core::fetch_songs(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
            self.state.app_data_dir.to_string_lossy().to_string(),
        ).await?;
        
        Ok(())
    }

    async fn get_sync_state(&self) -> ApiResult<aurelia_core::domain::SyncState> {
        aurelia_core::get_sync_state(self.state.app_data_dir.to_string_lossy().to_string())
    }

    async fn get_song(&self, song_id: String) -> ApiResult<Song> {
        // Try cache first
        if let Ok(Some(song)) = aurelia_core::get_cached_song(self.state.app_data_dir.to_string_lossy().to_string(), song_id.clone()) {
            return Ok(song);
        }
        
        Err(AppError::General("Song not found".to_string()))
    }

    async fn toggle_favorite_status(&self, item_id: String, is_favorite: bool) -> ApiResult<bool> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let new_state = aurelia_core::toggle_favorite(
            creds.server_url,
            creds.token,
            creds.user_id,
            item_id,
            is_favorite,
        ).await?;
        Ok(new_state)
    }

    async fn get_instant_mix(&self, item_id: String, _limit: Option<u32>) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_instant_mix(creds.server_url, creds.token, item_id).await
    }

    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>> {
        // Get song from cache
        let song = aurelia_core::get_cached_song(self.state.app_data_dir.to_string_lossy().to_string(), item_id)?
            .ok_or_else(|| AppError::General("Song not found".to_string()))?;
        
        aurelia_core::get_song_share_urls(song).await
    }

    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist> {
        // Try cache first
        if let Ok(Some(artist)) = aurelia_core::get_cached_artist(self.state.app_data_dir.to_string_lossy().to_string(), artist_id.clone()) {
            return Ok(artist);
        }
        
        // Fetch from server
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_artist(
            creds.server_url,
            creds.token,
            creds.user_id,
            artist_id,
            self.state.app_data_dir.to_string_lossy().to_string(),
        ).await
    }

    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>> {
        aurelia_core::get_related_artists(self.state.app_data_dir.to_string_lossy().to_string(), artist_id).await
    }

    async fn get_album(&self, album_id: String) -> ApiResult<Album> {
        // Try cache first
        if let Ok(Some(album)) = aurelia_core::get_cached_album(self.state.app_data_dir.to_string_lossy().to_string(), album_id.clone()) {
            return Ok(album);
        }
        
        // Fetch from server
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_album(
            creds.server_url,
            creds.token,
            creds.user_id,
            album_id,
            self.state.app_data_dir.to_string_lossy().to_string(),
        ).await
    }

    async fn get_album_share_urls(&self, album_id: String) -> ApiResult<HashMap<String, String>> {
        // Get album from cache
        let album = aurelia_core::get_cached_album(self.state.app_data_dir.to_string_lossy().to_string(), album_id)?
            .ok_or_else(|| AppError::General("Album not found".to_string()))?;
        
        // Use MusicBrainz to get share URLs
        aurelia_core::services::MusicBrainzService::get_album_share_urls(&album)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn get_playlists(&self) -> ApiResult<Vec<Playlist>> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlists(creds.server_url, creds.token, creds.user_id).await
    }

    async fn get_playlist_items(&self, playlist_id: String) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlist_items(creds.server_url, creds.token, playlist_id).await
    }

    async fn create_playlist(&self, data: PlaylistCreateData) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        // Build the proper aurelia_core PlaylistCreateData
        let core_data = aurelia_core::models::PlaylistCreateData {
            name: data.name,
            ids: data.ids,
            user_id: creds.user_id.clone(),
            is_public: data.is_public,
        };
        aurelia_core::create_playlist(
            creds.server_url,
            creds.token,
            core_data,
        ).await
    }

    async fn update_playlist(&self, playlist_id: String, updates: PlaylistUpdateData) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        // Build the proper aurelia_core PlaylistUpdateData
        let core_data = aurelia_core::models::PlaylistUpdateData {
            name: updates.name,
            ids: updates.ids,
            user_id: Some(creds.user_id.clone()),
            is_public: updates.is_public,
            songs: None,
            is_favorite: None,
        };
        aurelia_core::update_playlist(
            creds.server_url,
            creds.token,
            playlist_id,
            core_data,
        ).await
    }

    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::delete_playlist(creds.server_url, creds.token, playlist_id).await
    }

    async fn add_playlist_items(&self, playlist_id: String, song_ids: Vec<String>) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::add_playlist_items(creds.server_url, creds.token, playlist_id, song_ids).await
    }

    async fn remove_playlist_items(&self, playlist_id: String, song_ids: Vec<String>) -> ApiResult<()> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::remove_playlist_items(creds.server_url, creds.token, playlist_id, song_ids).await
    }

    async fn get_home_view_data(&self) -> ApiResult<HomeViewData> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let all_songs = aurelia_core::load_cached_songs(self.state.app_data_dir.to_string_lossy().to_string())?;
        
        // Get recently played from server
        let recently_played = aurelia_core::get_recently_played(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
        ).await.unwrap_or_default();
        
        // Derive recently added (newest songs by date_created)
        let mut recently_added = all_songs.clone();
        recently_added.sort_by(|a, b| b.date_created.cmp(&a.date_created));
        recently_added.truncate(20);
        
        // Get albums for random selection
        let mut albums: Vec<Album> = Vec::new();
        let mut album_ids: HashMap<String, bool> = HashMap::new();
        
        for song in &all_songs {
            if let Some(album_id) = &song.album_id {
                if !album_ids.contains_key(album_id) {
                    album_ids.insert(album_id.clone(), true);
                    albums.push(Album {
                        id: Some(album_id.clone()),
                        name: song.album.clone().unwrap_or_default(),
                        artist: song.artists.as_ref().and_then(|a| a.first()).cloned().unwrap_or_default(),
                        artist_id: song.artist_ids.as_ref().and_then(|a| a.first()).cloned(),
                        album_art_url: song.album_art_url.clone(),
                        song_count: 0,
                        songs: None,
                        image_tags: None,
                        provider_ids: None,
                        date_created: song.date_created.clone(),
                        date_modified: None,
                    });
                }
            }
        }
        
        // Shuffle albums for random selection
        albums.sort_by(|_, _| rand::random::<bool>().cmp(&rand::random::<bool>()));
        let random_albums: Vec<Album> = albums.iter().take(20).cloned().collect();
        
        // Featured albums (same as random for now)
        let featured_albums = random_albums.clone();
        
        Ok(HomeViewData {
            recently_played,
            recently_added,
            random_albums,
            featured_albums,
        })
    }

    async fn get_recently_played(&self, limit: Option<u32>) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.state)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let songs = aurelia_core::get_recently_played(
            creds.server_url,
            creds.token,
            creds.user_id,
        ).await?;
        
        if let Some(lim) = limit {
            Ok(songs.into_iter().take(lim as usize).collect())
        } else {
            Ok(songs)
        }
    }

    async fn get_image(&self, item_id: String, image_type: String, server_url: String, token: String, width: Option<u32>, quality: Option<u32>) -> ApiResult<Option<String>> {
        // Build the Jellyfin image URL with provided credentials
        let mut url = format!("{}/Items/{}/Images/{}", server_url.trim_end_matches('/'), item_id, image_type);
        
        let mut query = Vec::new();
        if let Some(w) = width {
            query.push(format!("width={}", w));
        }
        if let Some(q) = quality {
            query.push(format!("quality={}", q));
        }
        query.push(format!("api_key={}", token));
        
        if !query.is_empty() {
            url.push_str("?");
            url.push_str(&query.join("&"));
        }
        
        Ok(Some(url))
    }

    async fn get_audio_stream_url(&self, item_id: String, server_url: String, token: String, container: Option<String>) -> ApiResult<String> {
        Ok(aurelia_core::build_stream_url(server_url, token, item_id, container))
    }

    async fn register_client_capabilities(&self, server_url: String, token: String, device_id: String) -> ApiResult<()> {
        use reqwest::Client;
        use serde_json::json;
        
        let capabilities_url = format!("{}/Sessions/Capabilities/Full", server_url.trim_end_matches('/'));
        
        // Build capabilities payload
        let request_body = json!({
            "capabilities": {
                "PlayableMediaTypes": ["Audio"],
                "SupportedCommands": ["PlayNow", "PlayNext", "SetVolume", "ToggleMute"],
                "SupportsMediaControl": true,
                "SupportsPersistentIdentifier": true,
                "DeviceProfile": {
                    "Name": "Aurelia Audio Profile",
                    "Id": device_id,
                    "MaxStreamingBitrate": 140000000,
                    "MaxStaticBitrate": 140000000,
                    "MusicStreamingTranscodingBitrate": 384000,
                    "MaxStaticMusicBitrate": 4000000,
                    "DirectPlayProfiles": [
                        {"Container": "mp3", "AudioCodec": "mp3", "Type": "Audio"},
                        {"Container": "flac", "AudioCodec": "flac", "Type": "Audio"},
                        {"Container": "ogg", "AudioCodec": "vorbis", "Type": "Audio"}
                    ],
                    "TranscodingProfiles": [
                        {"Container": "mp3", "AudioCodec": "mp3", "Type": "Audio", "Context": "Streaming"}
                    ],
                    "ContainerProfiles": [],
                    "CodecProfiles": [],
                    "SubtitleProfiles": [{"Format": "srt", "Method": "External"}],
                    "ResponseProfiles": [
                        {"Type": "Audio", "MimeType": "audio/mp3"}
                    ]
                }
            }
        });
        
        let client = Client::new();
        let response = client
            .post(&capabilities_url)
            .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "No response body".to_string());
            return Err(AppError::Network(format!(
                "Failed to register capabilities: HTTP {} - {}",
                status, body
            )));
        }
        
        Ok(())
    }

    async fn report_playback(&self, server_url: String, token: String, item_id: String, position_ticks: Option<i64>, event_name: Option<String>, is_paused: Option<bool>) -> ApiResult<()> {
        use reqwest::Client;
        use serde_json::json;
        
        let client = Client::new();
        let auth_header = format!("MediaBrowser Token=\"{}\"", token);
        
        // Determine endpoint and payload based on event type
        let (endpoint, request_body) = match event_name.as_deref() {
            Some("start") => {
                let url = format!("{}/Sessions/Playing", server_url.trim_end_matches('/'));
                let mut body = json!({
                    "ItemId": item_id,
                    "CanSeek": true,
                    "IsPaused": false,
                    "IsMuted": false
                });
                if let Some(pos) = position_ticks {
                    body["PositionTicks"] = json!(pos);
                }
                (url, body)
            }
            Some("stop") => {
                let url = format!("{}/Sessions/Playing/Stopped", server_url.trim_end_matches('/'));
                let mut body = json!({
                    "ItemId": item_id,
                    "IsPaused": false,
                    "IsMuted": false
                });
                if let Some(pos) = position_ticks {
                    body["PositionTicks"] = json!(pos);
                }
                (url, body)
            }
            _ => {
                // Default to progress report
                let url = format!("{}/Sessions/Playing/Progress", server_url.trim_end_matches('/'));
                let mut body = json!({
                    "ItemId": item_id,
                    "IsMuted": false
                });
                if let Some(pos) = position_ticks {
                    body["PositionTicks"] = json!(pos);
                }
                if let Some(event) = &event_name {
                    body["EventName"] = json!(event);
                }
                body["IsPaused"] = json!(is_paused.unwrap_or(false));
                (url, body)
            }
        };
        
        let response = client
            .post(&endpoint)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to report playback: HTTP {}",
                response.status()
            )));
        }
        
        Ok(())
    }

    async fn clear_image_cache(&self) -> ApiResult<()> {
        // Web doesn't cache images locally
        Ok(())
    }

    async fn get_image_cache_stats(&self) -> ApiResult<String> {
        Ok("{}".to_string())
    }

    async fn get_lyrics(&self, _id: String, artist: String, title: String, _path: Option<String>) -> ApiResult<String> {
        Ok(aurelia_core::get_lyrics(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            artist,
            title,
        ).await)
    }

    async fn clear_cache(&self) -> ApiResult<()> {
        aurelia_core::clear_cache(self.state.app_data_dir.to_string_lossy().to_string())
    }

    async fn listenbrainz_set_credentials(&self, _credentials: ListenBrainzCredentials) -> ApiResult<()> {
        // Stub - ListenBrainz credential storage not implemented in aurelia_core yet
        Ok(())
    }

    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()> {
        // Stub - ListenBrainz credential storage not implemented in aurelia_core yet
        Ok(())
    }

    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool> {
        // Stub - ListenBrainz credential storage not implemented in aurelia_core yet
        Ok(false)
    }

    async fn listenbrainz_validate_token(&self, _user_token: String) -> ApiResult<ListenBrainzCredentials> {
        // Stub - ListenBrainz validation not implemented in aurelia_core yet
        Err(AppError::General("Not implemented".to_string()))
    }

    async fn listenbrainz_submit_listen(&self, _listen: ListenBrainzListen, _timestamp: i64) -> ApiResult<()> {
        // Stub - ListenBrainz submission not implemented in aurelia_core yet
        Ok(())
    }

    async fn listenbrainz_playing_now(&self, _artist: String, _track: String, _album: Option<String>) -> ApiResult<()> {
        // Stub - ListenBrainz submission not implemented in aurelia_core yet
        Ok(())
    }

    // Desktop-only methods - these return errors on web
    async fn audio_play(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_pause(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_stop(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_get_volume(&self) -> ApiResult<f64> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn audio_set_volume(&self, _volume: f64) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_start(&self, _app_id: String) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }

    async fn discord_rpc_stop(&self) -> ApiResult<()> {
        Err(AppError::General("Desktop-only feature".to_string()))
    }
}
