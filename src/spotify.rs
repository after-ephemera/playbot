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
        #[cfg(target_os = "linux")]
        {
            self.get_current_track_linux()
        }

        #[cfg(target_os = "macos")]
        {
            self.get_current_track_macos()
        }

        #[cfg(target_os = "windows")]
        {
            self.get_current_track_windows()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow!("Unsupported platform"))
        }
    }

    #[cfg(target_os = "linux")]
    fn get_current_track_linux(&self) -> Result<TrackInfoBasic> {
        // Try using dbus-send to query Spotify via MPRIS
        let output = Command::new("dbus-send")
            .args([
                "--print-reply",
                "--dest=org.mpris.MediaPlayer2.spotify",
                "/org/mpris/MediaPlayer2",
                "org.freedesktop.DBus.Properties.Get",
                "string:org.mpris.MediaPlayer2.Player",
                "string:Metadata",
            ])
            .output()
            .context("Failed to execute dbus-send. Make sure dbus-send is installed and Spotify is running.")?;

        if !output.status.success() {
            return Err(anyhow!("Spotify is not running or no track is playing. Make sure Spotify desktop app is open and playing a song."));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        
        // Parse the dbus output
        let title = Self::extract_dbus_field(&result, "xesam:title")?;
        let artist = Self::extract_dbus_field(&result, "xesam:artist")?;
        let album = Self::extract_dbus_field(&result, "xesam:album")
            .unwrap_or_else(|_| "Unknown Album".to_string());

        let id = format!("{}-{}", 
            title.to_lowercase().replace(' ', "-"),
            artist.to_lowercase().replace(' ', "-")
        );

        Ok(TrackInfoBasic {
            id,
            title,
            artist,
            album,
            release_date: String::new(),
            duration_ms: 0,
            popularity: 0,
            genres: Vec::new(),
            producers: Vec::new(),
            writers: Vec::new(),
        })
    }

    #[cfg(target_os = "linux")]
    fn extract_dbus_field(output: &str, field: &str) -> Result<String> {
        // Find the field in the dbus output
        let field_marker = format!("string \"{}\"", field);
        if let Some(pos) = output.find(&field_marker) {
            // Look for the value after the field marker
            let after_marker = &output[pos + field_marker.len()..];
            
            // For arrays (like artists), look for variant array
            if field == "xesam:artist" {
                if let Some(start) = after_marker.find("string \"") {
                    let value_part = &after_marker[start + 8..];
                    if let Some(end) = value_part.find('"') {
                        return Ok(value_part[..end].to_string());
                    }
                }
            } else {
                // For regular strings, look for the next string value
                if let Some(start) = after_marker.find("string \"") {
                    let value_part = &after_marker[start + 8..];
                    if let Some(end) = value_part.find('"') {
                        return Ok(value_part[..end].to_string());
                    }
                }
            }
        }
        Err(anyhow!("Field {} not found in metadata", field))
    }

    #[cfg(target_os = "macos")]
    fn get_current_track_macos(&self) -> Result<TrackInfoBasic> {
        // Use osascript to query Spotify
        let script = r#"
            if application "Spotify" is running then
                tell application "Spotify"
                    if player state is playing then
                        set trackName to name of current track
                        set artistName to artist of current track
                        set albumName to album of current track
                        set trackDuration to duration of current track
                        return trackName & "|" & artistName & "|" & albumName & "|" & trackDuration
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

        if parts.len() < 4 {
            return Err(anyhow!("Failed to parse Spotify track information"));
        }

        let title = parts[0].to_string();
        let artist = parts[1].to_string();
        let album = parts[2].to_string();
        let duration_ms = parts[3].parse::<i64>().unwrap_or(0);

        let id = format!("{}-{}", 
            title.to_lowercase().replace(' ', "-"),
            artist.to_lowercase().replace(' ', "-")
        );

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

    #[cfg(target_os = "windows")]
    fn get_current_track_windows(&self) -> Result<TrackInfoBasic> {
        // On Windows, we can use PowerShell to query the media session
        let script = r#"
            Add-Type -AssemblyName System.Runtime.WindowsRuntime
            $asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' })[0]
            
            Function Await($WinRtTask, $ResultType) {
                $asTask = $asTaskGeneric.MakeGenericMethod($ResultType)
                $netTask = $asTask.Invoke($null, @($WinRtTask))
                $netTask.Wait(-1) | Out-Null
                $netTask.Result
            }
            
            [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager,Windows.Media.Control,ContentType=WindowsRuntime] | Out-Null
            $sessionManager = Await ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]::RequestAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager])
            $currentSession = $sessionManager.GetCurrentSession()
            
            if ($null -eq $currentSession) {
                Write-Error "No active media session"
                exit 1
            }
            
            $mediaProperties = Await ($currentSession.TryGetMediaPropertiesAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties])
            
            $title = $mediaProperties.Title
            $artist = $mediaProperties.Artist
            $album = $mediaProperties.AlbumTitle
            
            Write-Output "$title|$artist|$album"
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .context("Failed to execute PowerShell")?;

        if !output.status.success() {
            return Err(anyhow!("Spotify is not running or no track is playing. Make sure Spotify desktop app is open and playing a song."));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();

        if parts.len() < 3 {
            return Err(anyhow!("Failed to parse media information"));
        }

        let title = parts[0].to_string();
        let artist = parts[1].to_string();
        let album = parts[2].to_string();

        let id = format!("{}-{}", 
            title.to_lowercase().replace(' ', "-"),
            artist.to_lowercase().replace(' ', "-")
        );

        Ok(TrackInfoBasic {
            id,
            title,
            artist,
            album,
            release_date: String::new(),
            duration_ms: 0,
            popularity: 0,
            genres: Vec::new(),
            producers: Vec::new(),
            writers: Vec::new(),
        })
    }
}

