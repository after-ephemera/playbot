use anyhow::{Context, Result};
use rusqlite::{params, Connection, Row};
use serde::Serialize;

/// Persistent track cache backed by SQLite.
///
/// Stores track metadata and lyrics fetched from Spotify and the lyrics service.
pub struct Database {
    conn: Connection,
}

/// Full track information stored in the cache.
#[derive(Debug, Serialize)]
pub struct TrackInfo {
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub release_date: String,
    pub duration_ms: i64,
    pub popularity: i32,
    /// Comma-separated genre names.
    pub genres: String,
    pub lyrics: Option<String>,
    /// Comma-separated producer names.
    pub producers: String,
    /// Comma-separated songwriter names.
    pub writers: String,
}

/// Stats for a single artist aggregated from play events.
#[derive(Debug, Serialize)]
pub struct ArtistStat {
    pub artist_name: String,
    pub total_ms: i64,
}

/// Stats for a single track aggregated from play events.
#[derive(Debug, Serialize)]
pub struct TrackStat {
    pub track: TrackInfo,
    pub play_count: usize,
    pub total_ms: i64,
}

/// Listening stats aggregated over a time window.
#[derive(Debug, Serialize)]
pub struct ListeningStats {
    pub top_tracks: Vec<TrackStat>,
    pub top_artists: Vec<ArtistStat>,
    /// Date strings → total ms listened that day.
    pub daily_ms: Vec<(String, i64)>,
}

fn row_to_track_info(row: &Row) -> rusqlite::Result<TrackInfo> {
    Ok(TrackInfo {
        track_id: row.get(0)?,
        track_name: row.get(1)?,
        artist_name: row.get(2)?,
        album_name: row.get(3)?,
        release_date: row.get(4)?,
        duration_ms: row.get(5)?,
        popularity: row.get(6)?,
        genres: row.get(7)?,
        lyrics: row.get(8)?,
        producers: row.get(9)?,
        writers: row.get(10)?,
    })
}

impl Database {
    /// Open (or create) the database at the given path.
    ///
    /// Pass `":memory:"` to create a temporary in-memory database.
    pub fn new(path: &str) -> Result<Self> {
        let conn =
            Connection::open(path).with_context(|| format!("Failed to open database: {}", path))?;

        Ok(Self { conn })
    }

    /// Run schema migrations. Safe to call multiple times.
    pub fn init(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tracks (
                track_id TEXT PRIMARY KEY,
                track_name TEXT NOT NULL,
                artist_name TEXT NOT NULL,
                album_name TEXT NOT NULL,
                release_date TEXT,
                duration_ms INTEGER,
                popularity INTEGER,
                genres TEXT,
                lyrics TEXT,
                producers TEXT,
                writers TEXT,
                cached_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
                [],
            )
            .context("Failed to create tracks table")?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
                [],
            )
            .context("Failed to create schema_version table")?;

        let current_version: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )?;

        // Migration 1: transition to Spotify URI track IDs.
        if current_version < 1 {
            self.conn
                .execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
        }

        // Migration 2: add indexes for query performance.
        if current_version < 2 {
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_tracks_cached_at ON tracks(cached_at)",
                [],
            )?;
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist_name COLLATE NOCASE)",
                [],
            )?;
            self.conn
                .execute("INSERT INTO schema_version (version) VALUES (2)", [])?;
        }

        // Migration 3: add play_events table for daemon-collected listening history.
        if current_version < 3 {
            self.conn.execute(
                "CREATE TABLE IF NOT EXISTS play_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    track_id TEXT NOT NULL,
                    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    ended_at DATETIME,
                    duration_listened_ms INTEGER,
                    FOREIGN KEY (track_id) REFERENCES tracks(track_id)
                )",
                [],
            )?;
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_play_events_track_id ON play_events(track_id)",
                [],
            )?;
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_play_events_started_at ON play_events(started_at)",
                [],
            )?;
            self.conn
                .execute("INSERT INTO schema_version (version) VALUES (3)", [])?;
        }

        Ok(())
    }

    /// Look up a track by its Spotify URI (e.g. `spotify:track:xxxxx`).
    ///
    /// Returns `None` if the track is not in the cache.
    pub fn get_track_info(&self, track_id: &str) -> Result<Option<TrackInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id, track_name, artist_name, album_name, release_date,
                    duration_ms, popularity, genres, lyrics, producers, writers
             FROM tracks WHERE track_id = ?1",
        )?;

        match stmt.query_row(params![track_id], row_to_track_info) {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Insert or replace a track in the cache.
    pub fn insert_track_info(&self, info: &TrackInfo) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO tracks
             (track_id, track_name, artist_name, album_name, release_date,
              duration_ms, popularity, genres, lyrics, producers, writers,
              cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)",
                params![
                    info.track_id,
                    info.track_name,
                    info.artist_name,
                    info.album_name,
                    info.release_date,
                    info.duration_ms,
                    info.popularity,
                    info.genres,
                    info.lyrics,
                    info.producers,
                    info.writers,
                ],
            )
            .context("Failed to insert track info")?;

        Ok(())
    }

    /// Return the most recently cached tracks, up to `limit`.
    pub fn get_recent_tracks(&self, limit: usize) -> Result<Vec<TrackInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id, track_name, artist_name, album_name, release_date,
                    duration_ms, popularity, genres, lyrics, producers, writers
             FROM tracks
             ORDER BY cached_at DESC
             LIMIT ?1",
        )?;

        let tracks = stmt
            .query_map(params![limit], row_to_track_info)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tracks)
    }

    /// Search for tracks by name, artist, or album (case-insensitive substring match).
    pub fn search_tracks(&self, query: &str) -> Result<Vec<TrackInfo>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT track_id, track_name, artist_name, album_name, release_date,
                    duration_ms, popularity, genres, lyrics, producers, writers
             FROM tracks
             WHERE track_name LIKE ?1 OR artist_name LIKE ?1 OR album_name LIKE ?1
             ORDER BY cached_at DESC",
        )?;

        let tracks = stmt
            .query_map(params![search_pattern], row_to_track_info)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tracks)
    }

    /// Return all cached tracks sorted by artist and track name.
    pub fn get_all_tracks(&self) -> Result<Vec<TrackInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id, track_name, artist_name, album_name, release_date,
                    duration_ms, popularity, genres, lyrics, producers, writers
             FROM tracks
             ORDER BY artist_name, track_name",
        )?;

        let tracks = stmt
            .query_map([], row_to_track_info)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tracks)
    }

    /// Return the total number of tracks in the cache.
    pub fn count_tracks(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?;
        Ok(count)
    }

    // ── Play event recording (used by daemon) ─────────────────────────────────

    /// Record the start of a play event. Returns the new event's row id.
    pub fn record_play_start(&self, track_id: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO play_events (track_id, started_at) VALUES (?1, CURRENT_TIMESTAMP)",
            params![track_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Fill in the end time and duration for an open play event.
    pub fn record_play_end(&self, event_id: i64, duration_ms: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE play_events
             SET ended_at = CURRENT_TIMESTAMP, duration_listened_ms = ?1
             WHERE id = ?2",
            params![duration_ms, event_id],
        )?;
        Ok(())
    }

    /// Return the number of times a track has been played (complete events only).
    pub fn get_play_count(&self, track_id: &str) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM play_events WHERE track_id = ?1 AND ended_at IS NOT NULL",
            params![track_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // ── Analytics queries (used by `pb stats`) ────────────────────────────────

    /// Return top tracks by play count, looking back `days` calendar days.
    pub fn get_top_tracks(&self, days: usize, limit: usize) -> Result<Vec<TrackStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.track_id, t.track_name, t.artist_name, t.album_name, t.release_date,
                    t.duration_ms, t.popularity, t.genres, t.lyrics, t.producers, t.writers,
                    COUNT(p.id) as play_count,
                    COALESCE(SUM(p.duration_listened_ms), 0) as total_ms
             FROM tracks t
             JOIN play_events p ON p.track_id = t.track_id
             WHERE p.started_at >= datetime('now', printf('-%d days', ?1))
               AND p.ended_at IS NOT NULL
             GROUP BY t.track_id
             ORDER BY play_count DESC
             LIMIT ?2",
        )?;

        let stats = stmt
            .query_map(params![days, limit], |row| {
                let track = row_to_track_info(row)?;
                let play_count: usize = row.get(11)?;
                let total_ms: i64 = row.get(12)?;
                Ok(TrackStat {
                    track,
                    play_count,
                    total_ms,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
    }

    /// Return top artists by total listening time, looking back `days` calendar days.
    pub fn get_top_artists(&self, days: usize, limit: usize) -> Result<Vec<ArtistStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.artist_name, SUM(p.duration_listened_ms) as total_ms
             FROM tracks t
             JOIN play_events p ON p.track_id = t.track_id
             WHERE p.started_at >= datetime('now', printf('-%d days', ?1))
               AND p.ended_at IS NOT NULL
             GROUP BY t.artist_name
             ORDER BY total_ms DESC
             LIMIT ?2",
        )?;

        let stats = stmt
            .query_map(params![days, limit], |row| {
                Ok(ArtistStat {
                    artist_name: row.get(0)?,
                    total_ms: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
    }

    /// Return total listening time per calendar day, looking back `days` days.
    pub fn get_listening_by_day(&self, days: usize) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT date(started_at) as day, SUM(duration_listened_ms) as total_ms
             FROM play_events
             WHERE started_at >= datetime('now', printf('-%d days', ?1))
               AND ended_at IS NOT NULL
             GROUP BY day
             ORDER BY day DESC",
        )?;

        let rows = stmt
            .query_map(params![days], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Return aggregated stats for the last `days` days.
    pub fn get_stats(&self, days: usize) -> Result<ListeningStats> {
        Ok(ListeningStats {
            top_tracks: self.get_top_tracks(days, 10)?,
            top_artists: self.get_top_artists(days, 10)?,
            daily_ms: self.get_listening_by_day(days)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let db = Database::new(":memory:").unwrap();
        db.init().unwrap();
        db
    }

    fn sample_track(id: &str, name: &str, artist: &str) -> TrackInfo {
        TrackInfo {
            track_id: id.to_string(),
            track_name: name.to_string(),
            artist_name: artist.to_string(),
            album_name: "Test Album".to_string(),
            release_date: "2024-01-01".to_string(),
            duration_ms: 240000,
            popularity: 75,
            genres: "rock, indie".to_string(),
            lyrics: Some("Test lyrics".to_string()),
            producers: "Test Producer".to_string(),
            writers: "Test Writer".to_string(),
        }
    }

    #[test]
    fn count_tracks_empty_db() {
        let db = test_db();
        assert_eq!(db.count_tracks().unwrap(), 0);
    }

    #[test]
    fn insert_and_retrieve_track() {
        let db = test_db();
        let track = sample_track("spotify:track:abc123", "Test Song", "Test Artist");
        db.insert_track_info(&track).unwrap();

        let retrieved = db.get_track_info("spotify:track:abc123").unwrap();
        assert!(retrieved.is_some());
        let info = retrieved.unwrap();
        assert_eq!(info.track_name, "Test Song");
        assert_eq!(info.artist_name, "Test Artist");
        assert_eq!(info.lyrics, Some("Test lyrics".to_string()));
    }

    #[test]
    fn missing_track_returns_none() {
        let db = test_db();
        let result = db.get_track_info("spotify:track:doesnotexist").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn count_tracks_after_inserts() {
        let db = test_db();
        db.insert_track_info(&sample_track("id:1", "Song One", "Artist A"))
            .unwrap();
        db.insert_track_info(&sample_track("id:2", "Song Two", "Artist B"))
            .unwrap();
        db.insert_track_info(&sample_track("id:3", "Song Three", "Artist A"))
            .unwrap();
        assert_eq!(db.count_tracks().unwrap(), 3);
    }

    #[test]
    fn search_finds_by_artist() {
        let db = test_db();
        db.insert_track_info(&sample_track("id:1", "Alpha", "Radiohead"))
            .unwrap();
        db.insert_track_info(&sample_track("id:2", "Beta", "Portishead"))
            .unwrap();
        db.insert_track_info(&sample_track("id:3", "Gamma", "Radiohead"))
            .unwrap();

        let results = db.search_tracks("Radiohead").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_finds_by_track_name() {
        let db = test_db();
        db.insert_track_info(&sample_track("id:1", "Karma Police", "Radiohead"))
            .unwrap();
        db.insert_track_info(&sample_track("id:2", "Creep", "Radiohead"))
            .unwrap();

        let results = db.search_tracks("karma").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].track_name, "Karma Police");
    }

    #[test]
    fn get_recent_respects_limit() {
        let db = test_db();
        for i in 0..5 {
            db.insert_track_info(&sample_track(
                &format!("id:{}", i),
                &format!("Song {}", i),
                "Artist",
            ))
            .unwrap();
        }
        let recent = db.get_recent_tracks(3).unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn insert_replaces_existing_track() {
        let db = test_db();
        let track = sample_track("id:1", "Original", "Artist");
        db.insert_track_info(&track).unwrap();

        let updated = TrackInfo {
            track_name: "Updated".to_string(),
            ..sample_track("id:1", "Updated", "Artist")
        };
        db.insert_track_info(&updated).unwrap();

        assert_eq!(db.count_tracks().unwrap(), 1);
        let retrieved = db.get_track_info("id:1").unwrap().unwrap();
        assert_eq!(retrieved.track_name, "Updated");
    }

    #[test]
    fn schema_migration_is_idempotent() {
        let db = test_db();
        // Calling init() again should not fail
        db.init().unwrap();
        db.init().unwrap();
    }

    #[test]
    fn play_events_record_and_count() {
        let db = test_db();
        let track = sample_track("spotify:track:x1", "Song", "Artist");
        db.insert_track_info(&track).unwrap();

        let event_id = db.record_play_start("spotify:track:x1").unwrap();
        db.record_play_end(event_id, 200_000).unwrap();

        assert_eq!(db.get_play_count("spotify:track:x1").unwrap(), 1);
    }

    #[test]
    fn play_events_open_event_not_counted() {
        let db = test_db();
        let track = sample_track("spotify:track:x2", "Song", "Artist");
        db.insert_track_info(&track).unwrap();

        // Start event but don't end it — should not count
        db.record_play_start("spotify:track:x2").unwrap();

        assert_eq!(db.get_play_count("spotify:track:x2").unwrap(), 0);
    }

    #[test]
    fn get_stats_returns_empty_with_no_events() {
        let db = test_db();
        let stats = db.get_stats(30).unwrap();
        assert!(stats.top_tracks.is_empty());
        assert!(stats.top_artists.is_empty());
        assert!(stats.daily_ms.is_empty());
    }
}
