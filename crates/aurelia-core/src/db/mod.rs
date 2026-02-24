pub mod repositories;
pub mod schema;

pub use repositories::*;
pub use schema::*;

use crate::models::{Album, Artist, Song};
use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use redb::{Database, ReadOnlyTable, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

#[derive(Clone)]
struct DatabaseHandle {
    app_data_dir: PathBuf,
    db: Arc<Database>,
}

static DB: Lazy<Mutex<Option<DatabaseHandle>>> = Lazy::new(|| Mutex::new(None));

// Table definitions
const SONGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("songs");
const ARTISTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("artists");
const ALBUMS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("albums");

pub fn init(app_data_dir: &PathBuf) -> Result<()> {
    {
        let guard = DB
            .lock()
            .map_err(|_| anyhow!("Failed to lock database handle"))?;
        if let Some(existing) = guard.as_ref()
            && existing.app_data_dir == *app_data_dir
        {
            debug!("Database already initialized for {:?}, skipping", app_data_dir);
            return Ok(());
        }
    }

    info!("Database path: {:?}", app_data_dir);

    let db_path = app_data_dir.join("aurelia.redb");
    debug!("Full database path: {:?}", db_path);

    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| anyhow!("Failed to create app data directory: {}", e))?;

    let db =
        Arc::new(Database::create(&db_path).map_err(|e| anyhow!("Failed to create database: {}", e))?);

    // Initialize all tables
    let write_txn = db
        .begin_write()
        .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
    {
        // Primary tables
        let _ = write_txn.open_table(schema::SONGS)?;
        let _ = write_txn.open_table(schema::ARTISTS)?;
        let _ = write_txn.open_table(schema::ALBUMS)?;
        let _ = write_txn.open_table(schema::PLAYLISTS)?;

        // Index tables
        let _ = write_txn.open_table(schema::SONGS_BY_ALBUM)?;
        let _ = write_txn.open_table(schema::SONGS_BY_ARTIST)?;
        let _ = write_txn.open_table(schema::ALBUMS_BY_ARTIST)?;

        // Metadata tables
        let _ = write_txn.open_table(schema::FAVORITES)?;
        let _ = write_txn.open_table(schema::SYNC_STATE)?;
        let _ = write_txn.open_table(schema::CREDENTIALS)?;
    }
    write_txn.commit()?;

    let mut guard = DB
        .lock()
        .map_err(|_| anyhow!("Failed to lock database handle"))?;
    *guard = Some(DatabaseHandle {
        app_data_dir: app_data_dir.clone(),
        db,
    });

    info!("Database initialized successfully");
    Ok(())
}

pub fn get() -> Result<Arc<Database>> {
    let guard = DB
        .lock()
        .map_err(|_| anyhow!("Failed to lock database handle"))?;
    guard
        .as_ref()
        .map(|handle| handle.db.clone())
        .ok_or_else(|| anyhow!("Database not initialized"))
}

// ============================================================================
// Sync functions
// ============================================================================

fn get_all_items<T: DeserializeOwned>(table: &ReadOnlyTable<&str, &[u8]>) -> Result<Vec<T>> {
    debug!("Getting all items from table");
    let items: Vec<T> = table
        .iter()
        .map_err(|e| anyhow!("Failed to iterate over table: {}", e))?
        .map(|res| {
            let (_, bytes) = res.map_err(|e| anyhow!("Failed to get table item: {}", e))?;
            let item = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode table item: {}", e))?;
            Ok(item)
        })
        .collect::<Result<Vec<T>>>()?;
    debug!("Retrieved {} items from table", items.len());
    Ok(items)
}

pub(crate) fn clear_table(table: &mut redb::Table<&str, &[u8]>) -> Result<()> {
    let mut keys = Vec::new();
    for result in table
        .iter()
        .map_err(|e| anyhow!("Failed to iterate over table: {}", e))?
    {
        let (key, _) = result.map_err(|e| anyhow!("Failed to get table item: {}", e))?;
        keys.push(key.value().to_string());
    }
    for key in keys {
        table
            .remove(key.as_str())
            .map_err(|e| anyhow!("Failed to remove table item: {}", e))?;
    }
    Ok(())
}

pub fn sync_all(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<()> {
    let db = get()?;

    let service = crate::domain::services::LibraryService::new(db);
    service
        .sync_library(songs, artists, albums, true)
        .map_err(|e| anyhow!("Sync failed: {}", e))?;

    Ok(())
}

/// Incremental sync - only updates items that have changed
/// Falls back to full sync if no previous sync data exists
pub fn sync_incremental(songs: &[Song], artists: &[Artist], albums: &[Album]) -> Result<bool> {
    let db = get()?;
    let service = crate::domain::services::LibraryService::new(db);

    let (song_count, artist_count, _album_count) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    if song_count == 0 && artist_count == 0 {
        info!("No existing library data found, performing full sync");
        service
            .sync_library(songs, artists, albums, true)
            .map_err(|e| anyhow!("Full sync failed: {}", e))?;
        return Ok(true);
    }

    let delta = service
        .compute_delta(songs, artists, albums)
        .map_err(|e| anyhow!("Failed to compute sync delta: {}", e))?;

    let total_items = songs.len() + artists.len() + albums.len();
    let threshold = total_items / 2;
    if delta.total_changes() > threshold {
        info!(
            "Delta too large ({} changes out of {} items), performing full sync",
            delta.total_changes(),
            total_items
        );
        service
            .sync_library(songs, artists, albums, true)
            .map_err(|e| anyhow!("Full sync failed: {}", e))?;
        return Ok(true);
    }

    if delta.is_empty() {
        info!("No changes detected, library is up to date");
    } else {
        info!(
            "Applying incremental sync: {} changes",
            delta.total_changes()
        );
        service
            .apply_delta(&delta, songs, artists, albums)
            .map_err(|e| anyhow!("Delta sync failed: {}", e))?;
    }

    Ok(false)
}

/// Songs-only incremental sync - for hybrid lazy-load approach
pub fn sync_songs_only(songs: &[Song]) -> Result<bool> {
    let db = get()?;
    let service = crate::domain::services::LibraryService::new(db);

    let (song_count, _, _) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    if song_count == 0 {
        info!("No existing song data found, performing full songs sync");
        service
            .sync_library(songs, &[], &[], true)
            .map_err(|e| anyhow!("Songs sync failed: {}", e))?;
        return Ok(true);
    }

    let delta = service
        .compute_delta(songs, &[], &[])
        .map_err(|e| anyhow!("Failed to compute song delta: {}", e))?;

    if delta.songs_to_add.is_empty()
        && delta.songs_to_remove.is_empty()
        && delta.songs_to_update.is_empty()
    {
        info!("No song changes detected, library is up to date");
    } else {
        info!(
            "Applying incremental song sync: {} adds, {} removes, {} updates",
            delta.songs_to_add.len(),
            delta.songs_to_remove.len(),
            delta.songs_to_update.len()
        );
        service
            .apply_delta(&delta, songs, &[], &[])
            .map_err(|e| anyhow!("Song delta sync failed: {}", e))?;
    }

    Ok(false)
}

/// Update favorite status for all songs based on a list of favorite IDs
pub fn update_songs_favorite_status(_app_data_dir: &PathBuf, favorite_ids: &[String]) -> Result<u32> {
    let db = get()?;
    let songs_repo = crate::db::repositories::SongRepository::new(db);
    
    let all_songs = songs_repo.get_all()?;
    let mut updated_count = 0;
    
    let favorite_set: std::collections::HashSet<_> = favorite_ids.iter().collect();
    
    for song in all_songs {
        let is_favorite = favorite_set.contains(&song.id);
        // Only update if the status is different or was None
        if song.is_favorite != Some(is_favorite) {
            songs_repo.update_favorite_status(&song.id, is_favorite)?;
            updated_count += 1;
        }
    }
    
    info!("Updated favorite status for {} songs", updated_count);
    Ok(updated_count)
}

// ============================================================================
// Smart Sync (paginated + incremental)
// ============================================================================

use crate::domain::models::{SyncProgress, SyncReport, SyncState};
use crate::services::JellyfinClient;

/// Global in-memory sync progress, updated after each page during a full sync.
/// Polled by the UI via `get_sync_progress()`.
pub static SYNC_PROGRESS: Lazy<Mutex<SyncProgress>> =
    Lazy::new(|| Mutex::new(SyncProgress::default()));

/// Page size for paginated sync fetches.
/// 200 balances round-trip count against progress granularity — small enough
/// to give smooth UI updates, large enough to keep the total request count reasonable.
const SYNC_PAGE_SIZE: usize = 200;

/// Perform a smart sync: decides between full (paginated) or incremental sync
/// based on the existing SyncState. Handles resumability for interrupted full syncs.
///
/// Returns a SyncReport with stats about what was changed.
pub async fn sync_smart(client: &JellyfinClient, user_id: &str) -> Result<SyncReport> {
    let db = get()?;
    let service = crate::domain::services::LibraryService::new(db);
    let state = service
        .get_sync_state()
        .map_err(|e| anyhow!("Failed to get sync state: {}", e))?;

    let start = std::time::Instant::now();

    // Reset progress for UI polling
    if let Ok(mut p) = SYNC_PROGRESS.lock() {
        *p = SyncProgress::default();
    }

    // Determine sync strategy
    let is_first_sync = state.last_sync_time == "1970-01-01T00:00:00Z";
    let library_empty = state.song_count == 0;
    
    tracing::info!("sync_smart: is_first_sync={}, library_empty={}, last_sync_time={}, full_sync_in_progress={}", 
        is_first_sync, library_empty, state.last_sync_time, state.full_sync_in_progress);

    // Also do full sync if the library is empty (song_count == 0)
    if state.full_sync_in_progress || is_first_sync || library_empty {
        // Full sync (first time, or resuming an interrupted sync)
        let report = sync_smart_full(client, user_id, &service, &state).await?;
        let duration = start.elapsed();
        info!("Smart full sync completed in {}ms", duration.as_millis());
        Ok(SyncReport {
            duration_ms: duration.as_millis() as u64,
            ..report
        })
    } else {
        // Incremental sync
        let report = sync_smart_incremental(client, user_id, &service, &state).await?;
        let duration = start.elapsed();
        info!(
            "Smart incremental sync completed in {}ms",
            duration.as_millis()
        );
        Ok(SyncReport {
            duration_ms: duration.as_millis() as u64,
            ..report
        })
    }
}

/// Full paginated sync with resumability.
/// Downloads everything in pages and commits each page to redb immediately.
///
/// Entity processing order: songs -> albums -> artists.
/// If resuming, we skip entities that were already completed.
async fn sync_smart_full(
    client: &JellyfinClient,
    user_id: &str,
    service: &crate::domain::services::LibraryService,
    state: &SyncState,
) -> Result<SyncReport> {
    let mut songs_updated: u32 = 0;
    let mut albums_updated: u32 = 0;
    let mut artists_updated: u32 = 0;

    // Determine where to resume from
    let resume_entity = state.full_sync_entity_type.as_deref().unwrap_or("songs");
    let resume_page = if state.full_sync_in_progress {
        state.full_sync_last_page_index as usize
    } else {
        // Starting fresh: clear all tables first
        info!("Starting fresh full sync - clearing all tables");
        let db = get()?;
        let write_txn = db.begin_write()?;
        {
            service.clear_all_tables(&write_txn)?;
        }
        write_txn.commit()?;
        0
    };

    // Assign a numeric stage to each entity for ordering
    let entity_stage = |e: &str| -> u8 {
        match e {
            "songs" => 0,
            "albums" => 1,
            "artists" => 2,
            _ => 0,
        }
    };
    let resume_stage = entity_stage(resume_entity);

    info!(
        "Full sync: resuming from entity={}, page={}",
        resume_entity, resume_page
    );

    // Track the server timestamp from the first response
    let mut server_date: Option<String> = None;

    // --- Songs ---
    if resume_stage == 0 {
        let start_index = if resume_entity == "songs" {
            resume_page * SYNC_PAGE_SIZE
        } else {
            0
        };

        // Mark in-progress
        update_full_sync_progress(service, "songs", (start_index / SYNC_PAGE_SIZE) as u32)?;

        songs_updated = sync_entity_paginated(
            client,
            user_id,
            "songs",
            start_index,
            service,
            &mut server_date,
        )
        .await?;
    }

    // --- Albums ---
    if resume_stage <= 1 {
        let start_index = if resume_entity == "albums" {
            resume_page * SYNC_PAGE_SIZE
        } else {
            0
        };

        update_full_sync_progress(service, "albums", (start_index / SYNC_PAGE_SIZE) as u32)?;

        albums_updated = sync_entity_paginated(
            client,
            user_id,
            "albums",
            start_index,
            service,
            &mut server_date,
        )
        .await?;
    }

    // --- Artists ---
    if resume_stage <= 2 {
        let start_index = if resume_entity == "artists" {
            resume_page * SYNC_PAGE_SIZE
        } else {
            0
        };

        update_full_sync_progress(service, "artists", (start_index / SYNC_PAGE_SIZE) as u32)?;

        artists_updated = sync_entity_paginated(
            client,
            user_id,
            "artists",
            start_index,
            service,
            &mut server_date,
        )
        .await?;
    }

    // All done - finalize sync state
    let sync_time = JellyfinClient::parse_server_date(server_date.as_deref());
    let (song_count, artist_count, album_count) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    let final_state = SyncState {
        last_sync_time: sync_time.clone(),
        last_full_sync_time: Some(sync_time),
        last_sync_version: None,
        song_count,
        artist_count,
        album_count,
        full_sync_in_progress: false,
        full_sync_last_page_index: 0,
        full_sync_entity_type: None,
    };
    service
        .update_sync_state(&final_state)
        .map_err(|e| anyhow!("Failed to update sync state: {}", e))?;

    Ok(SyncReport {
        full_sync: true,
        songs_updated,
        artists_updated,
        albums_updated,
        duration_ms: 0, // Filled by caller
    })
}

/// Helper: paginate-fetch one entity type and upsert each page.
async fn sync_entity_paginated(
    client: &JellyfinClient,
    user_id: &str,
    entity_type: &str,
    start_index: usize,
    service: &crate::domain::services::LibraryService,
    server_date: &mut Option<String>,
) -> Result<u32> {
    let mut total_upserted: u32 = 0;
    let mut current_start = start_index;
    let mut page_num = start_index / SYNC_PAGE_SIZE;

    loop {
        let base_query = match entity_type {
            "songs" => format!(
                "/Items?userId={user_id}&IncludeItemTypes=Audio&Recursive=true&Fields=Genres,DateCreated,DateLastModified,MediaSources,ParentId,People,Tags,Path,RunTimeTicks,ImageTags,AlbumId,Artists,Album,ProductionYear,UserData,IndexNumber,PremiereDate,AlbumArtists,MediaStreams"
            ),
            "albums" => format!(
                "/Items?userId={user_id}&IncludeItemTypes=MusicAlbum&Recursive=true&Fields=ImageTags,Overview,ProductionYear,CommunityRating,Artists,ProviderIds,DateCreated,DateLastModified"
            ),
            "artists" => format!(
                "/Items?userId={user_id}&IncludeItemTypes=MusicArtist&Recursive=true&Fields=ImageTags,Overview,ProviderIds,CommunityRating,DateLastModified"
            ),
            other => return Err(anyhow!("Unknown entity type: {}", other)),
        };

        let page = client
            .fetch_items_page(&base_query, current_start, SYNC_PAGE_SIZE)
            .await
            .map_err(|e| anyhow!("Failed to fetch {} page: {}", entity_type, e))?;

        if server_date.is_none() {
            *server_date = page.server_date;
        }

        if page.items.is_empty() {
            break;
        }

        let page_count = page.items.len();

        // Parse and upsert this page immediately
        match entity_type {
            "songs" => {
                let songs: Vec<Song> = page
                    .items
                    .iter()
                    .filter_map(|item| client.parse_single_music_item(item).ok())
                    .collect();
                let count = service
                    .upsert_songs(&songs)
                    .map_err(|e| anyhow!("Failed to upsert songs: {}", e))?;
                total_upserted += count;
            }
            "albums" => {
                let albums: Vec<Album> = page
                    .items
                    .iter()
                    .map(|item| client.parse_single_album(item))
                    .collect();
                let count = service
                    .upsert_albums(&albums)
                    .map_err(|e| anyhow!("Failed to upsert albums: {}", e))?;
                total_upserted += count;
            }
            "artists" => {
                let artists: Vec<Artist> = page
                    .items
                    .iter()
                    .filter_map(|item| client.parse_single_artist(item).ok())
                    .collect();
                let count = service
                    .upsert_artists(&artists)
                    .map_err(|e| anyhow!("Failed to upsert artists: {}", e))?;
                total_upserted += count;
            }
            _ => {}
        }

        page_num += 1;
        current_start += page_count;

        // Update UI progress (polled by C# via get_sync_progress)
        if let Ok(mut p) = SYNC_PROGRESS.lock() {
            *p = SyncProgress::new(
                entity_type,
                current_start as u32,
                page.total_record_count as u32,
            );
        }

        // Update progress for resumability
        update_full_sync_progress(service, entity_type, page_num as u32)?;

        info!(
            "Full sync {}: page {} done ({}/{})",
            entity_type, page_num, current_start, page.total_record_count
        );

        if current_start >= page.total_record_count {
            break;
        }
    }

    Ok(total_upserted)
}

/// Update SyncState to track full sync progress (for resumability).
fn update_full_sync_progress(
    service: &crate::domain::services::LibraryService,
    entity_type: &str,
    page_index: u32,
) -> Result<()> {
    let mut state = service
        .get_sync_state()
        .map_err(|e| anyhow!("Failed to get sync state: {}", e))?;

    state.full_sync_in_progress = true;
    state.full_sync_entity_type = Some(entity_type.to_string());
    state.full_sync_last_page_index = page_index;

    service
        .update_sync_state(&state)
        .map_err(|e| anyhow!("Failed to update sync state: {}", e))?;
    Ok(())
}

/// Incremental sync: fetch only items changed since last sync, then detect deletions.
async fn sync_smart_incremental(
    client: &JellyfinClient,
    user_id: &str,
    service: &crate::domain::services::LibraryService,
    state: &SyncState,
) -> Result<SyncReport> {
    let since_date = &state.last_sync_time;
    info!("Incremental sync since: {}", since_date);

    // Step 1: Fetch updated items (only those changed since last sync)
    let (updated_songs, server_date) = client
        .get_songs_paginated(user_id, Some(since_date), SYNC_PAGE_SIZE)
        .await
        .map_err(|e| anyhow!("Failed to fetch updated songs: {}", e))?;

    let (updated_albums, _) = client
        .get_albums_paginated(user_id, Some(since_date), SYNC_PAGE_SIZE)
        .await
        .map_err(|e| anyhow!("Failed to fetch updated albums: {}", e))?;

    let (updated_artists, _) = client
        .get_artists_paginated(user_id, Some(since_date), SYNC_PAGE_SIZE)
        .await
        .map_err(|e| anyhow!("Failed to fetch updated artists: {}", e))?;

    info!(
        "Incremental fetch: {} songs, {} albums, {} artists changed",
        updated_songs.len(),
        updated_albums.len(),
        updated_artists.len()
    );

    // Step 2: Upsert the changed items
    let songs_upserted = service
        .upsert_songs(&updated_songs)
        .map_err(|e| anyhow!("Failed to upsert songs: {}", e))?;
    let albums_upserted = service
        .upsert_albums(&updated_albums)
        .map_err(|e| anyhow!("Failed to upsert albums: {}", e))?;
    let artists_upserted = service
        .upsert_artists(&updated_artists)
        .map_err(|e| anyhow!("Failed to upsert artists: {}", e))?;

    // Step 3: Detect deletions using lightweight ID-only fetch
    let remote_song_ids = client
        .get_all_item_ids(user_id, "Audio")
        .await
        .map_err(|e| anyhow!("Failed to fetch song IDs: {}", e))?;
    let remote_album_ids = client
        .get_all_item_ids(user_id, "MusicAlbum")
        .await
        .map_err(|e| anyhow!("Failed to fetch album IDs: {}", e))?;
    let remote_artist_ids = client
        .get_all_item_ids(user_id, "MusicArtist")
        .await
        .map_err(|e| anyhow!("Failed to fetch artist IDs: {}", e))?;

    let song_id_set: std::collections::HashSet<String> = remote_song_ids.into_iter().collect();
    let album_id_set: std::collections::HashSet<String> = remote_album_ids.into_iter().collect();
    let artist_id_set: std::collections::HashSet<String> = remote_artist_ids.into_iter().collect();

    let songs_deleted = service
        .remove_deleted_songs(&song_id_set)
        .map_err(|e| anyhow!("Failed to remove deleted songs: {}", e))?;
    let albums_deleted = service
        .remove_deleted_albums(&album_id_set)
        .map_err(|e| anyhow!("Failed to remove deleted albums: {}", e))?;
    let artists_deleted = service
        .remove_deleted_artists(&artist_id_set)
        .map_err(|e| anyhow!("Failed to remove deleted artists: {}", e))?;

    info!(
        "Deletions: {} songs, {} albums, {} artists removed",
        songs_deleted, albums_deleted, artists_deleted
    );

    // Step 4: Update sync state with server timestamp
    let sync_time = JellyfinClient::parse_server_date(server_date.as_deref());
    let (song_count, artist_count, album_count) = service
        .get_library_stats()
        .map_err(|e| anyhow!("Failed to get library stats: {}", e))?;

    let new_state = SyncState {
        last_sync_time: sync_time,
        last_full_sync_time: state.last_full_sync_time.clone(),
        last_sync_version: None,
        song_count,
        artist_count,
        album_count,
        full_sync_in_progress: false,
        full_sync_last_page_index: 0,
        full_sync_entity_type: None,
    };
    service
        .update_sync_state(&new_state)
        .map_err(|e| anyhow!("Failed to update sync state: {}", e))?;

    Ok(SyncReport {
        full_sync: false,
        songs_updated: songs_upserted + songs_deleted,
        artists_updated: artists_upserted + artists_deleted,
        albums_updated: albums_upserted + albums_deleted,
        duration_ms: 0, // Filled by caller
    })
}

/// Reset sync state to force a full sync on next call.
pub fn reset_sync_state() -> Result<()> {
    let db = get()?;
    let service = crate::domain::services::LibraryService::new(db);
    let initial_state = SyncState {
        last_sync_time: "1970-01-01T00:00:00Z".to_string(),
        last_full_sync_time: None,
        last_sync_version: None,
        song_count: 0,
        artist_count: 0,
        album_count: 0,
        full_sync_in_progress: false,
        full_sync_last_page_index: 0,
        full_sync_entity_type: None,
    };
    service
        .update_sync_state(&initial_state)
        .map_err(|e| anyhow!("Failed to reset sync state: {}", e))?;
    Ok(())
}

// ============================================================================
// Songs submodule
// ============================================================================

pub mod songs {
    use super::*;
    use redb::ReadableDatabase;

    pub fn get_all() -> Result<Vec<Song>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
        get_all_items(&table)
    }

    pub fn get_by_id(song_id: &str) -> Result<Option<Song>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(SONGS_TABLE)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;

        if let Some(bytes) = table.get(song_id)? {
            let song: Song = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode song: {}", e))?;
            Ok(Some(song))
        } else {
            Ok(None)
        }
    }

    pub fn update_favorite_status(song_id: &str, is_favorite: bool) -> Result<()> {
        let db = crate::db::get()?;
        let repo = crate::db::repositories::SongRepository::new(db);
        repo.update_favorite_status(song_id, is_favorite)
    }

    pub fn clear() -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(SONGS_TABLE)
                .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
            clear_table(&mut table)?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Artists submodule
// ============================================================================

pub mod artists {
    use super::*;
    use redb::ReadableDatabase;

    pub fn get_all() -> Result<Vec<Artist>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
        get_all_items(&table)
    }

    pub fn clear() -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ARTISTS_TABLE)
                .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
            clear_table(&mut table)?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        Ok(())
    }

    pub fn get_by_id(artist_id: &str) -> Result<Option<Artist>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ARTISTS_TABLE)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;

        if let Some(bytes) = table.get(artist_id)? {
            let artist: Artist = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode artist: {}", e))?;
            Ok(Some(artist))
        } else {
            Ok(None)
        }
    }

    pub fn cache(artist: &Artist) -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ARTISTS_TABLE)
                .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
            let encoded = postcard::to_stdvec(artist)
                .map_err(|e| anyhow!("Failed to encode artist: {}", e))?;
            table
                .insert(artist.id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert artist: {}", e))?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        debug!("Cached artist: {}", artist.name);
        Ok(())
    }
}

// ============================================================================
// Albums submodule
// ============================================================================

pub mod albums {
    use super::*;
    use redb::ReadableDatabase;

    pub fn get_all() -> Result<Vec<Album>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ALBUMS_TABLE)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
        let albums = get_all_items(&table)?;
        info!("Retrieved {} albums from database", albums.len());
        Ok(albums)
    }

    pub fn clear() -> Result<()> {
        let db = crate::db::get()?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ALBUMS_TABLE)
                .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
            clear_table(&mut table)?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        Ok(())
    }

    pub fn get_by_id(album_id: &str) -> Result<Option<Album>> {
        let db = crate::db::get()?;
        let read_txn = db
            .begin_read()
            .map_err(|e| anyhow!("Failed to begin read transaction: {}", e))?;
        let table = read_txn
            .open_table(ALBUMS_TABLE)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;

        if let Some(bytes) = table.get(album_id)? {
            let album: Album = postcard::from_bytes(bytes.value())
                .map_err(|e| anyhow!("Failed to decode album: {}", e))?;
            Ok(Some(album))
        } else {
            Ok(None)
        }
    }

    pub fn cache(album: &Album) -> Result<()> {
        let db = crate::db::get()?;
        let album_id = album.id.as_ref().ok_or(anyhow!("Album has no ID"))?;
        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
        {
            let mut table = write_txn
                .open_table(ALBUMS_TABLE)
                .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
            let encoded =
                postcard::to_stdvec(album).map_err(|e| anyhow!("Failed to encode album: {}", e))?;
            table
                .insert(album_id.as_str(), encoded.as_slice())
                .map_err(|e| anyhow!("Failed to insert album: {}", e))?;
        }
        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
        debug!("Cached album: {}", album.name);
        Ok(())
    }
}

#[cfg(test)]
pub fn reset_for_tests() -> Result<()> {
    let db = get()?;
    let write_txn = db
        .begin_write()
        .map_err(|e| anyhow!("Failed to begin write transaction: {}", e))?;
    {
        let mut songs = write_txn
            .open_table(schema::SONGS)
            .map_err(|e| anyhow!("Failed to open songs table: {}", e))?;
        clear_table(&mut songs)?;

        let mut artists = write_txn
            .open_table(schema::ARTISTS)
            .map_err(|e| anyhow!("Failed to open artists table: {}", e))?;
        clear_table(&mut artists)?;

        let mut albums = write_txn
            .open_table(schema::ALBUMS)
            .map_err(|e| anyhow!("Failed to open albums table: {}", e))?;
        clear_table(&mut albums)?;

        let mut playlists = write_txn
            .open_table(schema::PLAYLISTS)
            .map_err(|e| anyhow!("Failed to open playlists table: {}", e))?;
        clear_table(&mut playlists)?;

        let mut favorites = write_txn
            .open_table(schema::FAVORITES)
            .map_err(|e| anyhow!("Failed to open favorites table: {}", e))?;
        clear_table(&mut favorites)?;

        let mut sync_state = write_txn
            .open_table(schema::SYNC_STATE)
            .map_err(|e| anyhow!("Failed to open sync state table: {}", e))?;
        clear_table(&mut sync_state)?;

        let mut credentials = write_txn
            .open_table(schema::CREDENTIALS)
            .map_err(|e| anyhow!("Failed to open credentials table: {}", e))?;
        clear_table(&mut credentials)?;

        let mut songs_by_album = write_txn
            .open_table(schema::SONGS_BY_ALBUM)
            .map_err(|e| anyhow!("Failed to open songs_by_album table: {}", e))?;
        clear_composite_table_for_tests(&mut songs_by_album)?;

        let mut songs_by_artist = write_txn
            .open_table(schema::SONGS_BY_ARTIST)
            .map_err(|e| anyhow!("Failed to open songs_by_artist table: {}", e))?;
        clear_composite_table_for_tests(&mut songs_by_artist)?;

        let mut albums_by_artist = write_txn
            .open_table(schema::ALBUMS_BY_ARTIST)
            .map_err(|e| anyhow!("Failed to open albums_by_artist table: {}", e))?;
        clear_composite_table_for_tests(&mut albums_by_artist)?;
    }
    write_txn
        .commit()
        .map_err(|e| anyhow!("Failed to commit write transaction: {}", e))?;
    Ok(())
}

#[cfg(test)]
fn clear_composite_table_for_tests(table: &mut redb::Table<(&str, &str), ()>) -> Result<()> {
    let mut keys = Vec::new();
    for result in table
        .iter()
        .map_err(|e| anyhow!("Failed to iterate over composite table: {}", e))?
    {
        let (key, _) = result.map_err(|e| anyhow!("Failed to read composite table item: {}", e))?;
        let (k1, k2) = key.value();
        keys.push((k1.to_string(), k2.to_string()));
    }
    for (k1, k2) in keys {
        table
            .remove((k1.as_str(), k2.as_str()))
            .map_err(|e| anyhow!("Failed to remove composite table item: {}", e))?;
    }
    Ok(())
}
