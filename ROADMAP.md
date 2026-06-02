# Playbot Product Roadmap — Next High-Impact Features

## Context

Playbot (`pb`) is a lightweight, zero-API Spotify CLI tool for macOS that reads the current track via AppleScript, caches metadata in SQLite, and provides a TUI browser, analytics, and MCP server for AI agents. The goal is to identify high-leverage features that would expand its utility and adoption.

Research across Reddit, HN, GitHub issues, and competing tools (spotify-player, sptlrx, ncspot, Spotify-TUI, spotifatius) reveals strong, repeated demand for features that are **missing from most terminal clients** and **directly drive user adoption**. The current `pb` feature set is solid but incomplete relative to what power users are building workarounds for.

## Strategic Insights from Research

### Why Users Choose Terminal Spotify Tools
1. **RAM efficiency** — Spotify desktop app uses 300–500MB; terminal clients use 40–100MB (13x improvement cited as primary reason for switching)
2. **Flow state** — avoiding Alt-Tab to skip a track
3. **No bloat** — no ads, podcasts, algorithmic noise; just library + playlists + search

### Ecosystem Pattern: Burnout Cycle
Most Spotify terminal tools are abandoned or broken because:
- **Spotify API deprecation without warning** (November 2024 killed audio features, recommendations, related artists)
- **Spotify AppleScript breaks across macOS updates** (fragile but the only free option on macOS)
- **Daemon setup complexity** — users abandon tools with multi-step setups (spotify-tui + spotifyd)

### What Drives Adoption (Tier 1 Dealbreakers)
1. **Synced lyrics** — sptlrx, spotify-player, spotatui all added this independently. High signal.
2. **Statusbar integration** — entire category of tools (polybar-spotify, spotifatius, waybar plugins) exist solely for this.
3. **Scriptable/pipeable output** — users want `pb | jq`, structured JSON, stable CLI contracts.
4. **No separate daemon** — tools that "just work" out of the box win.

### Where `pb` Can Win
- **macOS-first** with rock-solid AppleScript (pb is already ahead here; spotify-player is cross-platform and thus less optimized for macOS)
- **Lightweight single-binary** (already true)
- **Offline-capable** via local SQLite caching (already true)
- **Integrated daemon + TUI + MCP** (unusual combination; most tools pick one)

---

## Recommended Roadmap: Four Phases

### Phase 1: Statusbar Integration & Scripting (HIGH IMPACT, 1–2 weeks)

**Why:** Three separate ecosystems (polybar, waybar, tmux) desperately want this. Users currently write their own shell scripts or poll inefficiently. First tool to provide a clean, standardized approach wins mindshare.

**What to build:**
1. **`pb --status` output mode** — optimized for statusbar/shell integration
   - Single line: `♫ Artist - Track (2:15 / 3:45) | ▶` (or `||` if paused)
   - Customizable format string (e.g., `pb --status-format "{artist} - {track}"`)
   - Max width with intelligent truncation (e.g., `pb --status --max-width 30`)
   - Exit code: 0 if playing, 1 if paused, 2 if nothing loaded
   - Support for simple placeholders: `{artist}`, `{track}`, `{album}`, `{duration}`, `{position}`, `{play_state}` (Playing/Paused/Stopped)

2. **Event-driven output for statusbar daemons**
   - `pb --daemon --status-output <FILE>` — writes a status JSON to a file that refreshes on track change (not polling)
   - Useful for: polybar's ipc mode, waybar event hooks, tmux run-shell
   - Example output: `{"artist":"Drake","track":"One Dance","state":"playing","position_ms":120000,"duration_ms":205000}`

3. **JSON output stability (already exists, document it)**
   - Codify the JSON schema in a `--json-schema` output or docs
   - Ensure exit codes are meaningful for scripting (0=success, non-zero=error)

**Files to modify:**
- `src/main.rs` — add `--status`, `--status-format`, `--max-width`, `--status-output` flags; add `StatusOutputMode` enum
- `src/config.rs` — optionally load default status format from config
- New helper function in `src/tui.rs` or new `src/format.rs` — format track info as single-line string with truncation logic
- `Cargo.toml` — if adding fancy truncation, add `unicode-width` or similar

**Success criteria:**
- `pb --status` outputs a one-liner suitable for polybar/waybar/tmux
- Users can integrate `pb --status` into polybar config in 3 lines
- Example: polybar module works without custom scripts

**Estimated effort:** 2–3 days (straightforward, no new dependencies needed)

---

### Phase 2: Synced Lyrics Display (MEDIUM IMPACT, 2–3 weeks)

**Why:** Synced lyrics (time-aligned with playback) is the single most-added feature across competing tools (spotify-player, sptlrx, spotatui, lyricstify). HN/Reddit discussion treats it as a differentiator.

**Current state:** `pb` fetches lyrics via `lyric_finder` but doesn't display time sync.

**What to build:**
1. **Spotify rich metadata for lyrics source**
   - `pb` currently doesn't fetch Spotify's native lyric sync capability (available in recent Spotify API for premium users; not via AppleScript but can be cached)
   - **Alternative (no API):** Fetch from Genius using existing `lyric_finder` crate + reverse-engineer line breaks + use `duration_ms` to synthesize timing (rough but usable)
   - **Better alternative:** Use a dedicated synced-lyrics crate (e.g., `synced-lyrics` or `lrc` parsing libraries on crates.io) with fallback to Genius

2. **TUI lyric view with progress bar**
   - `pb --browse` should support a lyric view mode (already has `ViewMode`; add `ViewMode::Lyrics`)
   - Display current + next 2–3 lines; highlight the active line as playback progresses
   - Arrow keys or `j/k` to scroll manually; real-time progress update every 100ms

3. **CLI mode: `pb --lyrics --synced`**
   - Print synced lyrics to stdout in a readable format (e.g., timestamp + line)
   - Example output:
     ```
     0:00 - 0:10  | Some lyrics here
     0:10 - 0:20  | More lyrics
     ```

**Files to modify:**
- `src/lyrics.rs` — extend `LyricsClient` to support synced lyrics; add a new `SyncedLyric` struct with timing info
- `src/tui.rs` — add `ViewMode::Lyrics`, implement rendering + real-time progress update
- `src/main.rs` — add `--synced` flag, route to lyric view handler
- `Cargo.toml` — add dependency (e.g., `lrc` crate for parsing LRC files, or geolocation-rs for reverse-geocoding timing)

**Success criteria:**
- `pb --browse` can display synced lyrics with current line highlighted
- `pb --lyrics --synced` outputs formatted synced text
- Plays inline with existing `pb` workflow (no breaking changes)

**Estimated effort:** 3–4 days (lyric sync source is the hardest part; LRC parsing is simple)

**Note:** This is the #1 feature request in the ecosystem and would be a major differentiator for `pb`.

---

### Phase 3: Track Interaction (MEDIUM IMPACT, 1–2 weeks)

**Why:** Users want to like/save tracks and manipulate the queue from the CLI. Currently, `pb` is read-only. This is a commonly-scripted workflow (e.g., `pb --like` in a shell function).

**What to build:**
1. **`pb --like` / `pb --unlike` to toggle liked status**
   - Requires AppleScript to interface with Spotify's native "Add to Liked Songs"
   - Can be implemented via `pb --like --track <TRACK_ID>` or just applied to now-playing

2. **`pb --queue` commands**
   - `pb --queue view` — show upcoming queue
   - `pb --queue add <TRACK_ID>` — add track to queue
   - `pb --queue next` — skip to next
   - `pb --queue prev` — skip to previous

3. **TUI integration**
   - In `--browse` mode, pressing `L` toggles like status on the current track (visual feedback)
   - Queuing tracks from search results (e.g., press `a` to add to queue)

**Files to modify:**
- `src/spotify.rs` — add methods to like/unlike tracks via AppleScript, query queue
- `src/main.rs` — add `--like`, `--unlike`, `--queue` flags and handlers
- `src/tui.rs` — add key bindings for like/queue actions

**Success criteria:**
- `pb --like` successfully saves the current track to liked songs
- `pb --queue add <track>` adds a track to the queue
- Can be scripted: `pb --like && echo "Liked!"`

**Estimated effort:** 2–3 days (AppleScript extensions are straightforward; queue management slightly more complex)

---

### Phase 4 (Future): Last.fm Scrobbling & Audio Features

**Why:** Last.fm scrobbling is a long-standing gap in the Spotify CLI ecosystem. It's a "dealbreaker" feature for users who use Last.fm for music discovery and stats. However, it requires either:
- A separate Last.fm API integration (requires user to create account + API key)
- Manual integration with `fm-notify` or `fish-shell` plugins

**Current state:** `pb --daemon` already records `play_events`; scrobbling would integrate this data to Last.fm.

**Not recommended for Phase 1** because it:
- Requires external API credentials
- Is niche compared to statusbar integration
- Can be added later without breaking existing code

---

## Implementation Priorities

| Phase | Feature | Impact | Effort | Dependencies | Recommended Timing |
|---|---|---|---|---|---|
| 1 | Statusbar integration | HIGH | 2–3 days | None | **Now** |
| 2 | Synced lyrics | MEDIUM | 3–4 days | LRC crate or Genius scraping | Week 2 |
| 3 | Track interaction | MEDIUM | 2–3 days | AppleScript extensions | Week 3 |
| 4 | Last.fm scrobbling | MEDIUM | 4–5 days | last.fm-rs crate, user config | Month 2 |

---

## Success Metrics

After implementing Phase 1–3, `pb` should:
1. **Be the easiest Spotify CLI to integrate into a statusbar** (winning over competitors)
2. **Offer synced lyrics parity with spotify-player and spotatui** (rare feature, high adoption driver)
3. **Support basic track manipulation from the CLI** (enabling shell alias workflows)
4. **Maintain macOS-first focus** with rock-solid, documented AppleScript integration

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| AppleScript breaks in future macOS | HIGH | Document AppleScript API surface; add CI test that checks script syntax |
| Spotify API future changes | LOW (pb uses AppleScript, not API) | Monitor Spotify release notes; keep daemon mode optional |
| Synced lyrics source becomes unavailable | MEDIUM | Support multiple sources (Genius fallback, local LRC files, Spotify API if accessible) |
| Feature creep dilutes focus | MEDIUM | Stick to the roadmap; ship Phase 1 before evaluating Phase 2 |

---

## Verification Plan

1. **Phase 1 (Statusbar):** Test with polybar config; ensure JSON output parses correctly; verify truncation works at edge cases
2. **Phase 2 (Synced Lyrics):** Compare rendering against spotify-player; test with songs that have/without lyrics; verify real-time progress updates
3. **Phase 3 (Track Interaction):** Verify AppleScript calls succeed; test queue add/next/prev; confirm TUI key bindings work
4. **Overall:** Run full test suite (`cargo test`), clippy, fmt; manually test on M1 Mac with Spotify running

---

## What to Build First

**Start with Phase 1 (Statusbar Integration).** It is:
- **Lowest risk** — no new dependencies, pure Rust string formatting
- **Highest immediate impact** — solves a concrete pain point for a large user segment
- **Fastest to ship** — 2–3 days to working implementation
- **Enables follow-up features** — Phase 2 and 3 naturally follow, building on the same infrastructure
