use anyhow::{anyhow, Context, Result};
use std::process::Command;

use crate::db::TrackInfo;

/// Client that reads track information from the local Spotify desktop app.
///
/// On macOS, this uses AppleScript via `osascript`. No API credentials are needed.
pub struct SpotifyClient;

impl SpotifyClient {
    /// Create a new Spotify client.
    ///
    /// Returns an error on unsupported platforms.
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Get the currently playing track from the Spotify desktop app.
    ///
    /// Returns an error if Spotify is not running or no track is playing.
    pub async fn get_current_track(&self) -> Result<TrackInfo> {
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
    fn get_current_track_macos(&self) -> Result<TrackInfo> {
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
            return Err(anyhow!(
                "Spotify is not running or no track is playing. \
                 Make sure Spotify desktop app is open and playing a song.\nError: {}",
                error.trim()
            ));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();

        if parts.len() < 5 {
            return Err(anyhow!("Failed to parse Spotify track information"));
        }

        let track_id = parts[0].to_string(); // Spotify URI: spotify:track:xxxxx
        let track_name = parts[1].to_string();
        let artist_name = parts[2].to_string();
        let album_name = parts[3].to_string();
        let duration_ms = parts[4].parse::<i64>().unwrap_or(0);

        Ok(TrackInfo {
            track_id,
            track_name,
            artist_name,
            album_name,
            release_date: String::new(),
            duration_ms,
            popularity: 0,
            genres: String::new(),
            lyrics: None,
            producers: String::new(),
            writers: String::new(),
        })
    }
}
