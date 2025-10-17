![logo](playbot.png)

# playbot

A CLI tool that fetches detailed information about your currently playing Spotify song, including lyrics, artist details, and more.

## Features

- 🎵 Fetches currently playing track from local Spotify desktop app (no API credentials needed!)
- 📝 Gets lyrics automatically (no API key required!)
- 👤 Retrieves artist information
- 💿 Shows album details and track duration
- 💾 Caches results in local SQLite database for faster subsequent lookups
- 🔍 Search and browse your music library
- 📊 Interactive TUI browser with vim-style navigation
- 📚 View recently queried songs
- 🚀 Built with Rust for performance and reliability
- 🍎 macOS support (currently macOS only)

## Installation

Make sure you have Rust installed, then clone and build the project:

```bash
git clone https://github.com/after-ephemera/playbot.git
cd playbot
cargo build --release
```

The binary will be available at `target/release/pb`.

## Configuration

Create a `config.toml` file in the project root (you can copy from `config.toml.example`):

```toml
[genius]
# Optional - lyrics are fetched automatically without an API key
# access_token = "your_genius_access_token"

[database]
path = "playbot.db"
```

**Note:** The `playbot.db` file included in this repository is an example database with sample data. Your own database will be created when you first run the application.

**Note:** No Spotify API credentials are needed! The tool reads directly from your local Spotify desktop application. Lyrics are also fetched automatically without requiring a Genius API key.

## Requirements

- **Spotify Desktop App**: Must be installed and running with a song playing
- **macOS**: `osascript` (built-in) - currently only macOS is supported

## Usage

Make sure Spotify desktop app is running and playing a song, then run:

```bash
cargo run
# or if you built with --release
./target/release/pb
```

### Options

- `-c, --config <FILE>`: Path to configuration file (default: `config.toml`)
- `-r, --refresh`: Force refresh data even if cached
- `-b, --browse`: Launch interactive TUI browser to explore your music library
- `-s, --search <QUERY>`: Search database by song title or artist name
- `--recent`: Show recently queried songs
- `-n, --count`: Count total tracks in database
- `-h, --help`: Print help information

### Examples

```bash
# Get info about currently playing song
pb

# Use a custom config file
pb --config /path/to/config.toml

# Force refresh cached data
pb --refresh

# Browse your music library with interactive TUI
pb --browse

# Search for songs or artists
pb --search "bohemian"

# View recently queried songs
pb --recent

# Count tracks in your database
pb --count
```

## How It Works

1. Queries your local Spotify desktop app to get the currently playing track
   - **macOS**: Uses AppleScript via `osascript` to query Spotify
2. Checks the local SQLite cache for existing data
3. If not cached (or `--refresh` is used), fetches lyrics automatically (using lyric_finder)
4. Stores the data in the cache for future use
5. Displays all information in a formatted output

## Technologies Used

- **clap**: Command-line argument parsing
- **anyhow**: Error handling
- **lyric_finder**: Automatic lyrics fetching
- **rusqlite**: SQLite database for caching
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **toml**: Configuration file parsing
- **ratatui**: Terminal UI library for the interactive browser
- **crossterm**: Cross-platform terminal manipulation

## Benefits Over API Approach

✅ No Spotify API credentials needed
✅ No Genius API credentials needed
✅ No OAuth flow required
✅ Simpler setup - just install and run
✅ Works offline (for cached songs)
✅ Faster - direct local access
✅ More privacy - no data sent to Spotify servers
✅ Interactive TUI for browsing your music library

## License

MIT
