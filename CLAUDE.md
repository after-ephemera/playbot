# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                  # debug build
cargo build --release        # optimized build -> target/release/pb
cargo run -- <args>          # run locally, e.g. cargo run -- --search "bohemian"
cargo install --path .       # install the `pb` binary to ~/.cargo/bin/

cargo test                   # run all tests
cargo test <name>            # run a single test by name, e.g. cargo test search_finds_by_artist

cargo fmt                    # format (CI runs `cargo fmt --check`)
cargo clippy -- -D warnings  # lint; warnings are errors in CI
```

CI (`.github/workflows/ci.yml`) runs fmt-check, clippy (`-D warnings`), build, and test on `macos-latest`. Run all three (`fmt`, `clippy`, `test`) before submitting a PR.

## Architecture

`pb` is a single-binary Rust CLI (binary name `pb`, package name `playbot`) that reports the currently playing Spotify track with lyrics and metadata, caching everything in a local SQLite database. There are **no API keys / OAuth** — it reads the local Spotify desktop app directly and fetches lyrics from a keyless service.

### Data flow (the core path: `pb` with no args)

`main.rs` orchestrates everything in three stages:
1. `initialize()` — ensures `~/.pb/` exists, resolves the config path, runs one-time migrations of a legacy `config.toml`/`playbot.db` from CWD into `~/.pb/`, then opens and `init()`s the DB.
2. `dispatch()` — routes to a handler based on CLI flags (`--browse`, `--count`, `--search`, `--recent`, else now-playing).
3. `handle_now_playing()` — gets the current track from Spotify, checks the cache by `track_id`, fetches lyrics on a miss (or with `--refresh`), writes back to the cache, and prints.

### Module responsibilities

- **`spotify.rs`** — `SpotifyClient` reads the current track via macOS AppleScript (`osascript`). The script returns a `|`-delimited string parsed into a `TrackInfo`. `track_id` is the Spotify URI (`spotify:track:xxxxx`). **macOS-only**: non-macOS builds compile but return an error at runtime (`#[cfg(target_os = "macos")]`).
- **`db.rs`** — `Database` wraps a `rusqlite` `Connection`. `TrackInfo` is the central data struct shared across all modules (comma-separated strings for `genres`/`producers`/`writers`). Schema migrations are versioned via a `schema_version` table inside `init()` — add new migrations as `if current_version < N { ... }` blocks. Use `Database::new(":memory:")` for tests.
- **`lyrics.rs`** — `LyricsClient` wraps the `lyric_finder` crate (keyless). Never errors on missing lyrics; returns a "not found" message instead. Contains cleanup logic that strips Genius-style metadata artifacts ("N Contributors", title + "Lyrics" prefix).
- **`config.rs`** — loads TOML config (`~/.pb/config.toml`), whose only field is `database.path`. Expands a leading `~/` to `$HOME`.
- **`tui.rs`** — `ratatui` + `crossterm` interactive browser (`--browse`). `App` holds view state with `InputMode` (Normal/Editing for search) and `ViewMode` (List/Detail). Vim-style keys (`j/k`, `h/l`, `Enter`, `Esc`, `q`).

### Conventions

- Errors use `anyhow::Result` with `.context(...)` throughout; AppleScript/parse failures surface user-facing guidance.
- The DB layer never knows about Spotify or lyrics — handlers in `main.rs` compose the modules and assemble the final `TrackInfo` before caching.
- DB unit tests run on any platform (in-memory SQLite); full manual testing requires macOS with Spotify running and playing.
