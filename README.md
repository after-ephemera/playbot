![logo](playbot.png)

# playbot

[![CI](https://github.com/after-ephemera/playbot/actions/workflows/ci.yml/badge.svg)](https://github.com/after-ephemera/playbot/actions/workflows/ci.yml)

A CLI tool that fetches detailed information about your currently playing Spotify song, including lyrics, artist details, and more.

## Features

- 🎵 Fetches currently playing track from local Spotify desktop app (no API credentials needed!)
- 📝 Gets lyrics automatically (no API key required!)
- 💾 Caches results in local SQLite database for faster subsequent lookups
- 🔍 Search and browse your music library
- 📊 Interactive TUI browser with vim-style navigation
- 📚 View recently queried songs
- 🚀 Built with Rust for performance and reliability
- 🍎 macOS support (currently macOS only)

## Installation

Make sure you have Rust installed, then clone and install:

```bash
git clone https://github.com/after-ephemera/playbot.git
cd playbot
cargo install --path .
```

This installs the `pb` binary to `~/.cargo/bin/`. Alternatively, build without installing:

```bash
cargo build --release
# binary at: target/release/pb
```

## Configuration

On first run, `pb` creates a `~/.pb/` directory. Copy the example config:

```bash
cp config.toml.example ~/.pb/config.toml
```

The config file only needs a database path:

```toml
[database]
path = "~/.pb/playbot.db"
```

## Requirements

- **Spotify Desktop App**: Must be installed and running with a song playing
- **macOS**: `osascript` (built-in) — currently only macOS is supported

## Usage

Make sure Spotify desktop app is running and playing a song, then run:

```bash
pb
```

### Options

- `-c, --config <FILE>`: Path to configuration file (default: `~/.pb/config.toml`)
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

# Use a custom config file
pb --config /path/to/config.toml
```

### TUI Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` / `l` | View track details |
| `h` / `Esc` | Go back |
| `q` | Quit |

## How It Works

1. Queries your local Spotify desktop app to get the currently playing track via AppleScript
2. Checks the local SQLite cache for existing data
3. If not cached (or `--refresh` is used), fetches lyrics automatically
4. Stores the data in the cache for future use
5. Displays all information in a formatted output

## Why No API Keys?

✅ No Spotify API credentials needed
✅ No OAuth flow required
✅ Simpler setup — just install and run
✅ Works offline (for cached songs)
✅ Faster — direct local access
✅ More private — no data sent to external servers

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style guidelines, and how to submit a pull request.

## License

MIT — see [LICENSE](LICENSE).
