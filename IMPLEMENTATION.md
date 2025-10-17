# Playbot Implementation Summary

## Overview
A Rust CLI tool that fetches and displays detailed information about the currently playing Spotify song from the local desktop app, including automatic lyrics fetching, with local SQLite caching and an interactive TUI browser.

## Architecture

### Core Components

1. **Main Application (`src/main.rs`)**
   - CLI argument parsing using `clap`
   - Orchestrates the workflow between local Spotify client, lyrics fetching, and database
   - Error handling with `anyhow`
   - Displays formatted output
   - Handles multiple modes: current track, search, browse, recent, count

2. **Configuration Module (`src/config.rs`)**
   - Loads TOML configuration file
   - Manages database path settings
   - No API credentials needed!

3. **Database Module (`src/db.rs`)**
   - SQLite integration with `rusqlite`
   - Schema for caching track information
   - CRUD operations for track data
   - Search functionality across tracks
   - Stores: track details, lyrics, artist info, etc.

4. **Spotify Module (`src/spotify.rs`)**
   - Platform-specific local integration (no API!)
   - **macOS**: Uses AppleScript via `osascript` command
   - Fetches currently playing track from local Spotify desktop app
   - Retrieves Spotify URI, track name, artist, album, and duration

5. **Genius Module (`src/genius.rs`)**
   - Integration with `lyric_finder` crate
   - Automatically fetches lyrics without requiring API credentials
   - Searches for songs by title and artist
   - Cleans up lyrics metadata

6. **TUI Module (`src/tui.rs`)**
   - Interactive terminal UI browser using `ratatui` and `crossterm`
   - List view with all cached tracks
   - Detail view showing full track information and lyrics
   - Search functionality with live filtering
   - Vim-style keyboard navigation (j/k, h/l)

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

**macOS (AppleScript)**
   ```applescript
   tell application "Spotify"
     spotify url of current track  # Returns spotify:track:xxxxx URI
     name of current track
     artist of current track
     album of current track
     duration of current track
   end tell
   ```

Note: Currently only macOS is supported. Linux (D-Bus/MPRIS) and Windows (PowerShell) support could be added in the future.

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

### Default Mode (Current Track)
1. User runs `pb`
2. Load configuration from `config.toml` (no API credentials needed)
3. Initialize SQLite database (creates schema if needed)
4. Query local Spotify desktop app for currently playing track
5. Check cache for existing data
   - If found and `--refresh` not set: Display cached data
   - If not found or `--refresh` set: Fetch fresh data
6. Fetch lyrics automatically via lyric_finder
7. Combine and store all information in cache
8. Display formatted output

### Browse Mode
1. User runs `pb --browse`
2. Launch interactive TUI browser
3. Display all cached tracks in a list
4. Support navigation, search, and detail viewing

### Search Mode
1. User runs `pb --search "query"`
2. Search database for matching tracks
3. Highlight currently playing track if found in results

### Other Modes
- `--recent`: Display 10 most recently queried tracks
- `--count`: Show total track count with celebration message

## Features Implemented

✅ TOML configuration file support (no API credentials needed!)
✅ Local Spotify integration (macOS via AppleScript)
✅ Automatic lyrics fetching via lyric_finder (no API key needed)
✅ SQLite caching system with search capabilities
✅ CLI with clap (multiple modes and flags)
✅ Interactive TUI browser with vim-style navigation
✅ Search functionality across cached tracks
✅ Recent tracks view
✅ Track count with celebration messages
✅ Error handling with anyhow
✅ Async/await with tokio
✅ Formatted console output with emojis
✅ Track metadata (album, duration, Spotify URI)
✅ Currently playing track highlighting in search results

## Usage Example

```bash
# First time setup
cp config.toml.example config.toml
# Edit config.toml if needed (no API credentials required!)

# Make sure Spotify desktop app is running and playing a song

# Run the application
cargo run

# Or use the release binary
cargo build --release
./target/release/pb

# Get info about currently playing track
pb

# Force refresh cached data
pb --refresh

# Browse your music library interactively
pb --browse

# Search for tracks
pb --search "bohemian"

# View recent tracks
pb --recent

# Count tracks in database
pb --count

# Use custom config file
pb --config /path/to/config.toml
```

## Dependencies

- `clap` v4.5 - CLI argument parsing with derive macros
- `anyhow` v1.0 - Error handling
- `lyric_finder` v0.2 - Automatic lyrics fetching (no API key needed)
- `rusqlite` v0.32 - SQLite database with bundled feature
- `tokio` v1.40 - Async runtime with full features
- `serde` v1.0 - Serialization with derive
- `toml` v0.8 - TOML configuration parsing
- `reqwest` v0.12 - HTTP client with JSON support
- `ratatui` v0.29 - Terminal UI library
- `crossterm` v0.28 - Cross-platform terminal manipulation

**Note:** No `rspotify` or Spotify API credentials needed! No Genius API credentials needed!

## Configuration

The application requires a minimal `config.toml` file with:
- Database file path (required)
- Genius API access token (optional - no longer needed)

See `config.toml.example` for the template.

## TUI Browser Controls

When running `pb --browse`:
- **j/Down**: Move down in list, or scroll down in detail view
- **k/Up**: Move up in list, or scroll up in detail view
- **Enter**: Toggle between list and detail view
- **/**: Enter search mode
- **h/Left**: Previous track (in detail view)
- **l/Right**: Next track (in detail view)
- **Esc**: Return to list view from detail view, or cancel search
- **q**: Quit the browser
