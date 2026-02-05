use serde::{Deserialize, Serialize};

/// Sync state persisted to track library sync history
#[derive(Serialize, Deserialize, Debug, Clone, uniffi::Record)]
pub struct SyncState {
    pub last_sync_time: String,
    pub last_full_sync_time: Option<String>,
    pub last_sync_version: Option<String>,
    pub song_count: u32,
    pub artist_count: u32,
    pub album_count: u32,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync_time: "1970-01-01T00:00:00Z".to_string(),
            last_full_sync_time: None,
            last_sync_version: None,
            song_count: 0,
            artist_count: 0,
            album_count: 0,
        }
    }
}

/// Progress update during sync (for UI feedback)
#[derive(Debug, Clone, uniffi::Record)]
pub struct SyncProgress {
    /// Current stage of sync (e.g., "Fetching songs", "Saving to database")
    pub stage: String,
    /// Current item being processed
    pub current: u32,
    /// Total items to process
    pub total: u32,
    /// Whether sync is complete
    pub is_complete: bool,
}

impl Default for SyncProgress {
    fn default() -> Self {
        Self {
            stage: "Starting".to_string(),
            current: 0,
            total: 0,
            is_complete: false,
        }
    }
}

impl SyncProgress {
    pub fn new(stage: &str, current: u32, total: u32) -> Self {
        Self {
            stage: stage.to_string(),
            current,
            total,
            is_complete: false,
        }
    }

    pub fn complete() -> Self {
        Self {
            stage: "Complete".to_string(),
            current: 0,
            total: 0,
            is_complete: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncReport {
    pub full_sync: bool,
    pub songs_updated: u32,
    pub artists_updated: u32,
    pub albums_updated: u32,
    pub duration_ms: u64,
}

/// Represents the changes detected between local and remote library state
#[derive(Debug, Clone, Default)]
pub struct SyncDelta {
    /// IDs of songs to add (new on server)
    pub songs_to_add: Vec<String>,
    /// IDs of songs to remove (deleted on server)
    pub songs_to_remove: Vec<String>,
    /// IDs of songs to update (modified on server)
    pub songs_to_update: Vec<String>,
    /// IDs of artists to add
    pub artists_to_add: Vec<String>,
    /// IDs of artists to remove
    pub artists_to_remove: Vec<String>,
    /// IDs of artists to update
    pub artists_to_update: Vec<String>,
    /// IDs of albums to add
    pub albums_to_add: Vec<String>,
    /// IDs of albums to remove
    pub albums_to_remove: Vec<String>,
    /// IDs of albums to update
    pub albums_to_update: Vec<String>,
}

impl SyncDelta {
    /// Returns true if there are no changes to sync
    pub fn is_empty(&self) -> bool {
        self.songs_to_add.is_empty()
            && self.songs_to_remove.is_empty()
            && self.songs_to_update.is_empty()
            && self.artists_to_add.is_empty()
            && self.artists_to_remove.is_empty()
            && self.artists_to_update.is_empty()
            && self.albums_to_add.is_empty()
            && self.albums_to_remove.is_empty()
            && self.albums_to_update.is_empty()
    }

    /// Returns total count of changes
    pub fn total_changes(&self) -> usize {
        self.songs_to_add.len()
            + self.songs_to_remove.len()
            + self.songs_to_update.len()
            + self.artists_to_add.len()
            + self.artists_to_remove.len()
            + self.artists_to_update.len()
            + self.albums_to_add.len()
            + self.albums_to_remove.len()
            + self.albums_to_update.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{SyncDelta, SyncProgress};

    #[test]
    fn sync_progress_defaults_and_helpers() {
        let default = SyncProgress::default();
        assert_eq!(default.stage, "Starting");
        assert_eq!(default.current, 0);
        assert_eq!(default.total, 0);
        assert!(!default.is_complete);

        let in_progress = SyncProgress::new("Fetching", 2, 10);
        assert_eq!(in_progress.stage, "Fetching");
        assert_eq!(in_progress.current, 2);
        assert_eq!(in_progress.total, 10);
        assert!(!in_progress.is_complete);

        let done = SyncProgress::complete();
        assert!(done.is_complete);
        assert_eq!(done.stage, "Complete");
    }

    #[test]
    fn sync_delta_counts_changes() {
        let mut delta = SyncDelta::default();
        assert!(delta.is_empty());

        delta.songs_to_add.push("s1".to_string());
        delta.albums_to_remove.push("a1".to_string());
        delta.artists_to_update.push("r1".to_string());

        assert!(!delta.is_empty());
        assert_eq!(delta.total_changes(), 3);
    }
}
