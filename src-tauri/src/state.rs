use crate::models::{Album, Artist, Song};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct AppState {
    pub songs: Arc<Mutex<Vec<Song>>>,
    pub artists: Arc<Mutex<Vec<Artist>>>,
    pub albums: Arc<Mutex<Vec<Album>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
