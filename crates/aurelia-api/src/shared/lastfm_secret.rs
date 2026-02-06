use crate::{ApiResult, AppError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct LastFmSecretStore {
    api_secret: String,
}

#[must_use]
pub fn path(app_dir: &Path) -> PathBuf {
    app_dir.join("lastfm_secret.json")
}

#[must_use]
pub fn load(app_dir: &Path) -> Option<String> {
    let secret_path = path(app_dir);
    let contents = std::fs::read_to_string(secret_path).ok()?;
    let store: LastFmSecretStore = serde_json::from_str(&contents).ok()?;
    Some(store.api_secret)
}

pub fn save(app_dir: &Path, api_secret: &str) -> ApiResult<()> {
    let secret_path = path(app_dir);
    let payload = serde_json::to_string(&LastFmSecretStore {
        api_secret: api_secret.to_string(),
    })
    .map_err(|error| AppError::Serialization(error.to_string()))?;
    std::fs::write(secret_path, payload)
        .map_err(|error| AppError::FileSystem(error.to_string()))?;
    Ok(())
}

pub fn clear(app_dir: &Path) -> ApiResult<()> {
    let secret_path = path(app_dir);
    if secret_path.exists() {
        std::fs::remove_file(secret_path)
            .map_err(|error| AppError::FileSystem(error.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{clear, load, save};
    use tempfile::tempdir;

    #[test]
    fn lastfm_secret_roundtrip() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path();

        assert!(load(path).is_none());

        save(path, "secret").expect("save");
        assert_eq!(load(path).as_deref(), Some("secret"));

        clear(path).expect("clear");
        assert!(load(path).is_none());
    }
}
