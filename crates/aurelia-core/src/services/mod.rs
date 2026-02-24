//! Service layer for external API interactions
//!
//! This module contains service clients for interacting with external APIs
//! like Jellyfin, `LrcLib`, and MusicBrainz.

pub mod jellyfin;
pub mod lrclib;
pub mod musicbrainz;
pub mod navidrome;

pub use jellyfin::*;
pub use lrclib::*;
pub use musicbrainz::*;
pub use navidrome::*;
