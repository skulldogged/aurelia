use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct Credentials {
    #[serde(rename = "serverUrl")]
    server_url: String,
    username: String,
    token: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct JellyfinUser {
    #[serde(rename = "Id")]
    id: String,
}

#[derive(Serialize, Deserialize)]
struct JellyfinAuthResponse {
    #[serde(rename = "AccessToken")]
    access_token: String,
    #[serde(rename = "User")]
    user: JellyfinUser,
}

#[derive(Serialize, Deserialize)]
struct MusicItem {
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
struct ArtistItem {
    #[serde(rename = "Items")]
    items: Vec<ArtistInfo>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
    let client = reqwest::Client::new();
    let library_url = format!(
        "{}/Items?IncludeItemTypes=Audio&Recursive=true&Fields=Path,ParentId,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,ArtistItems,IndexNumber,Genres,PremiereDate",
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
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

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
async fn get_audio_stream_url(
    server_url: String,
    token: String,
    item_id: String,
) -> Result<String, String> {
    let stream_url = format!(
        "{}/Audio/{}/stream.flac?api_key={}",
        server_url.trim_end_matches('/'),
        item_id,
        token
    );
    Ok(stream_url)
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            login_to_jellyfin,
            save_credentials,
            get_saved_credentials,
            get_music_library,
            get_all_artists,
            get_artist_details,
            get_audio_stream_url,
            save_volume,
            get_saved_volume,
            toggle_favorite_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
