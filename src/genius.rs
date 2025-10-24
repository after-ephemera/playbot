use anyhow::{Context, Result};
use lyric_finder::{Client, LyricResult};

pub struct GeniusClient {
    client: Client,
}

impl GeniusClient {
    pub fn new(_access_token: &str) -> Self {
        let client = Client::new();
        Self { client }
    }

    pub async fn get_lyrics(&self, song_title: &str, artist_name: &str) -> Result<String> {
        // Search for the song - try song title first for better results
        let search_query = format!("{} {}", song_title, artist_name);

        let result = self.client
            .get_lyric(&search_query)
            .await
            .context("Failed to fetch lyrics from Genius")?;

        match result {
            LyricResult::Some { track, artists, lyric } => {
                // Clean up the lyrics by removing Genius metadata
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
            },
            LyricResult::None => {
                Ok(format!("No lyrics found for '{}' by '{}'", song_title, artist_name))
            }
        }
    }
}
