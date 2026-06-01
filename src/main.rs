mod config;
mod daemon;
mod db;
mod lyrics;
mod mcp;
mod spotify;
mod tui;

use anyhow::Result;
use clap::Parser;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "playbot")]
#[command(about = "Get detailed information about the currently playing Spotify song", long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Force refresh data even if cached
    #[arg(short, long)]
    refresh: bool,

    /// Show recently queried songs
    #[arg(long)]
    recent: bool,

    /// Browse database with interactive TUI
    #[arg(short, long)]
    browse: bool,

    /// Search database by song title or artist name
    #[arg(short, long)]
    search: Option<String>,

    /// Count total tracks in database
    #[arg(short = 'n', long)]
    count: bool,

    /// Show listening analytics (top tracks, artists, daily time)
    #[arg(long)]
    stats: bool,

    /// Number of days to include in stats (default: 30)
    #[arg(long, default_value = "30")]
    days: usize,

    /// Run daemon + MCP server — polls Spotify, records play events, and serves MCP on stdio
    #[arg(long)]
    daemon: bool,

    /// Poll interval for daemon mode in seconds (default: 5)
    #[arg(long, default_value = "5")]
    poll: u64,

    /// Expose playbot as an MCP server over stdio
    #[arg(long)]
    serve: bool,

    /// Output results as JSON (for agent/script use)
    #[arg(short = 'j', long)]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (config, db) = initialize(&cli)?;
    dispatch(cli, config, db).await
}

fn initialize(cli: &Cli) -> Result<(config::Config, db::Database)> {
    config::Config::ensure_app_dir()?;
    let config_path = resolve_config_path(cli)?;
    let config = config::Config::load(&config_path)?;
    migrate_database(&config)?;
    let db = db::Database::new(&config.database.path)?;
    db.init()?;
    Ok((config, db))
}

fn resolve_config_path(cli: &Cli) -> Result<String> {
    if let Some(path) = &cli.config {
        return Ok(path.clone());
    }

    let default_path = config::Config::get_default_config_path()?;
    let old_config = std::path::PathBuf::from("config.toml");

    if !default_path.exists() && old_config.exists() {
        println!(
            "📦 Migrating config from {} to {:?}",
            old_config.display(),
            default_path
        );
        std::fs::copy(&old_config, &default_path)?;
    }

    if !default_path.exists() {
        println!("⚠️  Config file not found at {:?}", default_path);
        println!(
            "Please create one or copy config.toml.example to {:?}",
            default_path
        );
        std::process::exit(1);
    }

    Ok(default_path.to_string_lossy().to_string())
}

fn migrate_database(config: &config::Config) -> Result<()> {
    let old_db = std::path::PathBuf::from("playbot.db");
    let new_db = std::path::PathBuf::from(&config.database.path);
    if old_db.exists() && old_db != new_db && !new_db.exists() {
        println!(
            "📦 Migrating database from {} to {}",
            old_db.display(),
            new_db.display()
        );
        std::fs::copy(&old_db, &new_db)?;
    }
    Ok(())
}

async fn dispatch(cli: Cli, config: config::Config, db: db::Database) -> Result<()> {
    if cli.daemon {
        return daemon::run(&config.database.path, cli.poll).await;
    }
    if cli.serve {
        return mcp::serve(db).await;
    }
    if cli.browse {
        return tui::run(db);
    }
    if cli.count {
        return handle_count(&db, cli.json);
    }
    if let Some(query) = &cli.search {
        return handle_search(&db, query, cli.json).await;
    }
    if cli.recent {
        return handle_recent(&db, cli.json);
    }
    if cli.stats {
        return handle_stats(&db, cli.days, cli.json);
    }
    handle_now_playing(cli, config, db).await
}

fn handle_count(db: &db::Database, as_json: bool) -> Result<()> {
    let count = db.count_tracks()?;

    if as_json {
        println!("{}", json!({"count": count}));
        return Ok(());
    }

    let celebration = match count {
        0 => "Your music library is empty! Time to start exploring!",
        1 => "You've got your first track! The journey begins!",
        2..=10 => "Nice start! You're building a collection!",
        11..=50 => "Great collection! You're really getting into it!",
        51..=100 => "Impressive library! You've got serious variety!",
        101..=500 => "Wow! You're a true music enthusiast!",
        501..=1000 => "Absolutely incredible! Your library is massive!",
        _ => "LEGENDARY STATUS! Your music collection is epic!",
    };

    println!("🎵 Total tracks in database: {}", count);
    println!("🎉 {}", celebration);

    Ok(())
}

async fn handle_search(db: &db::Database, query: &str, as_json: bool) -> Result<()> {
    let results = db.search_tracks(query)?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if results.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    let current_track_id = match spotify::SpotifyClient::new() {
        Ok(client) => match client.get_current_track().await {
            Ok(track) => Some(track.track_id),
            Err(_) => None,
        },
        Err(_) => None,
    };

    println!("Found {} result(s) for '{}':\n", results.len(), query);
    for (i, track) in results.iter().enumerate() {
        let is_playing = current_track_id.as_ref() == Some(&track.track_id);

        if is_playing {
            println!(
                "\x1b[1;92m{}. 🎵 {} by {} ⚡ NOW PLAYING ⚡\x1b[0m",
                i + 1,
                track.track_name,
                track.artist_name
            );
        } else {
            println!("{}. {} by {}", i + 1, track.track_name, track.artist_name);
        }
        println!("   Album: {}", track.album_name);
        if !track.release_date.is_empty() {
            println!("   Released: {}", track.release_date);
        }
        println!();
    }

    Ok(())
}

fn handle_recent(db: &db::Database, as_json: bool) -> Result<()> {
    let recent_tracks = db.get_recent_tracks(10)?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&recent_tracks)?);
        return Ok(());
    }

    if recent_tracks.is_empty() {
        println!("No recently queried songs found in the database.");
        return Ok(());
    }

    println!("📚 Recently Queried Songs:\n");
    for (i, track) in recent_tracks.iter().enumerate() {
        println!("{}. {} by {}", i + 1, track.track_name, track.artist_name);
        println!("   Album: {}", track.album_name);
        if !track.release_date.is_empty() {
            println!("   Released: {}", track.release_date);
        }
        println!();
    }

    Ok(())
}

fn handle_stats(db: &db::Database, days: usize, as_json: bool) -> Result<()> {
    let stats = db.get_stats(days)?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    println!("📊 Listening Stats (last {} days)\n", days);

    if stats.top_tracks.is_empty() {
        println!("No listening data yet. Run `pb --daemon` in the background to start tracking.");
        return Ok(());
    }

    println!("🎵 Top Tracks:");
    for (i, ts) in stats.top_tracks.iter().enumerate() {
        let mins = ts.total_ms / 60_000;
        let secs = (ts.total_ms % 60_000) / 1000;
        println!(
            "  {}. {} — {} ({} plays, {}m {:02}s)",
            i + 1,
            ts.track.track_name,
            ts.track.artist_name,
            ts.play_count,
            mins,
            secs
        );
    }

    println!("\n👤 Top Artists:");
    for (i, artist) in stats.top_artists.iter().enumerate() {
        let hours = artist.total_ms / 3_600_000;
        let mins = (artist.total_ms % 3_600_000) / 60_000;
        println!(
            "  {}. {} — {}h {:02}m",
            i + 1,
            artist.artist_name,
            hours,
            mins
        );
    }

    println!("\n📅 Daily Listening:");
    for (date, ms) in &stats.daily_ms {
        let mins = ms / 60_000;
        let bar_len = (mins / 5).min(30) as usize;
        let bar = "█".repeat(bar_len);
        println!("  {}  {:30}  {}m", date, bar, mins);
    }

    Ok(())
}

async fn handle_now_playing(cli: Cli, config: config::Config, db: db::Database) -> Result<()> {
    let spotify_client = spotify::SpotifyClient::new()?;
    let track_info = spotify_client.get_current_track().await?;

    if !cli.refresh {
        if let Some(cached_info) = db.get_track_info(&track_info.track_id)? {
            if cli.json {
                let play_count = db.get_play_count(&track_info.track_id).unwrap_or(0);
                let mut v = serde_json::to_value(&cached_info)?;
                v["play_count"] = json!(play_count);
                v["source"] = json!("cache");
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!(
                    "🎵 Now Playing: {} by {}",
                    cached_info.track_name, cached_info.artist_name
                );
                println!("\n📦 (Using cached data)\n");
                print_track_info(&cached_info);
            }
            return Ok(());
        }
    }

    let lyrics_client = lyrics::LyricsClient::new();
    let lyric_text = lyrics_client
        .get_lyrics(&track_info.track_name, &track_info.artist_name)
        .await?;

    let full_info = db::TrackInfo {
        lyrics: Some(lyric_text),
        ..track_info
    };

    db.insert_track_info(&full_info)?;

    if cli.json {
        let play_count = db.get_play_count(&full_info.track_id).unwrap_or(0);
        let mut v = serde_json::to_value(&full_info)?;
        v["play_count"] = json!(play_count);
        v["source"] = json!("live");
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!(
            "🎵 Now Playing: {} by {}",
            full_info.track_name, full_info.artist_name
        );
        println!("\n✨ Fresh data fetched!\n");
        print_track_info(&full_info);
    }

    let _ = config;
    Ok(())
}

fn print_track_info(info: &db::TrackInfo) {
    println!("📀 Track: {}", info.track_name);
    println!("👤 Artist: {}", info.artist_name);
    println!("💿 Album: {}", info.album_name);

    if !info.release_date.is_empty() {
        println!("📅 Release Date: {}", info.release_date);
    }

    println!(
        "⏱️  Duration: {}:{:02}",
        info.duration_ms / 60000,
        (info.duration_ms % 60000) / 1000
    );
    println!("⭐ Popularity: {}/100", info.popularity);

    if !info.genres.is_empty() {
        println!("🎸 Genres: {}", info.genres);
    }

    if !info.producers.is_empty() {
        println!("🎛️  Producers: {}", info.producers);
    }

    if !info.writers.is_empty() {
        println!("✍️  Writers: {}", info.writers);
    }

    if let Some(lyrics) = &info.lyrics {
        println!("\n📝 Lyrics:\n");
        println!("{}", lyrics);
    }
}
