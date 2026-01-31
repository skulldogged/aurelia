//! Tauri API implementation
//!
//! This module provides the Tauri-specific implementation of the Api trait.

use tauri::{AppHandle, Manager};
use crate::{Api, ApiResult, AppError, Credentials, Song, Album, Artist, Playlist, LibraryData, HomeViewData, PlaylistCreateData, PlaylistUpdateData};
use aurelia_core::listenbrainz_core::{ListenBrainzCredentials, ListenBrainzListen};
use std::collections::HashMap;

/// Helper to get cached credentials from Tauri state
fn get_credentials(app: &AppHandle) -> ApiResult<Option<Credentials>> {
    let app_state: tauri::State<'_, aurelia_core::state::AppState> = app.state();
    
    // Check memory cache first
    if let Some(creds) = app_state.get_credentials() {
        return Ok(Some(creds));
    }
    
    // Load from disk
    let app_dir = app.path().app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))?;
    
    match aurelia_core::load_credentials(app_dir.to_string_lossy().to_string())? {
        Some(creds) => {
            // Cache for future
            app_state.set_credentials(Some(creds.clone()));
            Ok(Some(creds))
        }
        None => Ok(None),
    }
}

/// Tauri API implementation
pub struct TauriApiImpl {
    app: AppHandle,
}

impl TauriApiImpl {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Api for TauriApiImpl {
    async fn get_saved_credentials(&self) -> ApiResult<Option<Credentials>> {
        get_credentials(&self.app)
    }

    async fn authenticate(&self, server_url: String, username: String, password: String) -> ApiResult<Credentials> {
        let login_resp = aurelia_core::authenticate(server_url.clone(), username.clone(), password).await?;
        let creds = Credentials {
            server_url: server_url.clone(),
            username: username.clone(),
            token: login_resp.token.clone(),
            user_id: login_resp.user_id.clone(),
        };
        
        // Save to disk and cache
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e: tauri::Error| AppError::FileSystem(e.to_string()))?;
        let _ = aurelia_core::save_credentials(app_dir.to_string_lossy().to_string(), creds.clone());
        
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        app_state.set_credentials(Some(creds.clone()));
        
        Ok(creds)
    }

    async fn logout(&self) -> ApiResult<()> {
        // Clear memory cache
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        app_state.set_credentials(None);
        
        // Clear from disk
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::clear_credentials(app_dir.to_string_lossy().to_string())
    }

    async fn get_library(&self) -> ApiResult<LibraryData> {
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = app_state.songs.lock().unwrap().clone();
        
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
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        // Fetch songs from Jellyfin
        let songs = aurelia_core::fetch_songs(
            creds.server_url.clone(),
            creds.token.clone(),
            creds.user_id.clone(),
            app_dir.to_string_lossy().to_string(),
        ).await?;
        
        // Update in-memory state
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        *app_state.songs.lock().unwrap() = songs;
        
        Ok(())
    }

    async fn get_sync_state(&self) -> ApiResult<aurelia_core::domain::SyncState> {
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::get_sync_state(app_dir.to_string_lossy().to_string())
    }

    async fn get_song(&self, song_id: String) -> ApiResult<Song> {
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = app_state.songs.lock().unwrap().clone();
        
        songs.into_iter()
            .find(|s| s.id == song_id)
            .ok_or_else(|| AppError::General("Song not found".to_string()))
    }

    async fn toggle_favorite_status(&self, item_id: String, is_favorite: bool) -> ApiResult<bool> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
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
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_instant_mix(creds.server_url, creds.token, item_id).await
    }

    async fn get_song_share_urls(&self, item_id: String) -> ApiResult<HashMap<String, String>> {
        // First get the song from cache
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let songs = app_state.songs.lock().unwrap().clone();
        
        let song = songs.into_iter()
            .find(|s| s.id == item_id)
            .ok_or_else(|| AppError::General("Song not found".to_string()))?;
        
        aurelia_core::get_song_share_urls(song).await
    }

    async fn get_artist(&self, artist_id: String) -> ApiResult<Artist> {
        // Try cache first
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        if let Ok(Some(artist)) = aurelia_core::get_cached_artist(app_dir.to_string_lossy().to_string(), artist_id.clone()) {
            return Ok(artist);
        }
        
        // Fetch from server
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_artist(
            creds.server_url,
            creds.token,
            creds.user_id,
            artist_id,
            app_dir.to_string_lossy().to_string(),
        ).await
    }

    async fn get_related_artists(&self, artist_id: String) -> ApiResult<Vec<Artist>> {
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::get_related_artists(app_dir.to_string_lossy().to_string(), artist_id).await
    }

    async fn get_album(&self, album_id: String) -> ApiResult<Album> {
        // Try cache first
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        if let Ok(Some(album)) = aurelia_core::get_cached_album(app_dir.to_string_lossy().to_string(), album_id.clone()) {
            return Ok(album);
        }
        
        // Fetch from server
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::fetch_album(
            creds.server_url,
            creds.token,
            creds.user_id,
            album_id,
            app_dir.to_string_lossy().to_string(),
        ).await
    }

    async fn get_album_share_urls(&self, album_id: String) -> ApiResult<HashMap<String, String>> {
        // Get album from cache
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        let album = aurelia_core::get_cached_album(app_dir.to_string_lossy().to_string(), album_id)?
            .ok_or_else(|| AppError::General("Album not found".to_string()))?;
        
        // Use MusicBrainz to get share URLs
        aurelia_core::services::MusicBrainzService::get_album_share_urls(&album)
            .await
            .map_err(|e| AppError::General(e.to_string()))
    }

    async fn get_playlists(&self) -> ApiResult<Vec<Playlist>> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlists(creds.server_url, creds.token, creds.user_id).await
    }

    async fn get_playlist_items(&self, playlist_id: String) -> ApiResult<Vec<Song>> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::get_playlist_items(creds.server_url, creds.token, playlist_id).await
    }

    async fn create_playlist(&self, data: PlaylistCreateData) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let core_data = aurelia_core::models::PlaylistCreateData {
            name: data.name,
            ids: data.ids,
            user_id: creds.user_id,
            is_public: None,
        };
        aurelia_core::create_playlist(
            creds.server_url,
            creds.token,
            core_data,
        ).await
    }

    async fn update_playlist(&self, playlist_id: String, updates: PlaylistUpdateData) -> ApiResult<Playlist> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let core_updates = aurelia_core::models::PlaylistUpdateData {
            name: updates.name,
            ids: updates.ids,
            user_id: None,
            is_public: None,
            songs: None,
            is_favorite: None,
        };
        aurelia_core::update_playlist(
            creds.server_url,
            creds.token,
            playlist_id,
            core_updates,
        ).await
    }

    async fn delete_playlist(&self, playlist_id: String) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::delete_playlist(creds.server_url, creds.token, playlist_id).await
    }

    async fn add_playlist_items(&self, playlist_id: String, song_ids: Vec<String>) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::add_playlist_items(creds.server_url, creds.token, playlist_id, song_ids).await
    }

    async fn remove_playlist_items(&self, playlist_id: String, song_ids: Vec<String>) -> ApiResult<()> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        aurelia_core::remove_playlist_items(creds.server_url, creds.token, playlist_id, song_ids).await
    }

    async fn get_home_view_data(&self) -> ApiResult<HomeViewData> {
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
        let app_state: tauri::State<'_, aurelia_core::state::AppState> = self.app.state();
        let all_songs = app_state.songs.lock().unwrap().clone();
        
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
        
        // Get random albums
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        let mut albums: Vec<Album> = Vec::new();
        let mut album_ids: HashMap<String, ()> = HashMap::new();
        
        for song in &all_songs {
            if let Some(album_id) = &song.album_id {
                if !album_ids.contains_key(album_id) {
                    album_ids.insert(album_id.clone(), ());
                    if let Some(album) = all_songs.iter().find(|s| s.album_id == Some(album_id.clone())) {
                        albums.push(Album {
                            id: Some(album_id.clone()),
                            name: album.album.clone().unwrap_or_default(),
                            artist: album.artists.as_ref().and_then(|a| a.first()).cloned().unwrap_or_default(),
                            artist_id: album.artist_ids.as_ref().and_then(|a| a.first()).cloned(),
                            album_art_url: album.album_art_url.clone(),
                            song_count: 0,
                            songs: None,
                            image_tags: None,
                            provider_ids: None,
                            date_created: album.date_created.clone(),
                            date_modified: None,
                        });
                    }
                }
            }
        }
        
        albums.shuffle(&mut rng);
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
        let creds = get_credentials(&self.app)?.ok_or_else(|| AppError::Auth("Not authenticated".to_string()))?;
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
        // Build the Jellyfin image URL with authentication
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
        use aurelia_core::models::jellyfin::{ClientCapabilities, DeviceProfile, DirectPlayProfile, TranscodingProfile, ContainerProfile, CodecProfile, SubtitleProfile, ResponseProfile};
        
        // Create default device profile for audio
        let device_profile = DeviceProfile {
            name: Some("Aurelia Audio Profile".to_string()),
            id: Some(device_id.clone()),
            max_streaming_bitrate: Some(140000000),
            max_static_bitrate: Some(140000000),
            music_streaming_transcoding_bitrate: Some(384000),
            max_static_music_bitrate: Some(4000000),
            direct_play_profiles: vec![
                DirectPlayProfile {
                    container: "mp3".to_string(),
                    audio_codec: Some("mp3".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
                DirectPlayProfile {
                    container: "flac".to_string(),
                    audio_codec: Some("flac".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
                DirectPlayProfile {
                    container: "ogg".to_string(),
                    audio_codec: Some("vorbis".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                },
            ],
            transcoding_profiles: vec![
                TranscodingProfile {
                    container: "mp3".to_string(),
                    audio_codec: Some("mp3".to_string()),
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                    transcode_seek_info: None,
                    copy_timestamps: None,
                    context: Some("Streaming".to_string()),
                    enable_subtitles_in_manifest: None,
                },
            ],
            container_profiles: vec![],
            codec_profiles: vec![],
            subtitle_profiles: vec![
                SubtitleProfile {
                    format: "srt".to_string(),
                    method: "External".to_string(),
                    didl_mode: None,
                },
            ],
            response_profiles: vec![
                ResponseProfile {
                    container: None,
                    audio_codec: None,
                    video_codec: None,
                    profile_type: "Audio".to_string(),
                    mime_type: Some("audio/mp3".to_string()),
                },
            ],
        };
        
        let capabilities = ClientCapabilities {
            playable_media_types: vec!["Audio".to_string()],
            supported_commands: vec![
                "PlayNow".to_string(),
                "PlayNext".to_string(),
                "SetVolume".to_string(),
                "ToggleMute".to_string(),
            ],
            supports_media_control: true,
            supports_persistent_identifier: true,
            device_profile,
            app_store_url: None,
            icon_url: None,
        };
        
        let client = aurelia_core::services::JellyfinClient::with_auth(server_url, token);
        client.register_capabilities(&capabilities).await.map_err(|e| AppError::General(e.to_string()))
    }

    async fn report_playback(&self, server_url: String, token: String, item_id: String, position_ticks: Option<i64>, event_name: Option<String>, is_paused: Option<bool>) -> ApiResult<()> {
        let client = aurelia_core::services::JellyfinClient::with_auth(server_url, token);
        
        // Map event name to appropriate playback report
        match event_name.as_deref() {
            Some("start") => {
                client.report_playback_start(&item_id, position_ticks).await.map_err(|e| AppError::General(e.to_string()))
            }
            Some("stop") => {
                client.report_playback_stop(&item_id, position_ticks).await.map_err(|e| AppError::General(e.to_string()))
            }
            _ => {
                // Default to progress report
                client.report_playback_progress(&item_id, position_ticks, event_name.as_deref(), is_paused).await.map_err(|e| AppError::General(e.to_string()))
            }
        }
    }

    async fn clear_image_cache(&self) -> ApiResult<()> {
        // This would call the existing image cache handler
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
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        aurelia_core::clear_cache(app_dir.to_string_lossy().to_string())
    }

    async fn listenbrainz_set_credentials(&self, _credentials: ListenBrainzCredentials) -> ApiResult<()> {
        // TODO: Implement ListenBrainz credentials storage
        Err(AppError::General("Not implemented".to_string()))
    }

    async fn listenbrainz_clear_credentials(&self) -> ApiResult<()> {
        // TODO: Implement ListenBrainz credentials clearing
        Err(AppError::General("Not implemented".to_string()))
    }

    async fn listenbrainz_is_authenticated(&self) -> ApiResult<bool> {
        // TODO: Implement ListenBrainz authentication check
        Ok(false)
    }

    async fn listenbrainz_validate_token(&self, _user_token: String) -> ApiResult<ListenBrainzCredentials> {
        // TODO: Implement ListenBrainz token validation
        Err(AppError::General("Not implemented".to_string()))
    }

    async fn listenbrainz_submit_listen(&self, _listen: ListenBrainzListen, _timestamp: i64) -> ApiResult<()> {
        // TODO: Implement ListenBrainz listen submission
        Err(AppError::General("Not implemented".to_string()))
    }

    async fn listenbrainz_playing_now(&self, _artist: String, _track: String, _album: Option<String>) -> ApiResult<()> {
        // TODO: Implement ListenBrainz playing now
        Err(AppError::General("Not implemented".to_string()))
    }

    // Desktop-only methods - these delegate to the audio system
    async fn audio_play(&self) -> ApiResult<()> {
        // This would integrate with the audio player system
        // For now, stub - the real implementation is in the existing Tauri audio handlers
        Ok(())
    }

    async fn audio_pause(&self) -> ApiResult<()> {
        Ok(())
    }

    async fn audio_stop(&self) -> ApiResult<()> {
        Ok(())
    }

    async fn audio_get_volume(&self) -> ApiResult<f64> {
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let volume_path = app_dir.join("volume.json");
        
        if !volume_path.exists() {
            return Ok(0.5);
        }
        
        let json = std::fs::read_to_string(&volume_path)
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        serde_json::from_str(&json)
            .map_err(|e| AppError::Serialization(e.to_string()))
    }

    async fn audio_set_volume(&self, volume: f64) -> ApiResult<()> {
        let app_dir = self.app.path().app_data_dir()
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        let volume_path = app_dir.join("volume.json");
        
        let json = serde_json::to_string(&volume)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        
        std::fs::write(&volume_path, json)
            .map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        Ok(())
    }

    async fn discord_rpc_start(&self, _app_id: String) -> ApiResult<()> {
        // TODO: Implement Discord RPC start
        Err(AppError::General("Not implemented".to_string()))
    }

    async fn discord_rpc_stop(&self) -> ApiResult<()> {
        // TODO: Implement Discord RPC stop
        Err(AppError::General("Not implemented".to_string()))
    }
}
