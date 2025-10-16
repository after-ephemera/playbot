# playbot

A CLI tool that fetches detailed information about your currently playing Spotify song, including lyrics, artist details, and more.

## Features

- 🎵 Fetches currently playing track from Spotify
- 📝 Gets lyrics from Genius API
- 👤 Retrieves artist information and genres
- 💿 Shows album details, release date, and popularity
- 💾 Caches results in local SQLite database for faster subsequent lookups
- 🚀 Built with Rust for performance and reliability

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
[spotify]
client_id = "your_spotify_client_id"
client_secret = "your_spotify_client_secret"

[genius]
access_token = "your_genius_access_token"

[database]
path = "playbot.db"
```

### Getting API Credentials

**Spotify:**
1. Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Create a new app
3. Copy the Client ID and Client Secret
4. Add `http://localhost:8888/callback` as a Redirect URI in your app settings

**Genius:**
1. Go to [Genius API Clients](https://genius.com/api-clients)
2. Create a new API client
3. Generate an access token

## Usage

Run the tool to get information about your currently playing Spotify track:

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

1. Connects to Spotify API to get your currently playing track
2. Checks the local SQLite cache for existing data
3. If not cached (or `--refresh` is used), fetches:
   - Lyrics from Genius API
   - Artist genres from Spotify
   - Track details (album, release date, popularity)
4. Stores the data in the cache for future use
5. Displays all information in a formatted output

## Technologies Used

- **clap**: Command-line argument parsing
- **anyhow**: Error handling
- **rspotify**: Spotify API client
- **genius-rust**: Genius API client for lyrics
- **rusqlite**: SQLite database for caching
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **toml**: Configuration file parsing

## License

MIT

