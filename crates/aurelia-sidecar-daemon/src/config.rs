//! Configuration management

use crate::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub jellyfin_url: Option<String>,
    pub jellyfin_api_key: Option<String>,
    pub music_paths: Vec<PathBuf>,
    pub bind: String,
    pub port: u16,
    pub cache_ttl_seconds: u64,
}

impl Config {
    /// Load config from TOML file and merge with CLI args/env vars
    pub async fn from_file_and_args(path: PathBuf, args: &Args) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(&path).await?;
        let mut config: Config = toml::from_str(&content)?;

        // Override with env vars/CLI args if present
        if let Some(url) = &args.jellyfin_url {
            config.jellyfin_url = Some(url.clone());
        }
        if let Some(key) = &args.jellyfin_api_key {
            config.jellyfin_api_key = Some(key.clone());
        }
        if let Some(paths) = &args.music_paths {
            config.music_paths = paths.split(',').map(PathBuf::from).collect();
        }

        Ok(config)
    }

    /// Create config from CLI args
    pub async fn from_args(args: &Args) -> anyhow::Result<Self> {
        let music_paths = args
            .music_paths
            .as_ref()
            .map(|paths| paths.split(',').map(PathBuf::from).collect())
            .unwrap_or_default();

        Ok(Config {
            jellyfin_url: args.jellyfin_url.clone(),
            jellyfin_api_key: args.jellyfin_api_key.clone(),
            music_paths,
            bind: args.bind.clone(),
            port: args.port,
            cache_ttl_seconds: 3600, // 1 hour default
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jellyfin_url: None,
            jellyfin_api_key: None,
            music_paths: vec![],
            bind: "0.0.0.0".to_string(),
            port: 8080,
            cache_ttl_seconds: 3600,
        }
    }
}
