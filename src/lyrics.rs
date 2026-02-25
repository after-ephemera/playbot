use anyhow::{Context, Result};
use lyric_finder::{Client, LyricResult};

/// Client for fetching song lyrics automatically, without any API key.
pub struct LyricsClient {
    client: Client,
}

impl LyricsClient {
    /// Create a new lyrics client.
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }

    /// Fetch lyrics for a song by title and artist name.
    ///
    /// Returns the lyrics as a formatted string, or a "not found" message if
    /// no lyrics are available. Never returns an error for missing lyrics.
    pub async fn get_lyrics(&self, song_title: &str, artist_name: &str) -> Result<String> {
        let search_query = format!("{} {}", song_title, artist_name);

        let result = self
            .client
            .get_lyric(&search_query)
            .await
            .context("Failed to fetch lyrics")?;

        match result {
            LyricResult::Some {
                track,
                artists,
                lyric,
            } => {
                // Clean up the lyrics by removing metadata artifacts
                let cleaned_lyric = lyric
                    .trim()
                    // Remove patterns like "1 Contributor", "2 Contributors", etc.
                    .trim_start_matches(|c: char| c.is_numeric())
                    .trim_start_matches(" Contributor")
                    .trim_start_matches("s") // for plural
                    // Remove the song title + "Lyrics" prefix
                    .trim_start_matches(&track)
                    .trim_start_matches(" Lyrics")
                    .trim();

                Ok(format!("🎵 {}\n👤 {}\n\n{}", track, artists, cleaned_lyric))
            }
            LyricResult::None => Ok(format!(
                "No lyrics found for '{}' by '{}'",
                song_title, artist_name
            )),
        }
    }
}
