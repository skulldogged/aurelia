use crate::MusicItem;
use once_cell::sync::Lazy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result};
use std::path::PathBuf;

pub type DbPool = Pool<SqliteConnectionManager>;

fn get_db_path() -> PathBuf {
    let mut db_path = crate::get_app_data_dir().expect("Failed to get app data dir");
    db_path.push("library.db");
    db_path
}

pub static DB_POOL: Lazy<DbPool> = Lazy::new(|| {
    let manager = SqliteConnectionManager::file(get_db_path());
    Pool::new(manager).expect("Failed to create DB pool")
});

pub fn initialize_database() -> Result<()> {
    let conn = DB_POOL
        .get()
        .expect("Failed to get DB connection from pool");
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS songs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            item_type TEXT NOT NULL,
            album TEXT,
            path TEXT,
            duration REAL,
            album_art_url TEXT,
            year INTEGER,
            play_count INTEGER,
            is_favorite BOOLEAN,
            track_number INTEGER,
            container TEXT,
            premiere_date TEXT,
            date_played TEXT,
            lyrics TEXT
        );
        CREATE TABLE IF NOT EXISTS artists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS song_artists (
            song_id TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            PRIMARY KEY (song_id, artist_id),
            FOREIGN KEY (song_id) REFERENCES songs (id),
            FOREIGN KEY (artist_id) REFERENCES artists (id)
        );
        CREATE TABLE IF NOT EXISTS song_album_artists (
            song_id TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            PRIMARY KEY (song_id, artist_id),
            FOREIGN KEY (song_id) REFERENCES songs (id),
            FOREIGN KEY (artist_id) REFERENCES artists (id)
        );
        ",
    )?;
    Ok(())
}

pub fn cache_music_library(items: &[MusicItem]) -> Result<()> {
    let mut conn = DB_POOL
        .get()
        .expect("Failed to get DB connection from pool");

    // Start a transaction for bulk inserts
    let tx = conn.transaction()?;

    for item in items {
        // Insert or replace song
        tx.execute(
            "INSERT OR REPLACE INTO songs (id, name, item_type, album, path, duration, album_art_url, year, play_count, is_favorite, track_number, container, premiere_date, date_played, lyrics)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                item.id,
                item.name,
                item.item_type,
                item.album,
                item.path,
                item.duration,
                item.album_art_url,
                item.year,
                item.play_count,
                item.is_favorite,
                item.track_number,
                item.container,
                item.premiere_date,
                item.date_played,
                item.lyrics,
            ],
        )?;

        // Insert artists and link to song
        if let Some(artists) = &item.artists {
            if let Some(artist_ids) = &item.artist_ids {
                for (i, artist_name) in artists.iter().enumerate() {
                    if let Some(artist_id) = artist_ids.get(i) {
                        // Insert or ignore artist
                        tx.execute(
                            "INSERT OR IGNORE INTO artists (id, name) VALUES (?1, ?2)",
                            params![artist_id, artist_name],
                        )?;

                        // Link song to artist
                        tx.execute(
                            "INSERT OR IGNORE INTO song_artists (song_id, artist_id) VALUES (?1, ?2)",
                            params![item.id, artist_id],
                        )?;
                    }
                }
            }
        }

        // Insert album artists and link to song
        if let Some(artists) = &item.album_artists {
            for artist in artists {
                tx.execute(
                    "INSERT OR IGNORE INTO artists (id, name) VALUES (?1, ?2)",
                    params![artist.id, artist.name],
                )?;
                tx.execute(
                    "INSERT OR IGNORE INTO song_album_artists (song_id, artist_id) VALUES (?1, ?2)",
                    params![item.id, artist.id],
                )?;
            }
        }
    }

    tx.commit()
}

pub fn get_cached_music_library() -> Result<Vec<MusicItem>> {
    let conn = DB_POOL
        .get()
        .expect("Failed to get DB connection from pool");

    // Use a separator that's unlikely to appear in artist names (Unit Separator \x1F)
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.item_type, s.album, s.path, s.duration, s.album_art_url, s.year,
                s.play_count, s.is_favorite, s.track_number, s.container, s.premiere_date, s.date_played,
                s.lyrics,
                GROUP_CONCAT(DISTINCT a.name, '\x1F') as artists,
                GROUP_CONCAT(DISTINCT a.id, '\x1F') as artist_ids,
                GROUP_CONCAT(DISTINCT aa.name, '\x1F') as album_artist_names,
                GROUP_CONCAT(DISTINCT aa.id, '\x1F') as album_artist_ids
         FROM songs s
         LEFT JOIN song_artists sa ON s.id = sa.song_id
         LEFT JOIN artists a ON sa.artist_id = a.id
         LEFT JOIN song_album_artists saa ON s.id = saa.song_id
         LEFT JOIN artists aa ON saa.artist_id = aa.id
         GROUP BY s.id",
    )?;

    let music_items = stmt.query_map([], |row| {
        let artists_str: Option<String> = row.get(15)?;
        let artist_ids_str: Option<String> = row.get(16)?;
        let album_artist_names_str: Option<String> = row.get(17)?;
        let album_artist_ids_str: Option<String> = row.get(18)?;

        let artists = artists_str
            .map(|s| {
                s.split('\x1F')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect::<Vec<String>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty());

        let artist_ids = artist_ids_str
            .map(|s| {
                s.split('\x1F')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect::<Vec<String>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty());

        let album_artists = album_artist_names_str
            .zip(album_artist_ids_str)
            .map(|(names_str, ids_str)| {
                let names: Vec<&str> = names_str.split('\x1F').collect();
                let ids: Vec<&str> = ids_str.split('\x1F').collect();
                names
                    .into_iter()
                    .zip(ids.into_iter())
                    .filter(|(name, id)| !name.is_empty() && !id.is_empty())
                    .map(|(name, id)| crate::NameIdPair {
                        name: name.to_string(),
                        id: id.to_string(),
                    })
                    .collect::<Vec<crate::NameIdPair>>()
            })
            .filter(|v| !v.is_empty());

        Ok(MusicItem {
            id: row.get(0)?,
            name: row.get(1)?,
            item_type: row.get(2)?,
            album: row.get(3)?,
            path: row.get(4)?,
            duration: row.get(5)?,
            album_art_url: row.get(6)?,
            year: row.get(7)?,
            play_count: row.get(8)?,
            is_favorite: row.get(9)?,
            track_number: row.get(10)?,
            container: row.get(11)?,
            premiere_date: row.get(12)?,
            date_played: row.get(13)?,
            artists,
            artist_ids,
            genres: None, // You might want to implement genre caching as well
            album_artists,
            lyrics: row.get(14)?,
        })
    })?;

    let mut result = Vec::new();
    for item in music_items {
        result.push(item?);
    }
    Ok(result)
}

pub fn clear_music_cache() -> Result<(), String> {
    println!("DEBUG: DB clear_music_cache called");
    let conn = DB_POOL
        .get()
        .expect("Failed to get DB connection from pool");

    // Check counts before deletion
    let songs_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM songs", [], |row| row.get(0))
        .unwrap_or(0);
    let artists_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM artists", [], |row| row.get(0))
        .unwrap_or(0);
    let song_artists_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM song_artists", [], |row| row.get(0))
        .unwrap_or(0);
    let song_album_artists_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM song_album_artists", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    println!(
        "DEBUG: Before clear - Songs: {}, Artists: {}, SongArtists: {}, SongAlbumArtists: {}",
        songs_count, artists_count, song_artists_count, song_album_artists_count
    );

    let result = conn
        .execute_batch(
            "
        DELETE FROM song_artists;
        DELETE FROM song_album_artists;
        DELETE FROM artists;
        DELETE FROM songs;
        ",
        )
        .map_err(|e| format!("Failed to clear music cache: {}", e));

    match &result {
        Ok(_) => {
            // Check counts after deletion
            let songs_count_after: i64 = conn
                .query_row("SELECT COUNT(*) FROM songs", [], |row| row.get(0))
                .unwrap_or(0);
            let artists_count_after: i64 = conn
                .query_row("SELECT COUNT(*) FROM artists", [], |row| row.get(0))
                .unwrap_or(0);
            let song_artists_count_after: i64 = conn
                .query_row("SELECT COUNT(*) FROM song_artists", [], |row| row.get(0))
                .unwrap_or(0);
            let song_album_artists_count_after: i64 = conn
                .query_row("SELECT COUNT(*) FROM song_album_artists", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);

            println!(
                "DEBUG: After clear - Songs: {}, Artists: {}, SongArtists: {}, SongAlbumArtists: {}",
                songs_count_after,
                artists_count_after,
                song_artists_count_after,
                song_album_artists_count_after
            );
            println!("DEBUG: DB cache cleared successfully");
        }
        Err(e) => println!("DEBUG: DB cache clear failed: {}", e),
    }

    result
}
