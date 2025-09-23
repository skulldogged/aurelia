//! MusicBrainz service for generating share URLs
//!
//! This service handles generating share URLs for songs, albums, and artists
//! using MusicBrainz IDs from Jellyfin metadata and the MusicBrainz API.

use crate::models::{Album, Artist, Song};
use serde::Deserialize;
use std::collections::HashMap;

const MUSICBRAINZ_API_BASE: &str = "https://musicbrainz.org/ws/2";

/// MusicBrainz relationship data
#[derive(Debug, Deserialize)]
struct MusicBrainzRelationship {
    #[serde(rename = "type")]
    rel_type: String,
    url: MusicBrainzUrl,
}

/// MusicBrainz URL resource
#[derive(Debug, Deserialize)]
struct MusicBrainzUrl {
    resource: String,
}

/// MusicBrainz API response data
#[derive(Debug, Deserialize)]
struct MusicBrainzResponse {
    relations: Option<Vec<MusicBrainzRelationship>>,
}

/// Service for generating MusicBrainz-based share URLs
pub struct MusicBrainzService;

impl MusicBrainzService {
    /// Generate share URLs for a song
    pub async fn get_song_share_urls(song: &Song) -> Result<HashMap<String, String>, String> {
        let mut urls = HashMap::new();

        // For songs, create search URLs since songs don't have direct MusicBrainz IDs in Jellyfin
        let artist_name = song
            .artists
            .as_ref()
            .and_then(|a| a.first())
            .map(|s| s.as_str())
            .unwrap_or("Unknown Artist");
        let query = format!("{} {}", artist_name, song.name);

        urls.insert(
            "MusicBrainz Search".to_string(),
            format!(
                "https://musicbrainz.org/search?query={}&type=recording",
                urlencoding::encode(&query)
            ),
        );

        Ok(urls)
    }

    /// Generate share URLs for an album
    pub async fn get_album_share_urls(album: &Album) -> Result<HashMap<String, String>, String> {
        let mut urls = HashMap::new();

        // Check if we have a MusicBrainz album ID from Jellyfin
        if let Some(provider_ids) = &album.provider_ids {
            // Try different possible key variations - Jellyfin typically uses "MusicBrainzAlbum"
            let mbid = provider_ids
                .get("MusicBrainzAlbum")
                .or_else(|| provider_ids.get("MusicBrainz"))
                .or_else(|| provider_ids.get("musicbrainz"));
            if let Some(mbid) = mbid {
                // Add the MusicBrainz URL itself
                urls.insert(
                    "MusicBrainz".to_string(),
                    format!("https://musicbrainz.org/release/{}", mbid),
                );

                // Try to fetch the album data from MusicBrainz API to get external links
                match Self::fetch_album_relationships(mbid).await {
                    Ok(platform_urls) => {
                        urls.extend(platform_urls);
                    }
                    Err(e) => {
                        // If API call fails, log the error but continue with just the MusicBrainz URL
                        eprintln!(
                            "Failed to fetch MusicBrainz relationships for album {}: {}",
                            mbid, e
                        );
                    }
                }

                return Ok(urls);
            }
        }

        // Fallback to search-based URLs when no MBID is available
        let album_name = &album.name;
        let artist_name = &album.artist;

        urls.insert(
            "MusicBrainz Search".to_string(),
            format!(
                "https://musicbrainz.org/search?query={}&type=release",
                urlencoding::encode(&format!("{artist_name} {album_name}"))
            ),
        );

        Ok(urls)
    }

    /// Generate share URLs for an artist
    pub async fn get_artist_share_urls(artist: &Artist) -> Result<HashMap<String, String>, String> {
        let mut urls = HashMap::new();

        // Try to get direct MusicBrainz URL and external platform links if we have the MBID
        if let Some(provider_ids) = &artist.provider_ids {
            // Try different possible key variations - Jellyfin typically uses "MusicBrainzArtist"
            let mbid = provider_ids
                .get("MusicBrainzArtist")
                .or_else(|| provider_ids.get("MusicBrainz"))
                .or_else(|| provider_ids.get("musicbrainz"))
                .or_else(|| provider_ids.get("Musicbrainz"));
            if let Some(mbid) = mbid {
                // Add the MusicBrainz URL itself
                urls.insert(
                    "MusicBrainz".to_string(),
                    format!("https://musicbrainz.org/artist/{mbid}"),
                );

                // Try to fetch the artist data from MusicBrainz API to get external links
                match Self::fetch_artist_relationships(mbid).await {
                    Ok(platform_urls) => {
                        urls.extend(platform_urls);
                        return Ok(urls);
                    }
                    Err(e) => {
                        // If API call fails, log the error but continue with search URLs
                        eprintln!(
                            "Failed to fetch MusicBrainz relationships for artist {}: {}",
                            mbid, e
                        );
                    }
                }

                // Fallback: still try to provide some platform links using the MBID
                urls.insert(
                    "Spotify".to_string(),
                    format!("https://open.spotify.com/artist/{}", mbid),
                );

                return Ok(urls);
            }
        }

        // Fallback to search-based URLs when no MBID is available
        let artist_name = &artist.name;

        urls.insert(
            "MusicBrainz Search".to_string(),
            format!(
                "https://musicbrainz.org/search?query={}&type=artist",
                urlencoding::encode(artist_name)
            ),
        );

        Ok(urls)
    }

    /// Fetch artist relationships from MusicBrainz API
    async fn fetch_artist_relationships(mbid: &str) -> Result<HashMap<String, String>, String> {
        let url = format!(
            "{}/artist/{}?inc=url-rels&fmt=json",
            MUSICBRAINZ_API_BASE, mbid
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header(
                "User-Agent",
                "JellyfinMusicPlayer/1.0.0 (https://github.com/pupbrained/jellyfin-music-player)",
            )
            .send()
            .await
            .map_err(|e| format!("Failed to fetch from MusicBrainz API: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "MusicBrainz API returned status: {} - URL: {}",
                response.status(),
                url
            ));
        }

        let artist_data: MusicBrainzResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))?;

        let mut urls = HashMap::new();

        if let Some(relations) = &artist_data.relations {
            for relation in relations {
                // Add raw relationship data - no processing, just relationship type -> URL
                urls.insert(relation.rel_type.clone(), relation.url.resource.clone());
            }
        }

        Ok(urls)
    }

    /// Fetch album relationships from MusicBrainz API
    async fn fetch_album_relationships(mbid: &str) -> Result<HashMap<String, String>, String> {
        let url = format!(
            "{}/release/{}?inc=url-rels&fmt=json",
            MUSICBRAINZ_API_BASE, mbid
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header(
                "User-Agent",
                "JellyfinMusicPlayer/1.0.0 (https://github.com/pupbrained/jellyfin-music-player)",
            )
            .send()
            .await
            .map_err(|e| format!("Failed to fetch from MusicBrainz API: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "MusicBrainz API returned status: {} - URL: {}",
                response.status(),
                url
            ));
        }

        let album_data: MusicBrainzResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))?;

        let mut urls = HashMap::new();

        if let Some(relations) = &album_data.relations {
            for relation in relations {
                urls.insert(relation.rel_type.clone(), relation.url.resource.clone());
            }
        }

        Ok(urls)
    }
}
