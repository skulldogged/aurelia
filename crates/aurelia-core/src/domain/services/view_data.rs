use crate::models::{Album, Artist, HomeViewData, LibraryData, MobileHomeData, Song};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

/// Limits used when deriving home sections from a song list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type, uniffi::Record)]
#[serde(rename_all = "camelCase")]
#[specta(rename_all = "camelCase")]
pub struct HomeViewLimits {
    pub featured_albums: u32,
    pub random_albums: u32,
    pub recently_added_albums: u32,
}

impl Default for HomeViewLimits {
    fn default() -> Self {
        Self {
            featured_albums: 20,
            random_albums: 20,
            recently_added_albums: 20,
        }
    }
}

/// Limits used when deriving mobile home sections from a song list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type, uniffi::Record)]
#[serde(rename_all = "camelCase")]
#[specta(rename_all = "camelCase")]
pub struct MobileHomeViewLimits {
    pub most_played: u32,
    pub recently_played: u32,
    pub album_section: u32,
    pub featured_albums: u32,
}

impl Default for MobileHomeViewLimits {
    fn default() -> Self {
        Self {
            most_played: 10,
            recently_played: 10,
            album_section: 12,
            featured_albums: 5,
        }
    }
}

/// Build `LibraryData` from a cached song list.
#[must_use]
pub fn derive_library_data(songs: &[Song]) -> LibraryData {
    let mut album_map: HashMap<String, Vec<Song>> = HashMap::new();
    let mut artist_map: HashMap<String, Artist> = HashMap::new();

    for song in songs {
        if let Some(album_id) = &song.album_id {
            album_map
                .entry(album_id.clone())
                .or_default()
                .push(song.clone());
        }

        if let Some(artist_ids) = &song.artist_ids {
            for (i, artist_id) in artist_ids.iter().enumerate() {
                if !artist_map.contains_key(artist_id) {
                    let name = song
                        .artists
                        .as_ref()
                        .and_then(|artists| artists.get(i))
                        .cloned()
                        .unwrap_or_else(|| "Unknown Artist".to_string());

                    artist_map.insert(
                        artist_id.clone(),
                        Artist {
                            name,
                            id: artist_id.clone(),
                            image_tags: None,
                            image_url: None,
                            overview: None,
                            provider_ids: None,
                            community_rating: None,
                            song_count: None,
                            date_modified: None,
                            songs: None,
                        },
                    );
                }
            }
        }
    }

    let albums: Vec<Album> = album_map
        .iter()
        .filter_map(|(album_id, album_songs)| {
            let first_song = album_songs
                .iter()
                .max_by_key(|song| song.date_created.as_deref().unwrap_or(""))?;

            Some(Album {
                id: Some(album_id.clone()),
                name: first_song
                    .album
                    .clone()
                    .unwrap_or_else(|| "Unknown Album".to_string()),
                artist: first_song.artists.as_ref()?.first()?.clone(),
                artist_id: first_song.artist_ids.as_ref()?.first().cloned(),
                album_art_url: first_song.album_art_url.clone(),
                song_count: album_songs.len() as i64,
                songs: None,
                image_tags: None,
                provider_ids: None,
                date_created: first_song.date_created.clone(),
                date_modified: None,
            })
        })
        .collect();

    let artists: Vec<Artist> = artist_map.into_values().collect();

    LibraryData {
        albums,
        artists,
        songs: songs.to_vec(),
    }
}

/// Build `HomeViewData` from cached songs and a recently-played list.
#[must_use]
pub fn derive_home_view_data(
    all_songs: &[Song],
    recently_played: Vec<Song>,
    limits: HomeViewLimits,
    rng: &mut impl rand::Rng,
) -> HomeViewData {
    let mut albums: Vec<Album> = Vec::new();
    let mut album_song_counts: HashMap<String, usize> = HashMap::new();

    for song in all_songs {
        if let Some(album_id) = &song.album_id {
            let count = album_song_counts.entry(album_id.clone()).or_insert(0);
            *count += 1;
        }
    }

    let mut seen_albums: HashMap<String, bool> = HashMap::new();
    for song in all_songs {
        if let Some(album_id) = &song.album_id
            && !seen_albums.contains_key(album_id)
        {
            seen_albums.insert(album_id.clone(), true);
            let song_count = album_song_counts.get(album_id).copied().unwrap_or(0) as i64;
            albums.push(Album {
                id: Some(album_id.clone()),
                name: song.album.clone().unwrap_or_default(),
                artist: song
                    .artists
                    .as_ref()
                    .and_then(|artists| artists.first())
                    .cloned()
                    .unwrap_or_default(),
                artist_id: song
                    .artist_ids
                    .as_ref()
                    .and_then(|ids| ids.first())
                    .cloned(),
                album_art_url: song.album_art_url.clone(),
                song_count,
                songs: None,
                image_tags: None,
                provider_ids: None,
                date_created: song.date_created.clone(),
                date_modified: None,
            });
        }
    }

    let mut recently_added = albums.clone();
    recently_added.sort_by(|a, b| b.date_created.cmp(&a.date_created));
    recently_added.truncate(limits.recently_added_albums as usize);

    albums.shuffle(rng);
    let random_albums: Vec<Album> = albums
        .iter()
        .take(limits.random_albums as usize)
        .cloned()
        .collect();

    let featured_albums: Vec<Album> = random_albums
        .iter()
        .take(limits.featured_albums as usize)
        .cloned()
        .collect();

    HomeViewData {
        recently_played,
        recently_added,
        random_albums,
        featured_albums,
    }
}

/// Build `MobileHomeData` from cached songs for mobile clients.
#[must_use]
pub fn derive_mobile_home_data(
    all_songs: &[Song],
    limits: MobileHomeViewLimits,
    rng: &mut impl rand::Rng,
) -> MobileHomeData {
    let mut most_played: Vec<Song> = all_songs
        .iter()
        .filter(|song| song.play_count.unwrap_or(0) > 0)
        .cloned()
        .collect();
    most_played.sort_by(|a, b| b.play_count.unwrap_or(0).cmp(&a.play_count.unwrap_or(0)));
    most_played.truncate(limits.most_played as usize);

    let mut recently_played: Vec<Song> = all_songs
        .iter()
        .filter(|song| {
            song.date_played
                .as_ref()
                .is_some_and(|date_played| !date_played.is_empty())
        })
        .cloned()
        .collect();
    recently_played.sort_by(|a, b| b.date_played.cmp(&a.date_played));
    recently_played.truncate(limits.recently_played as usize);

    let derived_home = derive_home_view_data(
        all_songs,
        recently_played.clone(),
        HomeViewLimits {
            featured_albums: limits.featured_albums,
            random_albums: limits.album_section,
            recently_added_albums: limits.album_section,
        },
        rng,
    );

    MobileHomeData {
        most_played,
        recently_played: derived_home.recently_played,
        recently_added: derived_home.recently_added,
        random_albums: derived_home.random_albums,
        featured_albums: derived_home.featured_albums,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HomeViewLimits, MobileHomeViewLimits, derive_home_view_data, derive_library_data,
        derive_mobile_home_data,
    };
    use crate::models::Song;
    use rand::{SeedableRng, rngs::StdRng};

    fn song(
        id: &str,
        album_id: Option<&str>,
        album: Option<&str>,
        artist_id: Option<&str>,
        artist: Option<&str>,
        date_created: Option<&str>,
    ) -> Song {
        Song {
            id: id.to_string(),
            name: format!("Song {id}"),
            item_type: "Audio".to_string(),
            album: album.map(ToString::to_string),
            album_id: album_id.map(ToString::to_string),
            artists: artist.map(|value| vec![value.to_string()]),
            artist_ids: artist_id.map(|value| vec![value.to_string()]),
            path: None,
            duration: Some(180.0),
            album_art_url: Some(format!("https://img/{id}.jpg")),
            year: None,
            play_count: Some(1),
            is_favorite: Some(false),
            disc_number: None,
            track_number: None,
            container: None,
            bit_rate: None,
            sample_rate: None,
            codec: None,
            genres: None,
            premiere_date: None,
            date_played: None,
            date_created: date_created.map(ToString::to_string),
            date_modified: None,
            album_artists: None,
            lyrics: None,
            image_tags: None,
        }
    }

    #[test]
    fn derive_library_data_builds_artist_and_album_collections() {
        let songs = vec![
            song(
                "1",
                Some("alb-1"),
                Some("Album 1"),
                Some("art-1"),
                Some("Artist 1"),
                Some("2025-01-01"),
            ),
            song(
                "2",
                Some("alb-1"),
                Some("Album 1"),
                Some("art-1"),
                Some("Artist 1"),
                Some("2025-01-02"),
            ),
            song(
                "3",
                Some("alb-2"),
                Some("Album 2"),
                Some("art-2"),
                Some("Artist 2"),
                Some("2025-01-03"),
            ),
        ];

        let derived = derive_library_data(&songs);
        assert_eq!(derived.songs.len(), 3);
        assert_eq!(derived.artists.len(), 2);
        assert_eq!(derived.albums.len(), 2);
    }

    #[test]
    fn derive_home_view_data_respects_limits_and_sorting() {
        let songs = vec![
            song(
                "1",
                Some("alb-1"),
                Some("Album 1"),
                Some("art-1"),
                Some("Artist 1"),
                Some("2025-01-01"),
            ),
            song(
                "2",
                Some("alb-2"),
                Some("Album 2"),
                Some("art-2"),
                Some("Artist 2"),
                Some("2025-01-03"),
            ),
            song(
                "3",
                Some("alb-3"),
                Some("Album 3"),
                Some("art-3"),
                Some("Artist 3"),
                Some("2025-01-02"),
            ),
        ];
        let recently_played = vec![songs[0].clone()];
        let limits = HomeViewLimits {
            featured_albums: 1,
            random_albums: 2,
            recently_added_albums: 2,
        };
        let mut rng = StdRng::seed_from_u64(42);

        let derived = derive_home_view_data(&songs, recently_played.clone(), limits, &mut rng);
        assert_eq!(derived.recently_played, recently_played);
        assert_eq!(derived.recently_added.len(), 2);
        assert_eq!(derived.recently_added[0].id.as_deref(), Some("alb-2"));
        assert_eq!(derived.random_albums.len(), 2);
        assert_eq!(derived.featured_albums.len(), 1);
    }

    #[test]
    fn derive_mobile_home_data_respects_limits_and_sorting() {
        let mut songs = vec![
            song(
                "1",
                Some("alb-1"),
                Some("Album 1"),
                Some("art-1"),
                Some("Artist 1"),
                Some("2025-01-01"),
            ),
            song(
                "2",
                Some("alb-2"),
                Some("Album 2"),
                Some("art-2"),
                Some("Artist 2"),
                Some("2025-01-03"),
            ),
            song(
                "3",
                Some("alb-3"),
                Some("Album 3"),
                Some("art-3"),
                Some("Artist 3"),
                Some("2025-01-02"),
            ),
        ];
        songs[0].play_count = Some(3);
        songs[1].play_count = Some(8);
        songs[2].play_count = Some(1);
        songs[0].date_played = Some("2025-01-02T12:00:00Z".to_string());
        songs[1].date_played = Some("2025-01-03T12:00:00Z".to_string());

        let limits = MobileHomeViewLimits {
            most_played: 2,
            recently_played: 1,
            album_section: 2,
            featured_albums: 1,
        };
        let mut rng = StdRng::seed_from_u64(42);

        let derived = derive_mobile_home_data(&songs, limits, &mut rng);
        assert_eq!(derived.most_played.len(), 2);
        assert_eq!(derived.most_played[0].id, "2");
        assert_eq!(derived.recently_played.len(), 1);
        assert_eq!(derived.recently_played[0].id, "2");
        assert_eq!(derived.recently_added.len(), 2);
        assert_eq!(derived.random_albums.len(), 2);
        assert_eq!(derived.featured_albums.len(), 1);
    }
}
