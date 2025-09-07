pub mod db;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    token: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Credentials {
    #[serde(rename = "serverUrl")]
    server_url: String,
    username: String,
    token: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JellyfinUser {
    #[serde(rename = "Id")]
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JellyfinAuthResponse {
    #[serde(rename = "AccessToken")]
    access_token: String,
    #[serde(rename = "User")]
    user: JellyfinUser,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameIdPair {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Id")]
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MusicItem {
    id: String,
    name: String,
    item_type: String,
    album: Option<String>,
    artists: Option<Vec<String>>,
    #[serde(rename = "artistIds")]
    artist_ids: Option<Vec<String>>,
    path: Option<String>,
    duration: Option<f64>,
    #[serde(rename = "albumArtUrl")]
    album_art_url: Option<String>,
    year: Option<i64>,
    #[serde(rename = "playCount")]
    play_count: Option<i64>,
    #[serde(rename = "isFavorite")]
    is_favorite: Option<bool>,
    #[serde(rename = "trackNumber")]
    track_number: Option<i32>,
    container: Option<String>,
    genres: Option<Vec<String>>,
    #[serde(rename = "premiereDate")]
    premiere_date: Option<String>,
    #[serde(rename = "datePlayed")]
    date_played: Option<String>,
    #[serde(rename = "albumArtists")]
    album_artists: Option<Vec<NameIdPair>>,
    lyrics: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArtistInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "ImageTags")]
    image_tags: Option<serde_json::Value>,
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
    #[serde(rename = "Overview")]
    overview: Option<String>,
    #[serde(rename = "ProviderIds")]
    provider_ids: Option<serde_json::Value>,
    #[serde(rename = "CommunityRating")]
    community_rating: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LrcLibTrackResponse {
    id: i64,
    name: String,
    #[serde(rename = "trackName")]
    track_name: String,
    #[serde(rename = "artistName")]
    artist_name: String,
    album_name: Option<String>,
    duration: f64,
    instrumental: bool,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArtistItem {
    #[serde(rename = "Items")]
    items: Vec<ArtistInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AlbumInfo {
    name: String,
    artist: String,
    #[serde(rename = "artistId")]
    artist_id: Option<String>,
    #[serde(rename = "albumArtUrl")]
    album_art_url: Option<String>,
    #[serde(rename = "songCount")]
    song_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct AlbumWithSongs {
    name: String,
    artist: String,
    #[serde(rename = "artistId")]
    artist_id: Option<String>,
    #[serde(rename = "albumArtUrl")]
    album_art_url: Option<String>,
    #[serde(rename = "songCount")]
    song_count: i32,
    songs: Vec<MusicItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArtistWithSongs {
    id: String,
    name: String,
    #[serde(rename = "songCount")]
    song_count: i32,
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
    songs: Vec<MusicItem>,
}

#[tauri::command]
async fn login_to_jellyfin(
    server_url: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let client = reqwest::Client::new();

    // Create authentication header
    let auth_header = "MediaBrowser Client=\"JellyfinMusicPlayer\", Device=\"Desktop\", DeviceId=\"1\", Version=\"1.0.0\", Token=\"\"".to_string();

    let login_url = format!(
        "{}/Users/AuthenticateByName",
        server_url.trim_end_matches('/')
    );

    let response = client
        .post(&login_url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "Username": username,
            "Pw": password
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Login failed: HTTP {}", response.status()));
    }

    let auth_response: JellyfinAuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(LoginResponse {
        token: auth_response.access_token,
        user_id: auth_response.user.id,
    })
}

#[tauri::command]
fn save_credentials(
    server_url: String,
    username: String,
    token: String,
    user_id: String,
) -> Result<(), String> {
    let app_dir = get_app_data_dir()?;
    fs::create_dir_all(&app_dir).map_err(|e| format!("Failed to create app directory: {}", e))?;

    let credentials_path = app_dir.join("credentials.json");
    let credentials = Credentials {
        server_url,
        username,
        token,
        user_id,
    };
    let json = serde_json::to_string_pretty(&credentials)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    fs::write(&credentials_path, json).map_err(|e| format!("Failed to save credentials: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_saved_credentials() -> Result<Option<Credentials>, String> {
    let app_dir = get_app_data_dir()?;
    let credentials_path = app_dir.join("credentials.json");

    if !credentials_path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&credentials_path)
        .map_err(|e| format!("Failed to read credentials: {}", e))?;

    let credentials: Credentials =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse credentials: {}", e))?;

    Ok(Some(credentials))
}

#[tauri::command]
async fn get_music_library(server_url: String, token: String) -> Result<Vec<MusicItem>, String> {
    // Try to get from cache first
    match db::get_cached_music_library() {
        Ok(items) if !items.is_empty() => {
            return Ok(items);
        }
        _ => {
            // Cache is empty or there was an error, proceed to fetch from server
        }
    }

    let client = reqwest::Client::new();
    let library_url = format!(
        "{}/Items?IncludeItemTypes=Audio&Recursive=true&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate,AlbumArtists",
        server_url.trim_end_matches('/')
    );

    let response = client
        .get(&library_url)
        .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch library: HTTP {}",
            response.status()
        ));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let items = response_json["Items"]
        .as_array()
        .ok_or("Invalid response format")?
        .iter()
        .map(|item| {
            let duration_ticks = item["RunTimeTicks"].as_i64();
            let duration_seconds = duration_ticks.map(|ticks| ticks as f64 / 10_000_000.0);

            let item_id = item["Id"].as_str().unwrap_or("").to_string();
            let mut album_art_url = None;

            let image_id = item["AlbumId"].as_str().unwrap_or(&item_id);

            if let Some(tags) = item["ImageTags"].as_object() {
                if tags.contains_key("Primary") {
                    album_art_url = Some(format!(
                        "{}/Items/{}/Images/Primary",
                        server_url.trim_end_matches('/'),
                        image_id
                    ));
                }
            }

            // Extract artists from the "Artists" array of strings
            let artists_vec = item["Artists"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .filter(|v| !v.is_empty());

            // Extract artist IDs from the "ArtistItems" array of objects
            let artist_ids_vec = item["ArtistItems"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v["Id"].as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .filter(|v| !v.is_empty());

            let genres_vec = item["Genres"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .filter(|v| !v.is_empty());

            let album_artists: Option<Vec<NameIdPair>> = item["AlbumArtists"]
                .as_array()
                .and_then(|arr| serde_json::from_value(serde_json::Value::Array(arr.clone())).ok());

            let path_str = item["Path"].as_str();
            let container = path_str.and_then(|p| {
                std::path::Path::new(p)
                    .extension()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s.to_lowercase())
            });

            Ok(MusicItem {
                id: item_id,
                name: item["Name"].as_str().unwrap_or("").to_string(),
                item_type: item["Type"].as_str().unwrap_or("").to_string(),
                album: item["Album"].as_str().map(|s| s.to_string()),
                artists: artists_vec,
                artist_ids: artist_ids_vec,
                path: path_str.map(|s| s.to_string()),
                duration: duration_seconds,
                album_art_url,
                year: item["ProductionYear"].as_i64(),
                play_count: item["UserData"]["PlayCount"].as_i64(),
                is_favorite: item["UserData"]["IsFavorite"].as_bool(),
                track_number: item["IndexNumber"].as_i64().and_then(|n| n.try_into().ok()),
                container,
                genres: genres_vec,
                premiere_date: item["PremiereDate"].as_str().map(|s| s.to_string()),
                date_played: item["UserData"]["LastPlayedDate"]
                    .as_str()
                    .map(|s| s.to_string()),
                album_artists,
                lyrics: None, // Initialize lyrics to None
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Cache the library
    if let Err(e) = db::cache_music_library(&items) {
        eprintln!("Failed to cache music library: {}", e);
    }

    Ok(items)
}

#[tauri::command]
async fn get_artist_details(
    server_url: String,
    token: String,
    user_id: String,
    artist_id: String,
) -> Result<ArtistInfo, String> {
    let client = reqwest::Client::new();
    let artist_url = format!(
        "{}/Users/{}/Items/{}?Fields=Overview,ProviderIds,CommunityRating",
        server_url.trim_end_matches('/'),
        user_id,
        artist_id
    );

    let response = client
        .get(&artist_url)
        .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch artist details: HTTP {}",
            response.status()
        ));
    }

    let mut artist_info: ArtistInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse artist details response: {}", e))?;

    // The image URL is not part of the response for a single item, so construct it manually
    if let Some(tags) = &artist_info.image_tags {
        if tags.as_object().is_some_and(|t| t.contains_key("Primary")) {
            artist_info.image_url = Some(format!(
                "{}/Items/{}/Images/Primary",
                server_url.trim_end_matches('/'),
                artist_info.id
            ));
        }
    }

    Ok(artist_info)
}

#[tauri::command]
async fn get_all_artists(server_url: String, token: String) -> Result<Vec<ArtistInfo>, String> {
    let client = reqwest::Client::new();
    let artists_url = format!(
        "{}/Artists?Recursive=true",
        server_url.trim_end_matches('/')
    );

    let response = client
        .get(&artists_url)
        .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch artists: HTTP {}",
            response.status()
        ));
    }

    let response_json: ArtistItem = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse artists response: {}", e))?;

    let artists_with_urls = response_json
        .items
        .into_iter()
        .map(|mut artist| {
            if let Some(tags) = &artist.image_tags {
                if tags.as_object().is_some_and(|t| t.contains_key("Primary")) {
                    artist.image_url = Some(format!(
                        "{}/Items/{}/Images/Primary",
                        server_url.trim_end_matches('/'),
                        artist.id
                    ));
                }
            }
            artist
        })
        .collect();

    Ok(artists_with_urls)
}

#[tauri::command]
async fn get_albums_with_songs(
    server_url: String,
    token: String,
) -> Result<Vec<AlbumWithSongs>, String> {
    // First get all songs
    let songs = get_music_library(server_url.clone(), token.clone()).await?;

    // Group songs by album
    let mut album_map: std::collections::HashMap<String, Vec<MusicItem>> =
        std::collections::HashMap::new();

    for song in songs {
        let album_name = song
            .album
            .clone()
            .unwrap_or_else(|| "Unknown Album".to_string());
        album_map.entry(album_name).or_default().push(song);
    }

    // Convert to AlbumWithSongs
    let mut albums_with_songs: Vec<AlbumWithSongs> = Vec::new();

    for (album_name, songs) in album_map {
        if songs.is_empty() {
            continue;
        }

        // Get primary artist info from first song
        let primary_song = &songs[0];
        let primary_artist_name = primary_song
            .artists
            .as_ref()
            .and_then(|artists| artists.first())
            .unwrap_or(&"Unknown Artist".to_string())
            .clone();

        let primary_artist_id = primary_song
            .artist_ids
            .as_ref()
            .and_then(|artist_ids| artist_ids.first())
            .cloned();

        let album_art_url = primary_song.album_art_url.clone();

        // Sort songs by track number
        let mut sorted_songs = songs;
        sorted_songs.sort_by(|a, b| {
            let a_track = a.track_number.unwrap_or(0);
            let b_track = b.track_number.unwrap_or(0);
            a_track.cmp(&b_track)
        });

        let album_with_songs = AlbumWithSongs {
            name: album_name,
            artist: primary_artist_name,
            artist_id: primary_artist_id,
            album_art_url,
            song_count: sorted_songs.len() as i32,
            songs: sorted_songs,
        };

        albums_with_songs.push(album_with_songs);
    }

    // Sort albums by name
    albums_with_songs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(albums_with_songs)
}

#[tauri::command]
async fn get_artists_with_songs(
    server_url: String,
    token: String,
    album_artists_only: bool,
) -> Result<Vec<ArtistWithSongs>, String> {
    // First get all songs
    let songs = get_music_library(server_url.clone(), token.clone()).await?;

    // Get all artists to get artist metadata
    let artists = get_all_artists(server_url, token).await?;

    // Create case-insensitive maps for artist name lookup
    let mut artist_name_to_info: std::collections::HashMap<String, ArtistInfo> =
        std::collections::HashMap::new();
    let mut artist_name_lower_to_original: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for artist in artists {
        let original_name = artist.name.clone();
        let lower_name = original_name.to_lowercase();

        artist_name_to_info.insert(original_name.clone(), artist);
        artist_name_lower_to_original.insert(lower_name, original_name);
    }

    // Debug: Log all artist names from the API
    println!("Found {} artists from API", artist_name_to_info.len());

    // Group songs by artist
    let mut artist_map: std::collections::HashMap<String, Vec<MusicItem>> =
        std::collections::HashMap::new();

    if album_artists_only {
        for song in &songs {
            if let Some(artists) = &song.album_artists {
                for artist in artists {
                    artist_map
                        .entry(artist.name.clone())
                        .or_default()
                        .push(song.clone());
                }
            }
        }
    } else {
        // Collect all unique artist names from songs for debugging
        let mut song_artist_names = std::collections::HashSet::new();

        for song in &songs {
            if let Some(artists) = &song.artists {
                for artist_name in artists {
                    song_artist_names.insert(artist_name.clone());
                    artist_map
                        .entry(artist_name.clone())
                        .or_default()
                        .push(song.clone());
                }
            } else {
                // Handle songs with no artist
                artist_map
                    .entry("Unknown Artist".to_string())
                    .or_default()
                    .push(song.clone());
            }
        }
        // Debug: Log all artist names from songs
        println!(
            "DEBUG: Found {} unique artists in songs",
            song_artist_names.len()
        );
        for artist_name in &song_artist_names {
            println!("DEBUG: Song Artist: '{}'", artist_name);
        }
    }

    // Convert to ArtistWithSongs
    let mut artists_with_songs: Vec<ArtistWithSongs> = Vec::new();

    for (artist_name, songs) in artist_map {
        if songs.is_empty() {
            continue;
        }

        // Get artist info using case-insensitive lookup
        let artist_info = artist_name_to_info.get(&artist_name).or_else(|| {
            // Try case-insensitive lookup
            let lower_name = artist_name.to_lowercase();
            artist_name_lower_to_original
                .get(&lower_name)
                .and_then(|original_name| artist_name_to_info.get(original_name))
        });

        // Debug: Log artists without matching info
        if artist_info.is_none() {
            println!(
                "DEBUG: No API match for artist: '{}' (tried case-insensitive)",
                artist_name
            );
        } else {
            println!(
                "DEBUG: Found API match for artist: '{}' -> ID: '{}'",
                artist_name,
                artist_info.unwrap().id
            );
        }

        let artist_with_songs = ArtistWithSongs {
            id: artist_info.map(|a| a.id.clone()).unwrap_or_default(),
            name: artist_name,
            song_count: songs.len() as i32,
            image_url: artist_info.and_then(|a| a.image_url.clone()),
            songs,
        };

        artists_with_songs.push(artist_with_songs);
    }

    // Sort artists by name
    artists_with_songs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(artists_with_songs)
}

#[tauri::command]
async fn get_audio_stream_url(
    server_url: String,
    token: String,
    item_id: String,
    container: Option<String>,
) -> Result<String, String> {
    // Only add static=true for formats that support seeking
    let supports_seeking = match container.as_deref() {
        Some("flac") | Some("mp3") | Some("aac") | Some("ogg") => true,
        _ => false, // ALAC and other formats don't work well with static=true
    };

    let stream_url = if supports_seeking {
        format!(
            "{}/Audio/{}/stream.flac?api_key={}&static=true",
            server_url.trim_end_matches('/'),
            item_id,
            token
        )
    } else {
        format!(
            "{}/Audio/{}/stream.flac?api_key={}",
            server_url.trim_end_matches('/'),
            item_id,
            token
        )
    };

    Ok(stream_url)
}

#[tauri::command]
async fn get_lyrics(artist: String, title: String) -> Result<String, String> {
    println!(
        "[get_lyrics] Searching for artist: '{}', title: '{}'",
        artist, title
    );
    let client = reqwest::Client::new();
    let search_url = "https://lrclib.net/api/search";

    let response = client
        .get(search_url)
        .query(&[("artist_name", &artist), ("track_name", &title)])
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    println!("[get_lyrics] API response status: {}", status);

    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        println!("[get_lyrics] API error body: {}", error_body);
        return Err(format!("Failed to search for lyrics: HTTP {}", status));
    }

    // Read the full response text for logging
    let response_text = response.text().await.map_err(|e| {
        let err_msg = format!("Failed to read response body: {}", e);
        println!("[get_lyrics] {}", err_msg);
        err_msg
    })?;

    println!("[get_lyrics] API response body: {}", response_text);

    // Try to parse it
    let search_results: Vec<LrcLibTrackResponse> =
        serde_json::from_str(&response_text).map_err(|e| {
            let err_msg = format!("Failed to parse search results: {}", e);
            println!("[get_lyrics] {}", err_msg);
            err_msg
        })?;

    println!(
        "[get_lyrics] Found {} search results.",
        search_results.len()
    );

    if search_results.is_empty() {
        println!("[get_lyrics] No lyrics found for '{}'", title);
        return Err("No lyrics found".to_string());
    }

    // Log all found lyrics, as requested by the user.
    for (i, track) in search_results.iter().enumerate() {
        println!("[get_lyrics] Result {}: {:?}", i, track);
        if let Some(lyrics) = &track.synced_lyrics {
            println!("[get_lyrics] Result {} has synced lyrics.", i);
            println!("[get_lyrics] Synced Lyrics (result {}): \n{}", i, lyrics);
        }
        if let Some(lyrics) = &track.plain_lyrics {
            println!("[get_lyrics] Result {} has plain lyrics.", i);
            println!("[get_lyrics] Plain Lyrics (result {}): \n{}", i, lyrics);
        }
    }

    // The original logic only took the first result. I will keep it that way.
    if let Some(track) = search_results.into_iter().next() {
        if let Some(lyrics) = track.synced_lyrics.or(track.plain_lyrics) {
            println!(
                "[get_lyrics] Returning lyrics from the first result for '{}'",
                title
            );
            return Ok(lyrics);
        }
    }

    println!(
        "[get_lyrics] No lyrics found in any results for '{}'",
        title
    );
    Err("No lyrics found".to_string())
}

#[tauri::command]
fn save_volume(volume: f64) -> Result<(), String> {
    let app_dir = get_app_data_dir()?;
    let volume_path = app_dir.join("volume.json");
    let json =
        serde_json::to_string(&volume).map_err(|e| format!("Failed to serialize volume: {}", e))?;
    fs::write(&volume_path, json).map_err(|e| format!("Failed to save volume: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_saved_volume() -> Result<Option<f64>, String> {
    let app_dir = get_app_data_dir()?;
    let volume_path = app_dir.join("volume.json");

    if !volume_path.exists() {
        return Ok(None);
    }

    let json =
        fs::read_to_string(&volume_path).map_err(|e| format!("Failed to read volume: {}", e))?;
    let volume: f64 =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse volume: {}", e))?;
    Ok(Some(volume))
}

#[tauri::command]
fn clear_music_cache() -> Result<(), String> {
    println!("DEBUG: Starting cache clear...");
    let result = db::clear_music_cache();
    match &result {
        Ok(_) => println!("DEBUG: Cache cleared successfully"),
        Err(e) => println!("DEBUG: Cache clear failed: {}", e),
    }
    result
}

#[tauri::command]
async fn toggle_favorite_status(
    server_url: String,
    token: String,
    user_id: String,
    item_id: String,
    is_favorite: bool,
) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let fav_url = format!(
        "{}/Users/{}/FavoriteItems/{}",
        server_url.trim_end_matches('/'),
        user_id,
        item_id
    );

    let response = if is_favorite {
        // If it's currently a favorite, we want to unfavorite it
        client
            .delete(&fav_url)
            .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
    } else {
        // If it's not a favorite, we want to favorite it
        client
            .post(&fav_url)
            .header("Authorization", format!("MediaBrowser Token=\"{}\"", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
    };

    if !response.status().is_success() {
        return Err(format!(
            "Failed to toggle favorite status: HTTP {}",
            response.status()
        ));
    }

    // The API returns the new user data, we can just return the opposite of the previous state
    Ok(!is_favorite)
}

fn get_app_data_dir() -> Result<PathBuf, String> {
    let mut app_dir = dirs::data_dir().ok_or("Failed to get data directory")?;
    app_dir.push("JellyfinMusicPlayer");
    Ok(app_dir)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the database
    if let Err(e) = db::initialize_database() {
        // Handle error appropriately, maybe log it
        eprintln!("Failed to initialize database: {}", e);
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            login_to_jellyfin,
            save_credentials,
            get_saved_credentials,
            get_music_library,
            get_all_artists,
            get_artist_details,
            get_albums_with_songs,
            get_artists_with_songs,
            get_audio_stream_url,
            save_volume,
            get_saved_volume,
            toggle_favorite_status,
            clear_music_cache,
            get_lyrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
