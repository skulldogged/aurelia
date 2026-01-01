use crate::models::{Album, Artist, Credentials, Song};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct AppState {
    pub songs: Arc<Mutex<Vec<Song>>>,
    pub artists: Arc<Mutex<Vec<Artist>>>,
    pub albums: Arc<Mutex<Vec<Album>>>,
    /// Cached credentials to avoid disk I/O on every request
    pub credentials: Arc<Mutex<Option<Credentials>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get cached credentials, or None if not logged in
    pub fn get_credentials(&self) -> Option<Credentials> {
        self.credentials.lock().unwrap().clone()
    }

    /// Update cached credentials
    pub fn set_credentials(&self, creds: Option<Credentials>) {
        *self.credentials.lock().unwrap() = creds;
    }
}
