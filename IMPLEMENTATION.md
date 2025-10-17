# Playbot Implementation Summary

## Overview
A Rust CLI tool that fetches and displays detailed information about the currently playing Spotify song from the local desktop app, including lyrics from Genius API, with local SQLite caching.

## Architecture

### Core Components

1. **Main Application (`src/main.rs`)**
   - CLI argument parsing using `clap`
   - Orchestrates the workflow between local Spotify client, Genius, and database
   - Error handling with `anyhow`
   - Displays formatted output

2. **Configuration Module (`src/config.rs`)**
   - Loads TOML configuration file
   - Validates Genius credentials
   - Manages database path settings
   - No Spotify API credentials needed!

3. **Database Module (`src/db.rs`)**
   - SQLite integration with `rusqlite`
   - Schema for caching track information
   - CRUD operations for track data
   - Stores: track details, lyrics, artist info, etc.

4. **Spotify Module (`src/spotify.rs`)**
   - Platform-specific local integration (no API!)
   - **Linux**: Uses D-Bus/MPRIS via `dbus-send` command
   - **macOS**: Uses AppleScript via `osascript` command
   - **Windows**: Uses PowerShell to access Windows Media Control API
   - Fetches currently playing track from local Spotify desktop app

5. **Genius Module (`src/genius.rs`)**
   - Integration with Genius API via `genius-rust`
   - Searches for songs by title and artist
   - Fetches lyrics

## Local Spotify Integration

### Why Local Instead of API?

The tool queries the Spotify desktop application directly instead of using the Spotify Web API:

**Benefits:**
- ✅ No API credentials needed
- ✅ No OAuth authentication flow
- ✅ Simpler setup for users
- ✅ Works offline (with cache)
- ✅ Faster response time
- ✅ More privacy (no data sent to Spotify servers)
- ✅ No rate limits

**Platform Implementation:**

1. **Linux (D-Bus/MPRIS)**
   ```bash
   dbus-send --print-reply --dest=org.mpris.MediaPlayer2.spotify \
     /org/mpris/MediaPlayer2 org.freedesktop.DBus.Properties.Get \
     string:org.mpris.MediaPlayer2.Player string:Metadata
   ```

2. **macOS (AppleScript)**
   ```applescript
   tell application "Spotify"
     name of current track
     artist of current track
     album of current track
   end tell
   ```

3. **Windows (PowerShell + Media Control API)**
   Uses `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`

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
2. Load configuration from `config.toml` (only Genius token needed)
3. Initialize SQLite database (creates schema if needed)
4. Query local Spotify desktop app for currently playing track
5. Check cache for existing data
   - If found and `--refresh` not set: Display cached data
   - If not found or `--refresh` set: Fetch fresh data
6. Fetch lyrics from Genius API
7. Combine and store all information in cache
8. Display formatted output

## Features Implemented

✅ TOML configuration file support (simplified - no Spotify creds needed!)
✅ Local Spotify integration (Linux/macOS/Windows)
✅ Genius API integration for lyrics  
✅ SQLite caching system
✅ CLI with clap (help, config path, refresh flag)
✅ Error handling with anyhow
✅ Async/await with tokio
✅ Formatted console output with emojis
✅ Track metadata (album, duration)
✅ Cross-platform support

## Usage Example

```bash
# First time setup
cp config.toml.example config.toml
# Edit config.toml with your Genius API token (no Spotify credentials needed!)

# Make sure Spotify desktop app is running and playing a song

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
- `genius-rust` v0.1 - Genius API client
- `rusqlite` v0.32 - SQLite database
- `tokio` v1.40 - Async runtime
- `serde` v1.0 - Serialization
- `toml` v0.8 - TOML parsing
- `reqwest` v0.12 - HTTP client

**Note:** No longer requires `rspotify` or Spotify API credentials!

## Configuration

The application requires a minimal `config.toml` file with:
- Genius API access token
- Database file path

See `config.toml.example` for the template.
