//! Shared lyrics parsing for Aurelia's mobile clients.
//! Extracted from aurelia-core for standalone use

pub mod error;
pub mod models;
pub mod parser;
pub mod ttml;

pub use models::{ParsedLyrics, ParsedLyricsLine, ParsedLyricsWord};
pub use parser::{parse_auto, parse_lrc, parse_plain_text};
pub use ttml::parse_ttml;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
