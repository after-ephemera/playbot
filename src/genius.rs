use anyhow::{Context, Result};
use genius_rust::Genius;

pub struct GeniusClient {
    client: Genius,
}

impl GeniusClient {
    pub fn new(access_token: &str) -> Self {
        let client = Genius::new(access_token.to_string());
        Self { client }
    }

    pub async fn get_lyrics(&self, song_title: &str, artist_name: &str) -> Result<String> {
        // Search for the song
        let search_query = format!("{} {}", artist_name, song_title);
        
        let search_results = self.client
            .search(&search_query)
            .await
            .context("Failed to search for song on Genius")?;

        if search_results.is_empty() {
            return Ok(format!("No lyrics found for '{}' by '{}'", song_title, artist_name));
        }

        // Get the first result
        let song = &search_results[0];
        
        // Fetch the lyrics
        let lyrics = self.client
            .get_lyrics(song.result.id)
            .await
            .context("Failed to fetch lyrics from Genius")?;

        if lyrics.is_empty() {
            Ok("Lyrics not available".to_string())
        } else {
            Ok(lyrics.join("\n"))
        }
    }
}
