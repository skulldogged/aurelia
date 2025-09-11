//! Database module for caching music library data
use crate::models::{Album, NameIdPair, Song};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Database pool, wrapped in a RwLock to allow for re-initialization
static DB_POOL: RwLock<Option<SqlitePool>> = RwLock::const_new(None);

/// Get the path to the database file
fn get_db_path() -> PathBuf {
    crate::utils::get_app_data_dir()
        .expect("Failed to get app data dir")
        .join("library.db")
}

/// Helper function to initialize a new connection pool and run migrations
async fn initialize_pool() -> Result<SqlitePool, sqlx::Error> {
    let db_path = get_db_path();
    let db_url = format!("sqlite:{}", db_path.to_str().unwrap());

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

/// Gets the existing connection pool, or initializes it if it doesn't exist.
async fn get_pool() -> SqlitePool {
    // Fast path with read lock
    let read_guard = DB_POOL.read().await;
    if let Some(pool) = read_guard.as_ref() {
        return pool.clone();
    }
    drop(read_guard);

    // Slow path with write lock to initialize
    let mut write_guard = DB_POOL.write().await;
    // Check again in case another thread initialized while waiting for the write lock
    if let Some(pool) = write_guard.as_ref() {
        return pool.clone();
    }

    let pool = initialize_pool()
        .await
        .expect("Failed to initialize database pool");
    *write_guard = Some(pool.clone());
    pool
}

/// Initialize the database and create tables if they don't exist
pub async fn initialize_database() -> Result<(), sqlx::Error> {
    get_pool().await;
    Ok(())
}

/// Cache the entire music library
pub async fn cache_music_library(items: &[Song]) -> Result<(), sqlx::Error> {
    let pool = get_pool().await;
    let mut tx = pool.begin().await?;

    // --- 1. Gather all unique artists and albums ---
    let mut artists_to_cache = std::collections::HashMap::new();
    let mut albums_to_cache = std::collections::HashMap::new();

    for item in items {
        // Gather track artists
        if let (Some(names), Some(ids)) = (&item.artists, &item.artist_ids) {
            for (name, id) in names.iter().zip(ids.iter()) {
                artists_to_cache
                    .entry(id.clone())
                    .or_insert_with(|| name.clone());
            }
        }
        // Gather album artists
        if let Some(album_artists) = &item.album_artists {
            for artist in album_artists {
                artists_to_cache
                    .entry(artist.id.clone())
                    .or_insert_with(|| artist.name.clone());
            }
        }
        // Gather albums
        if let (Some(album_id), Some(album_name)) = (&item.album_id, &item.album) {
            let album_artist_id = item
                .album_artists
                .as_ref()
                .and_then(|aa| aa.first())
                .map(|a| a.id.clone());

            albums_to_cache.entry(album_id.clone()).or_insert_with(|| {
                (
                    album_name.clone(),
                    item.album_art_url.clone(),
                    album_artist_id,
                )
            });
        }
    }

    // --- 2. Insert all unique artists first ---
    for (id, name) in artists_to_cache {
        sqlx::query!(
            "INSERT OR IGNORE INTO artists (id, name, image_tag) VALUES (?, ?, ?)",
            id,
            name,
            None::<String>
        )
        .execute(&mut *tx)
        .await?;
    }

    // --- 3. Insert all unique albums second ---
    for (id, (name, art_url, artist_id)) in albums_to_cache {
        sqlx::query!(
            "INSERT OR IGNORE INTO albums (id, name, album_art_url, artist_id) VALUES (?, ?, ?, ?)",
            id,
            name,
            art_url,
            artist_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // --- 4. Insert songs and their relationships ---
    for item in items {
        // Insert song
        sqlx::query!(
            "INSERT OR REPLACE INTO songs (id, name, item_type, album_id, path, duration, album_art_url, year, play_count, is_favorite, track_number, container, premiere_date, date_played, date_created, lyrics, bit_rate, sample_rate, codec)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            item.id, item.name, item.item_type, item.album_id, item.path, item.duration, item.album_art_url,
            item.year, item.play_count, item.is_favorite, item.track_number, item.container,
            item.premiere_date, item.date_played, item.date_created, item.lyrics, item.bit_rate,
            item.sample_rate, item.codec,
        )
        .execute(&mut *tx)
        .await?;

        // Link song to track artists
        if let (Some(_), Some(artist_ids)) = (&item.artists, &item.artist_ids) {
            for artist_id in artist_ids {
                sqlx::query!(
                    "INSERT OR IGNORE INTO song_artists (song_id, artist_id) VALUES (?, ?)",
                    item.id,
                    artist_id,
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        // Link song to album artists
        if let Some(artists) = &item.album_artists {
            for artist in artists {
                sqlx::query!(
                    "INSERT OR IGNORE INTO song_album_artists (song_id, artist_id) VALUES (?, ?)",
                    item.id,
                    artist.id,
                )
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await
}

/// Get the cached music library
pub async fn get_cached_music_library() -> Result<Vec<Song>, sqlx::Error> {
    let pool = get_pool().await;

    let rows = sqlx::query!(
        r#"
        SELECT s.id, s.name, s.item_type, s.album_id, s.path, s.duration, s.album_art_url, s.year,
               s.play_count, s.is_favorite, s.track_number, s.container, s.premiere_date, s.date_played,
               s.date_created, s.lyrics, s.bit_rate, s.sample_rate, s.codec,
               GROUP_CONCAT(a.name, '\x1F') as artists,
               GROUP_CONCAT(a.id, '\x1F') as artist_ids,
               (SELECT GROUP_CONCAT(aa.name, '\x1F') FROM song_album_artists saa JOIN artists aa ON saa.artist_id = aa.id WHERE saa.song_id = s.id) as album_artist_names,
               (SELECT GROUP_CONCAT(aa.id, '\x1F') FROM song_album_artists saa JOIN artists aa ON saa.artist_id = aa.id WHERE saa.song_id = s.id) as album_artist_ids,
               (SELECT name FROM albums WHERE id = s.album_id) as album_name
        FROM songs s
        LEFT JOIN song_artists sa ON s.id = sa.song_id
        LEFT JOIN artists a ON sa.artist_id = a.id
        GROUP BY s.id
        "#
    )
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        let artists = row.artists.map(|s| {
            s.split('\x1F')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        });

        let artist_ids = row.artist_ids.map(|s| {
            s.split('\x1F')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        });

        let album_artists =
            row.album_artist_names
                .zip(row.album_artist_ids)
                .map(|(names_str, ids_str)| {
                    let names: Vec<&str> = names_str.split('\x1F').collect();
                    let ids: Vec<&str> = ids_str.split('\x1F').collect();
                    names
                        .into_iter()
                        .zip(ids.into_iter())
                        .filter(|(name, id)| !name.is_empty() && !id.is_empty())
                        .map(|(name, id)| NameIdPair {
                            name: name.to_string(),
                            id: id.to_string(),
                        })
                        .collect()
                });

        result.push(Song {
            id: row.id,
            name: row.name,
            item_type: row.item_type.unwrap_or_default(),
            album: row.album_name,
            album_id: row.album_id,
            path: row.path,
            duration: row.duration.map(|d| d as f64),
            album_art_url: row.album_art_url,
            year: row.year.map(|y| y as i32),
            play_count: row.play_count.map(|pc| pc as i32),
            is_favorite: row.is_favorite,
            track_number: row.track_number.map(|tn| tn as i32),
            container: row.container,
            premiere_date: row.premiere_date,
            date_played: row.date_played,
            date_created: row.date_created,
            artists,
            artist_ids,
            genres: None, // Implement genre caching if needed
            album_artists,
            lyrics: row.lyrics,
            bit_rate: row.bit_rate.map(|br| br as i32),
            sample_rate: row.sample_rate.map(|sr| sr as i32),
            codec: row.codec,
        });
    }
    Ok(result)
}

/// Cache full artist data including image_tags
pub async fn cache_artists(artists: &[crate::models::Artist]) -> Result<(), sqlx::Error> {
    let pool = get_pool().await;
    let mut tx = pool.begin().await?;

    for artist in artists {
        // Serialize image_tags to JSON string for storage
        let image_tag_json = artist
            .image_tags
            .as_ref()
            .and_then(|tags| serde_json::to_string(tags).ok());

        sqlx::query!(
            "INSERT OR REPLACE INTO artists (id, name, image_tag) VALUES (?, ?, ?)",
            artist.id,
            artist.name,
            image_tag_json
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

/// Get cached artist data
pub async fn get_cached_artists() -> Result<Vec<crate::models::Artist>, sqlx::Error> {
    let pool = get_pool().await;

    let rows = sqlx::query!("SELECT id, name, image_tag FROM artists ORDER BY name")
        .fetch_all(&pool)
        .await?;

    let mut result = Vec::new();
    for row in rows {
        // Parse image_tag JSON back to serde_json::Value
        let image_tags = match row.image_tag {
            Some(json_str) => serde_json::from_str::<serde_json::Value>(&json_str).ok(),
            None => None,
        };

        // Compute image_url from image_tags if available
        let image_url = if let Some(tags) = &image_tags {
            if let Some(tags_obj) = tags.as_object() {
                if tags_obj.contains_key("Primary") {
                    // Note: We can't generate the full URL here without server_url and token
                    // This would need to be handled by the frontend or a separate function
                    None
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let artist = crate::models::Artist {
            id: row.id,
            name: row.name,
            image_tags,
            image_url,
            overview: None,
            provider_ids: None,
            community_rating: None,
            song_count: None,
            songs: None,
        };

        result.push(artist);
    }

    Ok(result)
}

/// Clear the music cache by deleting the database file
pub async fn clear_music_cache() -> Result<(), String> {
    println!("DEBUG: DB clear_music_cache called");
    let mut write_guard = DB_POOL.write().await;

    // Take the pool out of the Option, which also closes the connection
    if let Some(pool) = write_guard.take() {
        pool.close().await;
    }

    let db_path = get_db_path();
    if db_path.exists() {
        if let Err(e) = std::fs::remove_file(&db_path) {
            let err_msg = format!("Failed to delete database file {:?}: {}", db_path, e);
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
        println!("DEBUG: Database file deleted successfully.");
    }

    // The pool will be re-initialized on the next call to `get_pool()`
    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct AlbumFromDb {
    id: Option<String>,
    name: String,
    artist: Option<String>,
    artist_id: Option<String>,
    album_art_url: Option<String>,
    song_count: i32,
}

/// Get all albums from the cache
pub async fn get_all_albums() -> Result<Vec<Album>, sqlx::Error> {
    let pool = get_pool().await;
    let albums_from_db = sqlx::query_as!(
        AlbumFromDb,
        r#"
        SELECT
            a.id,
            a.name,
            ar.name as "artist",
            a.artist_id,
            a.album_art_url,
            (SELECT COUNT(*) FROM songs s WHERE s.album_id = a.id) as "song_count: i32"
        FROM albums a
        LEFT JOIN artists ar ON a.artist_id = ar.id
        ORDER BY a.name
        "#
    )
    .fetch_all(&pool)
    .await?;

    let albums = albums_from_db
        .into_iter()
        .map(|a| Album {
            id: a.id,
            name: a.name,
            artist: a.artist.unwrap_or_else(|| "Unknown Artist".to_string()),
            artist_id: a.artist_id,
            album_art_url: a.album_art_url,
            song_count: a.song_count,
            songs: None,
        })
        .collect();

    Ok(albums)
}
