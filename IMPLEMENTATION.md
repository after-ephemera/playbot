# Playbot Implementation Summary

## Overview
A Rust CLI tool that fetches and displays detailed information about the currently playing Spotify song, including lyrics from Genius API, with local SQLite caching.

## Architecture

### Core Components

1. **Main Application (`src/main.rs`)**
   - CLI argument parsing using `clap`
   - Orchestrates the workflow between Spotify, Genius, and database
   - Error handling with `anyhow`
   - Displays formatted output

2. **Configuration Module (`src/config.rs`)**
   - Loads TOML configuration file
   - Validates Spotify and Genius credentials
   - Manages database path settings

3. **Database Module (`src/db.rs`)**
   - SQLite integration with `rusqlite`
   - Schema for caching track information
   - CRUD operations for track data
   - Stores: track details, lyrics, artist info, genres, etc.

4. **Spotify Module (`src/spotify.rs`)**
   - Integration with Spotify API via `rspotify`
   - OAuth authentication flow
   - Fetches currently playing track
   - Retrieves artist details and genres

5. **Genius Module (`src/genius.rs`)**
   - Integration with Genius API via `genius-rust`
   - Searches for songs by title and artist
   - Fetches lyrics

## Database Schema

```sql
CREATE TABLE tracks (
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
)
```

## Workflow

1. User runs `playbot`
2. Load configuration from `config.toml`
3. Initialize SQLite database (creates schema if needed)
4. Authenticate with Spotify (OAuth flow)
5. Get currently playing track from Spotify
6. Check cache for existing data
   - If found and `--refresh` not set: Display cached data
   - If not found or `--refresh` set: Fetch fresh data
7. Fetch lyrics from Genius API
8. Combine and store all information in cache
9. Display formatted output

## Features Implemented

✅ TOML configuration file support
✅ Spotify API integration for current track
✅ Genius API integration for lyrics  
✅ SQLite caching system
✅ CLI with clap (help, config path, refresh flag)
✅ Error handling with anyhow
✅ Async/await with tokio
✅ Formatted console output with emojis
✅ Artist genres from Spotify
✅ Track metadata (album, release date, popularity, duration)

## Usage Example

```bash
# First time setup
cp config.toml.example config.toml
# Edit config.toml with your API credentials

# Run the application
cargo run

# Or use the release binary
cargo build --release
./target/release/playbot

# Force refresh cached data
playbot --refresh

# Use custom config file
playbot --config /path/to/config.toml
```

## Dependencies

- `clap` v4.5 - CLI argument parsing
- `anyhow` v1.0 - Error handling
- `rspotify` v0.13 - Spotify API client
- `genius-rust` v0.1 - Genius API client
- `rusqlite` v0.32 - SQLite database
- `tokio` v1.40 - Async runtime
- `serde` v1.0 - Serialization
- `toml` v0.8 - TOML parsing
- `reqwest` v0.12 - HTTP client

## Configuration

The application requires a `config.toml` file with:
- Spotify client ID and secret
- Genius API access token
- Database file path

See `config.toml.example` for the template.
