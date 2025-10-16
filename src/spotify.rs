use anyhow::{anyhow, Context, Result};
use rspotify::{
    model::AdditionalType,
    prelude::*,
    AuthCodeSpotify, Credentials, OAuth,
};
use crate::config::SpotifyConfig;

pub struct SpotifyClient {
    client: AuthCodeSpotify,
}

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

impl SpotifyClient {
    pub fn new(config: &SpotifyConfig) -> Result<Self> {
        let creds = Credentials::new(&config.client_id, &config.client_secret);
        
        // Set up OAuth with necessary scopes
        let oauth = OAuth {
            redirect_uri: "http://localhost:8888/callback".to_string(),
            scopes: rspotify::scopes!("user-read-currently-playing", "user-read-playback-state"),
            ..Default::default()
        };

        let client = AuthCodeSpotify::new(creds, oauth);

        Ok(Self { client })
    }

    pub async fn get_current_track(&self) -> Result<TrackInfoBasic> {
        // Attempt to authenticate
        let url = self.client.get_authorize_url(false)
            .context("Failed to get authorization URL")?;
        
        println!("\n🔐 Please authorize the application:");
        println!("   {}\n", url);
        println!("After authorizing, paste the redirect URL here:");
        
        let mut code_url = String::new();
        std::io::stdin()
            .read_line(&mut code_url)
            .context("Failed to read redirect URL")?;
        
        let code = self.client
            .parse_response_code(code_url.trim())
            .context("Failed to parse response code from URL")?;
        
        self.client.request_token(&code).await
            .context("Failed to request access token")?;

        // Get currently playing track
        let playing = self.client
            .current_playing(None, Some(&[AdditionalType::Track]))
            .await
            .context("Failed to get currently playing track")?
            .ok_or_else(|| anyhow!("No track is currently playing"))?;

        let track = match playing.item {
            Some(rspotify::model::PlayableItem::Track(track)) => track,
            _ => return Err(anyhow!("Currently playing item is not a track")),
        };

        let artist_id = track.artists.first()
            .ok_or_else(|| anyhow!("Track has no artists"))?
            .id.clone()
            .ok_or_else(|| anyhow!("Artist has no ID"))?;

        // Get artist details for genres
        let artist = self.client
            .artist(artist_id)
            .await
            .context("Failed to get artist details")?;

        let track_info = TrackInfoBasic {
            id: track.id.as_ref().map(|id| id.id().to_string()).unwrap_or_default(),
            title: track.name.clone(),
            artist: track.artists.iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            album: track.album.name.clone(),
            release_date: track.album.release_date.clone().unwrap_or_default(),
            duration_ms: track.duration.num_milliseconds(),
            popularity: track.popularity as i32,
            genres: artist.genres.clone(),
            producers: Vec::new(), // Spotify API doesn't provide this directly
            writers: Vec::new(),   // Spotify API doesn't provide this directly
        };

        Ok(track_info)
    }
}
