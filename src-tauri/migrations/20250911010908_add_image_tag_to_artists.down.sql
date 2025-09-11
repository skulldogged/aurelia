-- Remove image_tag column from artists table
-- Note: SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
CREATE TABLE artists_backup (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

INSERT INTO artists_backup (id, name)
SELECT id, name FROM artists;

DROP TABLE artists;

CREATE TABLE artists (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

INSERT INTO artists (id, name)
SELECT id, name FROM artists_backup;

DROP TABLE artists_backup;
