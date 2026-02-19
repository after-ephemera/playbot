mod config;
mod db;
mod lyrics;
mod spotify;
mod tui;

use anyhow::Result;
use clap::Parser;

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
    if cli.browse {
        return tui::run(db);
    }
    if cli.count {
        return handle_count(&db);
    }
    if let Some(query) = &cli.search {
        return handle_search(&db, query).await;
    }
    if cli.recent {
        return handle_recent(&db);
    }
    handle_now_playing(cli, config, db).await
}

fn handle_count(db: &db::Database) -> Result<()> {
    let count = db.count_tracks()?;

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

async fn handle_search(db: &db::Database, query: &str) -> Result<()> {
    let results = db.search_tracks(query)?;

    if results.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    // Try to get currently playing track (if Spotify is running) to highlight it
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

fn handle_recent(db: &db::Database) -> Result<()> {
    let recent_tracks = db.get_recent_tracks(10)?;

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

async fn handle_now_playing(cli: Cli, config: config::Config, db: db::Database) -> Result<()> {
    let spotify_client = spotify::SpotifyClient::new()?;
    let track_info = spotify_client.get_current_track().await?;

    println!(
        "🎵 Now Playing: {} by {}",
        track_info.track_name, track_info.artist_name
    );

    if !cli.refresh {
        if let Some(cached_info) = db.get_track_info(&track_info.track_id)? {
            println!("\n📦 (Using cached data)\n");
            print_track_info(&cached_info);
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

    println!("\n✨ Fresh data fetched!\n");
    print_track_info(&full_info);

    // Suppress unused variable warning when config has no runtime-used fields
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
