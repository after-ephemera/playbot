use anyhow::Result;
use std::time::{Duration, Instant};

use crate::db::Database;
use crate::mcp;
use crate::spotify::SpotifyClient;

/// Run the daemon and MCP server together.
///
/// The poll loop records play events to the database while the MCP server
/// concurrently serves JSON-RPC requests on stdio. Both stop on Ctrl-C or
/// when stdin closes.
pub async fn run(db_path: &str, poll_secs: u64) -> Result<()> {
    eprintln!(
        "🎧 Playbot daemon + MCP server started (polling every {}s). Ctrl-C to stop.",
        poll_secs
    );

    let mcp_db = Database::new(db_path)?;

    tokio::select! {
        r = poll_loop(db_path, poll_secs) => r,
        r = mcp::serve(mcp_db) => r,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\n🛑 Daemon stopped.");
            Ok(())
        }
    }
}

async fn poll_loop(db_path: &str, poll_secs: u64) -> Result<()> {
    let db = Database::new(db_path)?;
    let client = SpotifyClient::new()?;
    let mut current_track_id: Option<String> = None;
    let mut current_event_id: Option<i64> = None;
    let mut track_start: Option<Instant> = None;

    loop {
        tokio::time::sleep(Duration::from_secs(poll_secs)).await;

        match client.get_current_track().await {
            Ok(track) => {
                let changed = current_track_id.as_deref() != Some(&track.track_id);
                if changed {
                    if let (Some(eid), Some(start)) = (current_event_id, track_start) {
                        let elapsed_ms = start.elapsed().as_millis() as i64;
                        let _ = db.record_play_end(eid, elapsed_ms);
                    }

                    if db.get_track_info(&track.track_id)?.is_none() {
                        db.insert_track_info(&track)?;
                    }

                    let eid = db.record_play_start(&track.track_id)?;
                    current_event_id = Some(eid);
                    track_start = Some(Instant::now());

                    eprintln!("▶  {} — {}", track.track_name, track.artist_name);
                    current_track_id = Some(track.track_id);
                }
            }
            Err(_) => {
                if let (Some(eid), Some(start)) = (current_event_id.take(), track_start.take()) {
                    let elapsed_ms = start.elapsed().as_millis() as i64;
                    let _ = db.record_play_end(eid, elapsed_ms);
                }
                current_track_id = None;
            }
        }
    }
}
