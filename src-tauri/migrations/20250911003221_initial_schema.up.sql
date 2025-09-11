-- Create artists table
CREATE TABLE IF NOT EXISTS artists (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

-- Create albums table
CREATE TABLE IF NOT EXISTS albums (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    album_art_url TEXT,
    artist_id TEXT,
    FOREIGN KEY (artist_id) REFERENCES artists(id)
);

-- Create songs table
CREATE TABLE IF NOT EXISTS songs (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    item_type TEXT,
    album_id TEXT,
    path TEXT,
    duration INTEGER,
    album_art_url TEXT,
    year INTEGER,
    play_count INTEGER,
    is_favorite BOOLEAN,
    track_number INTEGER,
    container TEXT,
    premiere_date TEXT,
    date_played TEXT,
    date_created TEXT,
    lyrics TEXT,
    bit_rate INTEGER,
    sample_rate INTEGER,
    codec TEXT,
    FOREIGN KEY (album_id) REFERENCES albums(id)
);

-- Create song_artists join table
CREATE TABLE IF NOT EXISTS song_artists (
    song_id TEXT NOT NULL,
    artist_id TEXT NOT NULL,
    PRIMARY KEY (song_id, artist_id),
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);

-- Create song_album_artists join table
CREATE TABLE IF NOT EXISTS song_album_artists (
    song_id TEXT NOT NULL,
    artist_id TEXT NOT NULL,
    PRIMARY KEY (song_id, artist_id),
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);
