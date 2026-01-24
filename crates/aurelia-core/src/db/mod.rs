pub mod repositories;
pub mod schema;

pub use repositories::*;
pub use schema::*;

use anyhow::{Result, anyhow};
use once_cell::sync::OnceCell;
use redb::Database;
use std::path::PathBuf;
use tracing::{debug, info};

pub static DB: OnceCell<Database> = OnceCell::new();

pub fn init(app_data_dir: &PathBuf) -> Result<()> {
    info!("Database path: {:?}", app_data_dir);

    let db_path = app_data_dir.join("aurelia.redb");
    debug!("Full database path: {:?}", db_path);

    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| anyhow!("Failed to create app data directory: {}", e))?;

    let db = Database::create(&db_path).map_err(|e| anyhow!("Failed to create database: {}", e))?;

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

    DB.set(db)
        .map_err(|_| anyhow!("Database already initialized"))?;

    info!("Database initialized successfully");
    Ok(())
}

pub fn get() -> Result<&'static Database> {
    DB.get().ok_or_else(|| anyhow!("Database not initialized"))
}
