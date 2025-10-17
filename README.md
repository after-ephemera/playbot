![logo](playbot.png)

# playbot

A CLI tool that fetches detailed information about your currently playing Spotify song, including lyrics, artist details, and more.

## Features

- 🎵 Fetches currently playing track from local Spotify desktop app (no API credentials needed!)
- 📝 Gets lyrics from Genius API
- 👤 Retrieves artist information
- 💿 Shows album details and track duration
- 💾 Caches results in local SQLite database for faster subsequent lookups
- 🚀 Built with Rust for performance and reliability
- 🌍 Cross-platform support (Linux, macOS, Windows)

## Installation

Make sure you have Rust installed, then clone and build the project:

```bash
git clone https://github.com/after-ephemera/playbot.git
cd playbot
cargo build --release
```

The binary will be available at `target/release/playbot`.

## Configuration

Create a `config.toml` file in the project root (you can copy from `config.toml.example`):

```toml
[genius]
access_token = "your_genius_access_token"

[database]
path = "playbot.db"
```

### Getting Genius API Credentials

**Genius:**
1. Go to [Genius API Clients](https://genius.com/api-clients)
2. Create a new API client
3. Generate an access token

**Note:** No Spotify API credentials are needed! The tool reads directly from your local Spotify desktop application.

## Requirements

- **Spotify Desktop App**: Must be installed and running with a song playing
- **Platform-specific tools**:
  - **Linux**: `dbus-send` (usually pre-installed)
  - **macOS**: `osascript` (built-in)
  - **Windows**: PowerShell (built-in)

## Usage

Make sure Spotify desktop app is running and playing a song, then run:

```bash
cargo run
# or if you built with --release
./target/release/playbot
```

### Options

- `-c, --config <FILE>`: Path to configuration file (default: `config.toml`)
- `-r, --refresh`: Force refresh data even if cached
- `-h, --help`: Print help information

### Examples

```bash
# Use default config file
playbot

# Use a custom config file
playbot --config /path/to/config.toml

# Force refresh cached data
playbot --refresh
```

## How It Works

1. Queries your local Spotify desktop app to get the currently playing track
   - **Linux**: Uses D-Bus/MPRIS to communicate with Spotify
   - **macOS**: Uses AppleScript via `osascript` to query Spotify
   - **Windows**: Uses PowerShell to access Windows Media Control API
2. Checks the local SQLite cache for existing data
3. If not cached (or `--refresh` is used), fetches lyrics from Genius API
4. Stores the data in the cache for future use
5. Displays all information in a formatted output

## Technologies Used

- **clap**: Command-line argument parsing
- **anyhow**: Error handling
- **genius-rust**: Genius API client for lyrics
- **rusqlite**: SQLite database for caching
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **toml**: Configuration file parsing

## Benefits Over API Approach

✅ No Spotify API credentials needed
✅ No OAuth flow required
✅ Simpler setup - just install and run
✅ Works offline (for cached songs)
✅ Faster - direct local access
✅ More privacy - no data sent to Spotify servers

## License

MIT
