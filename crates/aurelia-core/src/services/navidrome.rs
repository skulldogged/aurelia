use crate::error::{AppError, AppResult};
use crate::models::{
    Album, Artist, LoginResponse, NameIdPair, Playlist, PlaylistCreateData, PlaylistUpdateData,
    Song,
};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

const SUBSONIC_API_VERSION: &str = "1.16.1";
const SUBSONIC_CLIENT_NAME: &str = "aurelia";

#[derive(Clone)]
pub struct NavidromeClient {
    client: Client,
    server_url: String,
    username: String,
    password: String,
}

impl NavidromeClient {
    #[must_use]
    pub fn new(server_url: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            username: String::new(),
            password: String::new(),
        }
    }

    #[must_use]
    pub fn with_auth(server_url: String, username: String, password: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            username,
            password,
        }
    }

    pub async fn detect(server_url: &str) -> AppResult<bool> {
        let client = Client::new();
        let url = format!(
            "{}/rest/ping.view",
            server_url.trim_end_matches('/')
        );

        let response = client
            .get(url)
            .query(&[
                ("f", "json"),
                ("v", SUBSONIC_API_VERSION),
                ("c", SUBSONIC_CLIENT_NAME),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let json: Value = response.json().await?;
        Ok(json.get("subsonic-response").is_some())
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        _device_id: &str,
    ) -> AppResult<LoginResponse> {
        let temp = Self::with_auth(
            self.server_url.clone(),
            username.to_string(),
            password.to_string(),
        );

        let response = temp.request("ping", vec![]).await?;
        let _ = subsonic_root(&response)?;

        Ok(LoginResponse {
            token: password.to_string(),
            user_id: username.to_string(),
        })
    }

    pub fn get_audio_stream_url(&self, item_id: &str, _container: Option<&str>) -> String {
        let mut query = self.auth_query();
        query.push(("id".to_string(), item_id.to_string()));
        build_navidrome_url(&self.server_url, "stream", &query)
    }

    pub fn get_mobile_audio_stream_url(&self, item_id: &str, container: Option<&str>) -> String {
        self.get_audio_stream_url(item_id, container)
    }

    pub fn get_cover_art_url(&self, cover_art_id: &str) -> String {
        let mut query = self.auth_query();
        query.push(("id".to_string(), cover_art_id.to_string()));
        build_navidrome_url(&self.server_url, "getCoverArt", &query)
    }

    pub async fn get_music_library(&self, _user_id: &str) -> AppResult<Vec<Song>> {
        let albums = self.get_all_albums().await?;
        let mut songs = Vec::new();

        for album in albums {
            if let Some(album_id) = album.id {
                let album_songs = self.get_album_songs(&album_id).await?;
                songs.extend(album_songs);
            }
        }

        Ok(songs)
    }

    pub async fn get_recently_played(&self, _user_id: &str) -> AppResult<Vec<Song>> {
        let response = self
            .request(
                "getAlbumList2",
                vec![
                    ("type".to_string(), "recent".to_string()),
                    ("size".to_string(), "20".to_string()),
                    ("offset".to_string(), "0".to_string()),
                ],
            )
            .await?;
        let root = subsonic_root(&response)?;
        let albums = value_array(root.pointer("/albumList2/album"));

        let mut songs = Vec::new();
        for album in albums {
            if let Some(album_id) = album.get("id").and_then(Value::as_str) {
                songs.extend(self.get_album_songs(album_id).await?);
            }
        }

        Ok(songs)
    }

    pub async fn get_instant_mix(&self, _item_id: &str) -> AppResult<Vec<Song>> {
        let response = self
            .request(
                "getRandomSongs",
                vec![("size".to_string(), "50".to_string())],
            )
            .await?;
        let root = subsonic_root(&response)?;
        let songs = value_array(root.pointer("/randomSongs/song"))
            .into_iter()
            .map(|song| parse_song(self, song))
            .collect::<AppResult<Vec<_>>>()?;
        Ok(songs)
    }

    pub async fn get_all_artists_for_user(&self, _user_id: &str) -> AppResult<Vec<Artist>> {
        self.get_all_artists().await
    }

    pub async fn get_all_artists(&self) -> AppResult<Vec<Artist>> {
        let response = self.request("getArtists", vec![]).await?;
        let root = subsonic_root(&response)?;
        let indexes = value_array(root.pointer("/artists/index"));
        let mut artists = Vec::new();

        for index in indexes {
            for artist in value_array(index.get("artist")) {
                artists.push(parse_artist(self, artist));
            }
        }

        Ok(artists)
    }

    pub async fn get_albums(&self, _user_id: &str) -> AppResult<Vec<Album>> {
        self.get_all_albums().await
    }

    pub async fn get_artist_details(&self, _user_id: &str, artist_id: &str) -> AppResult<Artist> {
        let response = self
            .request(
                "getArtist",
                vec![("id".to_string(), artist_id.to_string())],
            )
            .await?;
        let root = subsonic_root(&response)?;
        let artist = root
            .get("artist")
            .ok_or_else(|| AppError::ApiParse("Missing artist in getArtist response".to_string()))?;

        let mut parsed = parse_artist(self, artist);
        let mut songs = Vec::new();
        for album in value_array(artist.get("album")) {
            if let Some(album_id) = album.get("id").and_then(Value::as_str) {
                songs.extend(self.get_album_songs(album_id).await?);
            }
        }
        parsed.song_count = Some(songs.len() as i64);
        parsed.songs = Some(songs);

        Ok(parsed)
    }

    pub async fn get_album_details(&self, _user_id: &str, album_id: &str) -> AppResult<Album> {
        let response = self
            .request(
                "getAlbum",
                vec![("id".to_string(), album_id.to_string())],
            )
            .await?;
        let root = subsonic_root(&response)?;
        let album = root
            .get("album")
            .ok_or_else(|| AppError::ApiParse("Missing album in getAlbum response".to_string()))?;

        let mut parsed = parse_album(self, album);
        let songs = value_array(album.get("song"))
            .into_iter()
            .map(|song| parse_song(self, song))
            .collect::<AppResult<Vec<_>>>()?;

        parsed.song_count = songs.len() as i64;
        parsed.songs = Some(songs);
        Ok(parsed)
    }

    pub async fn get_playlist_items(&self, playlist_id: &str) -> AppResult<Vec<Song>> {
        let response = self
            .request(
                "getPlaylist",
                vec![("id".to_string(), playlist_id.to_string())],
            )
            .await?;
        let root = subsonic_root(&response)?;

        let entries = value_array(root.pointer("/playlist/entry"));
        entries
            .into_iter()
            .map(|song| parse_song(self, song))
            .collect::<AppResult<Vec<_>>>()
    }

    pub async fn get_playlists(&self, _user_id: &str) -> AppResult<Vec<Playlist>> {
        let response = self.request("getPlaylists", vec![]).await?;
        let root = subsonic_root(&response)?;
        let playlists = value_array(root.pointer("/playlists/playlist"));

        let mut parsed = Vec::new();
        for playlist in playlists {
            parsed.push(parse_playlist(self, playlist));
        }

        Ok(parsed)
    }

    pub async fn create_playlist(&self, data: &PlaylistCreateData) -> AppResult<Playlist> {
        let mut params = vec![("name".to_string(), data.name.clone())];
        for song_id in data.ids.clone().unwrap_or_default() {
            params.push(("songId".to_string(), song_id));
        }

        let response = self.request("createPlaylist", params).await?;
        let root = subsonic_root(&response)?;
        let playlist = root
            .get("playlist")
            .ok_or_else(|| AppError::ApiParse("Missing playlist in createPlaylist response".to_string()))?;

        Ok(parse_playlist(self, playlist))
    }

    pub async fn update_playlist(
        &self,
        playlist_id: &str,
        updates: &PlaylistUpdateData,
    ) -> AppResult<Playlist> {
        let mut params = vec![("playlistId".to_string(), playlist_id.to_string())];
        if let Some(name) = &updates.name {
            params.push(("name".to_string(), name.clone()));
        }

        if let Some(ids) = &updates.ids {
            let existing = self.get_playlist_items(playlist_id).await?;

            for index in (0..existing.len()).rev() {
                params.push(("songIndexToRemove".to_string(), index.to_string()));
            }

            for id in ids {
                params.push(("songIdToAdd".to_string(), id.clone()));
            }
        }

        let _ = self.request("updatePlaylist", params).await?;

        let mut playlist = self
            .get_playlists("")
            .await?
            .into_iter()
            .find(|p| p.id == playlist_id)
            .unwrap_or_else(|| Playlist {
                name: updates
                    .name
                    .clone()
                    .unwrap_or_else(|| "Playlist".to_string()),
                server_id: self.server_url.clone(),
                id: playlist_id.to_string(),
                can_delete: Some(true),
                sort_name: None,
                is_folder: true,
                item_type: "Playlist".to_string(),
                user_data: None,
                run_time_ticks: None,
                child_count: updates.ids.as_ref().map(|ids| ids.len() as i32),
                image_tags: None,
                backdrop_image_tags: None,
                image_blur_hashes: None,
                location_type: "Virtual".to_string(),
                media_type: Some("Audio".to_string()),
                date_created: None,
                date_last_saved: None,
                is_favorite: None,
                description: None,
                songs: None,
            });

        if let Some(ids) = &updates.ids {
            playlist.songs = Some(
                ids.iter()
                    .map(|id| Song {
                        id: id.clone(),
                        name: "Unknown".to_string(),
                        item_type: "Audio".to_string(),
                        album: None,
                        album_id: None,
                        artists: None,
                        artist_ids: None,
                        path: None,
                        duration: None,
                        album_art_url: None,
                        year: None,
                        play_count: None,
                        is_favorite: None,
                        disc_number: None,
                        track_number: None,
                        container: None,
                        bit_rate: None,
                        sample_rate: None,
                        codec: None,
                        genres: None,
                        premiere_date: None,
                        date_played: None,
                        date_created: None,
                        date_modified: None,
                        album_artists: None,
                        lyrics: None,
                        image_tags: None,
                    })
                    .collect(),
            );
        }

        Ok(playlist)
    }

    pub async fn delete_playlist(&self, playlist_id: &str) -> AppResult<()> {
        let _ = self
            .request(
                "deletePlaylist",
                vec![("id".to_string(), playlist_id.to_string())],
            )
            .await?;
        Ok(())
    }

    pub async fn add_playlist_items(&self, playlist_id: &str, item_ids: &[String]) -> AppResult<()> {
        let mut params = vec![("playlistId".to_string(), playlist_id.to_string())];
        for item in item_ids {
            params.push(("songIdToAdd".to_string(), item.clone()));
        }
        let _ = self.request("updatePlaylist", params).await?;
        Ok(())
    }

    pub async fn remove_playlist_items(
        &self,
        playlist_id: &str,
        item_ids: &[String],
    ) -> AppResult<()> {
        let existing = self.get_playlist_items(playlist_id).await?;
        let mut params = vec![("playlistId".to_string(), playlist_id.to_string())];

        for (index, song) in existing.iter().enumerate().rev() {
            if item_ids.iter().any(|item_id| item_id == &song.id) {
                params.push(("songIndexToRemove".to_string(), index.to_string()));
            }
        }

        let _ = self.request("updatePlaylist", params).await?;
        Ok(())
    }

    pub async fn toggle_favorite(
        &self,
        _user_id: &str,
        item_id: &str,
        is_favorite: bool,
    ) -> AppResult<()> {
        let endpoint = if is_favorite { "star" } else { "unstar" };
        let _ = self
            .request(endpoint, vec![("id".to_string(), item_id.to_string())])
            .await?;
        Ok(())
    }

    pub async fn get_favorite_ids(&self, _user_id: &str) -> AppResult<Vec<String>> {
        let response = self.request("getStarred2", vec![]).await?;
        let root = subsonic_root(&response)?;
        let songs = value_array(root.pointer("/starred2/song"));

        Ok(songs
            .into_iter()
            .filter_map(|song| song.get("id").and_then(Value::as_str).map(ToString::to_string))
            .collect())
    }

    pub async fn mark_item_played(&self, _user_id: &str, item_id: &str) -> AppResult<()> {
        self.report_playback_stop(item_id, None).await
    }

    pub async fn report_playback_start(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
    ) -> AppResult<()> {
        let mut params = vec![
            ("id".to_string(), item_id.to_string()),
            ("submission".to_string(), "false".to_string()),
        ];

        if let Some(ticks) = position_ticks {
            params.push(("time".to_string(), (ticks / 10_000_000).to_string()));
        }

        let _ = self.request("scrobble", params).await?;
        Ok(())
    }

    pub async fn report_playback_progress(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
        _duration_ticks: Option<i64>,
        _is_paused: Option<bool>,
    ) -> AppResult<()> {
        self.report_playback_start(item_id, position_ticks).await
    }

    pub async fn report_playback_stop(
        &self,
        item_id: &str,
        position_ticks: Option<i64>,
    ) -> AppResult<()> {
        let mut params = vec![
            ("id".to_string(), item_id.to_string()),
            ("submission".to_string(), "true".to_string()),
        ];

        if let Some(ticks) = position_ticks {
            params.push(("time".to_string(), (ticks / 10_000_000).to_string()));
        }

        let _ = self.request("scrobble", params).await?;
        Ok(())
    }

    pub async fn request(&self, endpoint: &str, params: Vec<(String, String)>) -> AppResult<Value> {
        let mut query = self.auth_query();
        query.extend(params);

        let url = format!(
            "{}/rest/{}.view",
            self.server_url.trim_end_matches('/'),
            endpoint
        );

        let response = self.client.get(url).query(&query).send().await?;

        if !response.status().is_success() {
            return Err(AppError::Http {
                status: response.status().as_u16(),
                detail: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "request failed".to_string()),
            });
        }

        let json: Value = response.json().await?;
        let _ = subsonic_root(&json)?;
        Ok(json)
    }

    fn auth_query(&self) -> Vec<(String, String)> {
        let salt = uuid::Uuid::new_v4().to_string().replace('-', "");
        let token = format!("{:x}", md5::compute(format!("{}{}", self.password, salt)));

        vec![
            ("u".to_string(), self.username.clone()),
            ("t".to_string(), token),
            ("s".to_string(), salt),
            ("v".to_string(), SUBSONIC_API_VERSION.to_string()),
            ("c".to_string(), SUBSONIC_CLIENT_NAME.to_string()),
            ("f".to_string(), "json".to_string()),
        ]
    }

    async fn get_all_albums(&self) -> AppResult<Vec<Album>> {
        let mut offset = 0;
        let mut albums = Vec::new();

        loop {
            let response = self
                .request(
                    "getAlbumList2",
                    vec![
                        ("type".to_string(), "alphabeticalByName".to_string()),
                        ("size".to_string(), "500".to_string()),
                        ("offset".to_string(), offset.to_string()),
                    ],
                )
                .await?;

            let root = subsonic_root(&response)?;
            let page = value_array(root.pointer("/albumList2/album"));
            if page.is_empty() {
                break;
            }

            let count = page.len();
            for album in page {
                albums.push(parse_album(self, album));
            }

            offset += count;
            if count < 500 {
                break;
            }
        }

        Ok(albums)
    }

    async fn get_album_songs(&self, album_id: &str) -> AppResult<Vec<Song>> {
        let response = self
            .request(
                "getAlbum",
                vec![("id".to_string(), album_id.to_string())],
            )
            .await?;
        let root = subsonic_root(&response)?;
        let songs = value_array(root.pointer("/album/song"))
            .into_iter()
            .map(|song| parse_song(self, song))
            .collect::<AppResult<Vec<_>>>()?;
        Ok(songs)
    }
}

fn build_navidrome_url(server_url: &str, endpoint: &str, query: &[(String, String)]) -> String {
    let encoded_query = query
        .iter()
        .map(|(key, value)| {
            format!("{}={}", urlencoding::encode(key), urlencoding::encode(value))
        })
        .collect::<Vec<_>>()
        .join("&");

    format!(
        "{}/rest/{}.view?{}",
        server_url.trim_end_matches('/'),
        endpoint,
        encoded_query
    )
}

fn subsonic_root(json: &Value) -> AppResult<&Value> {
    let root = json
        .get("subsonic-response")
        .ok_or_else(|| AppError::ApiParse("Missing subsonic-response wrapper".to_string()))?;

    if root.get("status").and_then(Value::as_str) == Some("failed") {
        let detail = root
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("Unknown Subsonic API error")
            .to_string();
        return Err(AppError::Auth(detail));
    }

    Ok(root)
}

fn value_array(value: Option<&Value>) -> Vec<&Value> {
    let Some(value) = value else {
        return Vec::new();
    };

    match value {
        Value::Array(values) => values.iter().collect(),
        Value::Null => Vec::new(),
        other => vec![other],
    }
}

fn parse_song(client: &NavidromeClient, song: &Value) -> AppResult<Song> {
    let id = song
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::ApiParse("Song missing id".to_string()))?
        .to_string();

    let album_art_id = song
        .get("coverArt")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let album_art_lookup_id = album_art_id.clone().or_else(|| {
        song.get("albumId")
            .and_then(Value::as_str)
            .map(ToString::to_string)
    });
    let album_art_url = album_art_lookup_id
        .as_ref()
        .map(|cover_art_id| client.get_cover_art_url(cover_art_id));

    let mut image_tags = None;
    if let Some(cover_art_id) = &album_art_id {
        let mut map = HashMap::new();
        map.insert("Primary".to_string(), cover_art_id.clone());
        image_tags = Some(map);
    }

    let artist_name = song
        .get("artist")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let artist_id = song
        .get("artistId")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let album_artist = match (artist_name.clone(), artist_id.clone()) {
        (Some(name), Some(id)) => Some(vec![NameIdPair { name, id }]),
        _ => None,
    };

    let container = song
        .get("suffix")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            song.get("contentType")
                .and_then(Value::as_str)
                .and_then(|content_type| content_type.split('/').nth(1))
                .map(ToString::to_string)
        });
    let codec = container.clone().or_else(|| {
        song.get("contentType")
            .and_then(Value::as_str)
            .and_then(|content_type| content_type.split('/').nth(1))
            .map(ToString::to_string)
    });

    Ok(Song {
        id,
        name: song
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
            .to_string(),
        item_type: "Audio".to_string(),
        album: song.get("album").and_then(Value::as_str).map(ToString::to_string),
        album_id: song
            .get("albumId")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        artists: artist_name.clone().map(|artist| vec![artist]),
        artist_ids: artist_id.clone().map(|id| vec![id]),
        path: song.get("path").and_then(Value::as_str).map(ToString::to_string),
        duration: song.get("duration").and_then(Value::as_f64).or_else(|| {
            song.get("duration")
                .and_then(Value::as_i64)
                .map(|duration| duration as f64)
        }),
        album_art_url,
        year: song
            .get("year")
            .and_then(Value::as_i64)
            .map(|year| year as i32),
        play_count: song
            .get("playCount")
            .and_then(Value::as_i64)
            .map(|play_count| play_count as i32),
        is_favorite: Some(song.get("starred").is_some()),
        disc_number: song
            .get("discNumber")
            .and_then(Value::as_i64)
            .map(|disc| disc as i32),
        track_number: song
            .get("track")
            .and_then(Value::as_i64)
            .map(|track| track as i32),
        container,
        bit_rate: song
            .get("bitRate")
            .and_then(Value::as_i64)
            .map(normalize_bitrate_to_bps),
        sample_rate: song
            .get("samplingRate")
            .and_then(Value::as_i64)
            .map(|sample_rate| sample_rate as i32),
        codec,
        genres: song
            .get("genre")
            .and_then(Value::as_str)
            .map(|genre| vec![genre.to_string()]),
        premiere_date: song
            .get("created")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        date_played: song
            .get("played")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        date_created: song
            .get("created")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        date_modified: song
            .get("changed")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        album_artists: album_artist,
        lyrics: None,
        image_tags,
    })
}

fn parse_artist(client: &NavidromeClient, artist: &Value) -> Artist {
    let cover_art_id = artist
        .get("coverArt")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let mut image_tags = None;
    if let Some(cover_art_id) = cover_art_id.as_ref() {
        let mut map = HashMap::new();
        map.insert("Primary".to_string(), cover_art_id.clone());
        image_tags = Some(map);
    }

    Artist {
        name: artist
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Unknown Artist")
            .to_string(),
        id: artist
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        image_tags,
        image_url: cover_art_id
            .as_ref()
            .map(|cover_art_id| client.get_cover_art_url(cover_art_id)),
        overview: None,
        provider_ids: None,
        community_rating: None,
        song_count: artist.get("albumCount").and_then(Value::as_i64),
        date_modified: None,
        songs: None,
    }
}

fn parse_album(client: &NavidromeClient, album: &Value) -> Album {
    let cover_art_id = album
        .get("coverArt")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let mut image_tags = None;
    if let Some(cover_art_id) = cover_art_id.as_ref() {
        let mut map = HashMap::new();
        map.insert("Primary".to_string(), cover_art_id.clone());
        image_tags = Some(map);
    }

    Album {
        id: album.get("id").and_then(Value::as_str).map(ToString::to_string),
        name: album
            .get("name")
            .or_else(|| album.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("Unknown Album")
            .to_string(),
        artist: album
            .get("artist")
            .and_then(Value::as_str)
            .unwrap_or("Unknown Artist")
            .to_string(),
        artist_id: album
            .get("artistId")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        album_art_url: cover_art_id
            .as_ref()
            .map(|cover_art_id| client.get_cover_art_url(cover_art_id)),
        song_count: album
            .get("songCount")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        songs: None,
        image_tags,
        provider_ids: None,
        date_created: album
            .get("created")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        date_modified: album
            .get("changed")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

fn parse_playlist(client: &NavidromeClient, playlist: &Value) -> Playlist {
    let mut image_tags = None;
    if let Some(cover_art_id) = playlist.get("coverArt").and_then(Value::as_str) {
        let mut map = HashMap::new();
        map.insert("Primary".to_string(), cover_art_id.to_string());
        image_tags = Some(map);
    }

    Playlist {
        name: playlist
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Playlist")
            .to_string(),
        server_id: client.server_url.to_string(),
        id: playlist
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        can_delete: Some(true),
        sort_name: None,
        is_folder: true,
        item_type: "Playlist".to_string(),
        user_data: None,
        run_time_ticks: playlist
            .get("duration")
            .and_then(Value::as_i64)
            .map(|duration| duration * 10_000_000),
        child_count: playlist
            .get("songCount")
            .and_then(Value::as_i64)
            .map(|count| count as i32),
        image_tags,
        backdrop_image_tags: None,
        image_blur_hashes: None,
        location_type: "Virtual".to_string(),
        media_type: Some("Audio".to_string()),
        date_created: playlist
            .get("created")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        date_last_saved: playlist
            .get("changed")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        is_favorite: None,
        description: None,
        songs: None,
    }
}

fn normalize_bitrate_to_bps(bit_rate: i64) -> i32 {
    // OpenSubsonic commonly reports kbps. Convert to bps so UI formatters can consistently divide by 1000.
    let normalized = if (1..10_000).contains(&bit_rate) {
        bit_rate * 1_000
    } else {
        bit_rate
    };
    normalized as i32
}
