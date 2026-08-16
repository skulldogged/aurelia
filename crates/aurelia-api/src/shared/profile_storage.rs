use crate::{AppError, Credentials};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

fn normalize_component(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut previous_was_dash = false;

    for ch in value.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_was_dash = false;
            continue;
        }

        if !previous_was_dash {
            normalized.push('-');
            previous_was_dash = true;
        }
    }

    normalized.trim_matches('-').to_string()
}

pub fn profile_storage_key(credentials: &Credentials) -> String {
    let provider = match credentials.provider {
        crate::BackendProvider::Jellyfin => "jellyfin",
    };

    let normalized_username = normalize_component(&credentials.username);
    let normalized_server = normalize_component(&credentials.server_url);
    let slug = format!("{provider}-{normalized_username}-{normalized_server}");

    let stable_raw = format!(
        "{}|{}|{}",
        provider,
        credentials.username.trim().to_lowercase(),
        credentials.server_url.trim().to_lowercase()
    );
    let mut hasher = DefaultHasher::new();
    stable_raw.hash(&mut hasher);
    let checksum = hasher.finish();

    format!("{slug}-{checksum:016x}")
}

pub fn profile_data_dir(base_dir: &Path, credentials: &Credentials) -> Result<PathBuf, AppError> {
    let profile_dir = base_dir
        .join("profiles")
        .join(profile_storage_key(credentials));
    std::fs::create_dir_all(&profile_dir).map_err(|error| {
        AppError::FileSystem(format!(
            "Failed to create profile directory {}: {}",
            profile_dir.display(),
            error
        ))
    })?;
    Ok(profile_dir)
}

pub fn resolve_active_data_dir(
    base_dir: &Path,
    credentials: Option<&Credentials>,
) -> Result<PathBuf, AppError> {
    match credentials {
        Some(credentials) => profile_data_dir(base_dir, credentials),
        None => Ok(base_dir.to_path_buf()),
    }
}
