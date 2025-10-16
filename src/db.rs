use anyhow::{Context, Result};
use rusqlite::{params, Connection};

pub struct Database {
    conn: Connection,
}

#[derive(Debug)]
pub struct TrackInfo {
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub release_date: String,
    pub duration_ms: i64,
    pub popularity: i32,
    pub genres: String,
    pub lyrics: Option<String>,
    pub producers: String,
    pub writers: String,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {}", path))?;
        
        Ok(Self { conn })
    }

    pub fn init(&self) -> Result<()> {
        self.conn.execute(
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
        ).context("Failed to create tracks table")?;

        Ok(())
    }

    pub fn get_track_info(&self, track_id: &str) -> Result<Option<TrackInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id, track_name, artist_name, album_name, release_date, 
                    duration_ms, popularity, genres, lyrics, producers, writers
             FROM tracks WHERE track_id = ?1"
        )?;

        let track = stmt.query_row(params![track_id], |row| {
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
        });

        match track {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn insert_track_info(&self, info: &TrackInfo) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tracks 
             (track_id, track_name, artist_name, album_name, release_date, 
              duration_ms, popularity, genres, lyrics, producers, writers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
        ).context("Failed to insert track info")?;

        Ok(())
    }
}
