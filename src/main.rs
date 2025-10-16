mod config;
mod db;
mod spotify;
mod genius;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "playbot")]
#[command(about = "Get detailed information about the currently playing Spotify song", long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Force refresh data even if cached
    #[arg(short, long)]
    refresh: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = config::Config::load(&cli.config)?;

    // Initialize database
    let db = db::Database::new(&config.database.path)?;
    db.init()?;

    // Get currently playing track from Spotify
    let spotify_client = spotify::SpotifyClient::new(&config.spotify)?;
    let track_info = spotify_client.get_current_track().await?;

    println!("🎵 Now Playing: {} by {}", track_info.title, track_info.artist);

    // Check cache first
    if !cli.refresh {
        if let Some(cached_info) = db.get_track_info(&track_info.id)? {
            println!("\n📦 (Using cached data)\n");
            print_track_info(&cached_info);
            return Ok(());
        }
    }

    // Fetch lyrics from Genius
    let genius_client = genius::GeniusClient::new(&config.genius.access_token);
    let lyrics = genius_client.get_lyrics(&track_info.title, &track_info.artist).await?;

    // Combine all information
    let full_info = db::TrackInfo {
        track_id: track_info.id.clone(),
        track_name: track_info.title.clone(),
        artist_name: track_info.artist.clone(),
        album_name: track_info.album.clone(),
        release_date: track_info.release_date.clone(),
        duration_ms: track_info.duration_ms,
        popularity: track_info.popularity,
        genres: track_info.genres.join(", "),
        lyrics: Some(lyrics.clone()),
        producers: track_info.producers.join(", "),
        writers: track_info.writers.join(", "),
    };

    // Store in database
    db.insert_track_info(&full_info)?;

    println!("\n✨ Fresh data fetched!\n");
    print_track_info(&full_info);

    Ok(())
}

fn print_track_info(info: &db::TrackInfo) {
    println!("📀 Track: {}", info.track_name);
    println!("👤 Artist: {}", info.artist_name);
    println!("💿 Album: {}", info.album_name);
    
    if !info.release_date.is_empty() {
        println!("📅 Release Date: {}", info.release_date);
    }
    
    println!("⏱️  Duration: {}:{:02}", info.duration_ms / 60000, (info.duration_ms % 60000) / 1000);
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

