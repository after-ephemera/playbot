use anyhow::{anyhow, Context, Result};
use std::process::Command;

#[derive(Debug)]
pub struct TrackInfoBasic {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub release_date: String,
    pub duration_ms: i64,
    pub popularity: i32,
    pub genres: Vec<String>,
    pub producers: Vec<String>,
    pub writers: Vec<String>,
}

pub struct SpotifyClient;

impl SpotifyClient {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn get_current_track(&self) -> Result<TrackInfoBasic> {
        #[cfg(target_os = "macos")]
        {
            self.get_current_track_macos()
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(anyhow!("Only macOS is currently supported"))
        }
    }

    #[cfg(target_os = "macos")]
    fn get_current_track_macos(&self) -> Result<TrackInfoBasic> {
        // Use osascript to query Spotify - now including the Spotify URI
        let script = r#"
            if application "Spotify" is running then
                tell application "Spotify"
                    if player state is playing then
                        set trackURI to spotify url of current track
                        set trackName to name of current track
                        set artistName to artist of current track
                        set albumName to album of current track
                        set trackDuration to duration of current track
                        return trackURI & "|" & trackName & "|" & artistName & "|" & albumName & "|" & trackDuration
                    else
                        error "No track is currently playing"
                    end if
                end tell
            else
                error "Spotify is not running"
            end if
        "#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .context("Failed to execute osascript")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Spotify is not running or no track is playing. Make sure Spotify desktop app is open and playing a song.\nError: {}", error.trim()));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();

        if parts.len() < 5 {
            return Err(anyhow!("Failed to parse Spotify track information"));
        }

        let id = parts[0].to_string(); // Now using actual Spotify URI: spotify:track:xxxxx
        let title = parts[1].to_string();
        let artist = parts[2].to_string();
        let album = parts[3].to_string();
        let duration_ms = parts[4].parse::<i64>().unwrap_or(0);

        Ok(TrackInfoBasic {
            id,
            title,
            artist,
            album,
            release_date: String::new(),
            duration_ms,
            popularity: 0,
            genres: Vec::new(),
            producers: Vec::new(),
            writers: Vec::new(),
        })
    }

}

