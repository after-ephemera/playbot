# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed
- Renamed `genius.rs` to `lyrics.rs` and `GeniusClient` to `LyricsClient` — the module
  never used the Genius API directly, so the name was misleading
- Removed unused `[genius]` config section; the config file now only needs `[database]`
- Removed unused `reqwest` dependency
- Refactored monolithic `main()` into focused handler functions
- Unified `TrackInfoBasic` and `TrackInfo` into a single `TrackInfo` type
- Extracted repeated row-mapping code in `db.rs` into a shared helper
- Added database indexes on `cached_at` and `artist_name` columns (migration v2)
- `INSERT OR REPLACE` now explicitly updates `cached_at` timestamp on re-cache

### Added
- Unit tests for `db.rs` (insert, retrieve, search, recent, count, idempotent migrations)
- Doc comments on all public APIs
- LICENSE file (MIT)
- `.github/workflows/ci.yml` — CI on macOS with fmt, clippy, build, test
- `CONTRIBUTING.md`

## [0.1.0] - 2024

### Added
- Fetch currently playing track from local Spotify desktop app (macOS, no API key needed)
- Automatic lyrics fetching via `lyric_finder` (no API key required)
- SQLite caching for track metadata and lyrics
- Interactive TUI browser with vim-style navigation (j/k to move, Enter/l to view, h/Esc to go back, q to quit)
- Search across cached tracks by name, artist, or album (`pb --search`)
- Recently queried tracks view (`pb --recent`)
- Track count with celebration messages (`pb --count`)
- Force-refresh cached data (`pb --refresh`)
- TOML configuration file at `~/.pb/config.toml`
- Automatic migration from old `./config.toml` and `./playbot.db` locations
- Schema versioning for future database migrations
