use anyhow::Result;
use std::time::{Duration, Instant};

use crate::db::Database;
use crate::spotify::SpotifyClient;

/// Run the background polling daemon.
///
/// Polls Spotify every `poll_secs` seconds, records play events to the database,
/// and exits cleanly on Ctrl-C / SIGTERM.
pub async fn run(db: Database, poll_secs: u64) -> Result<()> {
    println!("🎧 Playbot daemon started (polling every {}s). Ctrl-C to stop.", poll_secs);

    let client = SpotifyClient::new()?;
    let mut current_track_id: Option<String> = None;
    let mut current_event_id: Option<i64> = None;
    let mut track_start: Option<Instant> = None;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(poll_secs)) => {
                let poll_result = client.get_current_track().await;

                match poll_result {
                    Ok(track) => {
                        let changed = current_track_id.as_deref() != Some(&track.track_id);
                        if changed {
                            // Close the previous event
                            if let (Some(eid), Some(start)) = (current_event_id, track_start) {
                                let elapsed_ms = start.elapsed().as_millis() as i64;
                                let _ = db.record_play_end(eid, elapsed_ms);
                            }

                            // Cache the track if not already present
                            if db.get_track_info(&track.track_id)?.is_none() {
                                db.insert_track_info(&track)?;
                            }

                            let eid = db.record_play_start(&track.track_id)?;
                            current_event_id = Some(eid);
                            track_start = Some(Instant::now());

                            println!("▶  {} — {}", track.track_name, track.artist_name);
                            current_track_id = Some(track.track_id);
                        }
                    }
                    Err(_) => {
                        // Spotify not running or paused — close any open event
                        if let (Some(eid), Some(start)) = (current_event_id.take(), track_start.take()) {
                            let elapsed_ms = start.elapsed().as_millis() as i64;
                            let _ = db.record_play_end(eid, elapsed_ms);
                        }
                        current_track_id = None;
                    }
                }
            }
        }
    }

    // Clean shutdown: close any open event
    if let (Some(eid), Some(start)) = (current_event_id, track_start) {
        let elapsed_ms = start.elapsed().as_millis() as i64;
        let _ = db.record_play_end(eid, elapsed_ms);
    }

    println!("\n🛑 Daemon stopped.");
    Ok(())
}
