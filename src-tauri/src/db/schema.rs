use redb::TableDefinition;

pub const SONGS: TableDefinition<&str, &[u8]> = TableDefinition::new("songs");
pub const ARTISTS: TableDefinition<&str, &[u8]> = TableDefinition::new("artists");
pub const ALBUMS: TableDefinition<&str, &[u8]> = TableDefinition::new("albums");
pub const PLAYLISTS: TableDefinition<&str, &[u8]> = TableDefinition::new("playlists");

pub const SONGS_BY_ALBUM: TableDefinition<(&str, &str), ()> =
    TableDefinition::new("songs_by_album");
pub const SONGS_BY_ARTIST: TableDefinition<(&str, &str), ()> =
    TableDefinition::new("songs_by_artist");
pub const ALBUMS_BY_ARTIST: TableDefinition<(&str, &str), ()> =
    TableDefinition::new("albums_by_artist");

pub const FAVORITES: TableDefinition<&str, &[u8]> = TableDefinition::new("favorites");
pub const RECENTLY_PLAYED: TableDefinition<&str, &[u8]> = TableDefinition::new("recently_played");

pub const SYNC_STATE: TableDefinition<&str, &[u8]> = TableDefinition::new("sync_state");
pub const DB_VERSION: TableDefinition<&str, u32> = TableDefinition::new("db_version");
