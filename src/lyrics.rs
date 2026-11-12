use anyhow::{Context, Result};
use serde::Deserialize;

/// LRCLIB: a free, open lyrics API that requires no API key.
/// https://lrclib.net/docs
const LRCLIB_SEARCH_URL: &str = "https://lrclib.net/api/search";

/// LRCLIB asks clients to identify themselves via User-Agent.
const USER_AGENT: &str = concat!(
    "playbot/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/after-ephemera/playbot)"
);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibTrack {
    track_name: String,
    artist_name: String,
    #[serde(default)]
    instrumental: bool,
    plain_lyrics: Option<String>,
}

/// Client for fetching song lyrics automatically, without any API key.
pub struct LyricsClient {
    http: reqwest::Client,
}

impl LyricsClient {
    /// Create a new lyrics client.
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { http }
    }

    /// Fetch lyrics for a song by title and artist name.
    ///
    /// Returns the lyrics as a formatted string, or a "not found" message if
    /// no lyrics are available. Only returns an error if the lyrics service
    /// itself could not be reached or returned an unexpected response.
    pub async fn get_lyrics(&self, song_title: &str, artist_name: &str) -> Result<String> {
        let results: Vec<LrclibTrack> = self
            .http
            .get(LRCLIB_SEARCH_URL)
            .query(&[("track_name", song_title), ("artist_name", artist_name)])
            .send()
            .await
            .context("Failed to reach lyrics service")?
            .error_for_status()
            .context("Lyrics service returned an error")?
            .json()
            .await
            .context("Failed to parse lyrics response")?;

        Ok(format_result(&results, song_title, artist_name))
    }
}

/// Pick the best match from LRCLIB search results and format it for display.
fn format_result(results: &[LrclibTrack], song_title: &str, artist_name: &str) -> String {
    let with_lyrics = results.iter().find(|t| {
        t.plain_lyrics
            .as_deref()
            .is_some_and(|l| !l.trim().is_empty())
    });

    if let Some(t) = with_lyrics {
        let lyric = t.plain_lyrics.as_deref().unwrap_or_default().trim();
        return format!("🎵 {}\n👤 {}\n\n{}", t.track_name, t.artist_name, lyric);
    }

    if results.iter().any(|t| t.instrumental) {
        return format!("'{}' by '{}' is instrumental", song_title, artist_name);
    }

    format!("No lyrics found for '{}' by '{}'", song_title, artist_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(lyrics: Option<&str>, instrumental: bool) -> LrclibTrack {
        LrclibTrack {
            track_name: "Song".into(),
            artist_name: "Artist".into(),
            instrumental,
            plain_lyrics: lyrics.map(String::from),
        }
    }

    #[test]
    fn formats_first_result_with_lyrics() {
        let results = vec![track(None, false), track(Some("  la la la\n"), false)];
        assert_eq!(
            format_result(&results, "Song", "Artist"),
            "🎵 Song\n👤 Artist\n\nla la la"
        );
    }

    #[test]
    fn reports_instrumental() {
        let results = vec![track(None, true)];
        assert!(format_result(&results, "Song", "Artist").contains("instrumental"));
    }

    #[test]
    fn reports_not_found() {
        assert!(format_result(&[], "Song", "Artist").starts_with("No lyrics found"));
    }

    #[test]
    fn parses_lrclib_json() {
        let json = r#"[{"id":1,"trackName":"T","artistName":"A","instrumental":false,"plainLyrics":"hi","syncedLyrics":null}]"#;
        let parsed: Vec<LrclibTrack> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed[0].plain_lyrics.as_deref(), Some("hi"));
    }
}
