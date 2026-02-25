# Contributing to playbot

Thanks for your interest in contributing!

## Development Setup

1. Install [Rust](https://rustup.rs/) (stable toolchain)
2. Clone the repo and build:
   ```bash
   git clone https://github.com/after-ephemera/playbot.git
   cd playbot
   cargo build
   ```
3. Copy and edit the example config:
   ```bash
   mkdir -p ~/.pb
   cp config.toml.example ~/.pb/config.toml
   ```

**Note:** Full manual testing requires macOS with the Spotify desktop app running. The database unit tests work on any platform.

## Running Tests

```bash
cargo test
```

## Code Style

This project uses standard Rust formatting and Clippy linting:

```bash
cargo fmt
cargo clippy -- -D warnings
```

CI enforces both. Please run them before submitting a PR.

## Submitting a Pull Request

1. Fork the repo and create a branch from `main`
2. Make your changes, adding tests where appropriate
3. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass
4. Open a PR against `main` with a clear description of what changed and why

## Areas Where Contributions Are Welcome

- **Linux support** via D-Bus/playerctl (replacing the macOS AppleScript integration)
- **More tests** — especially for the TUI module and config loading
- **Bug fixes** — check the issue tracker
- **Documentation** — improving doc comments or the README is always welcome

## Reporting Bugs

Open an issue with:
- What you expected to happen
- What actually happened
- Your macOS version and Spotify version
- The output of `pb` with any error messages
