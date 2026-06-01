/// Minimal MCP server over stdio (JSON-RPC 2.0).
///
/// Exposes playbot's local database as agent-callable tools so that agents can
/// query listening history and stats without parsing CLI output.
///
/// Protocol: newline-delimited JSON on stdin/stdout.
/// Supported methods: initialize, tools/list, tools/call
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::db::Database;
use crate::lyrics::LyricsClient;
use crate::spotify::SpotifyClient;

#[derive(Deserialize)]
struct Request {
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct Response {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn ok(id: Value, result: Value) -> Response {
    Response {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

fn err(id: Value, code: i32, message: &str) -> Response {
    Response {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(json!({"code": code, "message": message})),
    }
}

fn tool_list() -> Value {
    json!({
        "tools": [
            {
                "name": "get_now_playing",
                "description": "Get the currently playing Spotify track with full cached metadata and play count. Returns null if nothing is playing.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "search_library",
                "description": "Search the local playbot library by track name, artist, or album. Returns cached tracks that match the query.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search term (case-insensitive substring match)"}
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_recent",
                "description": "Get the most recently cached tracks from the local library.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": {"type": "integer", "description": "Maximum number of tracks to return (default: 10, max: 50)"}
                    },
                    "required": []
                }
            },
            {
                "name": "get_stats",
                "description": "Get listening analytics: top tracks, top artists, and daily listening time. Requires the daemon to have been running to accumulate data.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "days": {"type": "integer", "description": "Look-back window in days (default: 30)"}
                    },
                    "required": []
                }
            },
            {
                "name": "get_lyrics",
                "description": "Get lyrics for a track. Uses the cache if available, otherwise fetches live.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "track_id": {"type": "string", "description": "Spotify track URI (e.g. spotify:track:xxxxx)"}
                    },
                    "required": ["track_id"]
                }
            }
        ]
    })
}

async fn dispatch_tool(name: &str, args: &Value, db: &Database) -> Value {
    match name {
        "get_now_playing" => {
            let client = match SpotifyClient::new() {
                Ok(c) => c,
                Err(e) => return json!({"error": e.to_string()}),
            };
            match client.get_current_track().await {
                Ok(track) => {
                    let play_count = db.get_play_count(&track.track_id).unwrap_or(0);
                    let cached = db.get_track_info(&track.track_id).unwrap_or(None);
                    let info = cached.unwrap_or(track);
                    let mut v = serde_json::to_value(&info).unwrap();
                    v["play_count"] = json!(play_count);
                    v
                }
                Err(_) => json!(null),
            }
        }

        "search_library" => {
            let query = match args.get("query").and_then(|v| v.as_str()) {
                Some(q) => q,
                None => return json!({"error": "missing required parameter: query"}),
            };
            match db.search_tracks(query) {
                Ok(tracks) => serde_json::to_value(tracks).unwrap_or(json!([])),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "get_recent" => {
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(10)
                .min(50) as usize;
            match db.get_recent_tracks(limit) {
                Ok(tracks) => serde_json::to_value(tracks).unwrap_or(json!([])),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "get_stats" => {
            let days = args.get("days").and_then(|v| v.as_u64()).unwrap_or(30) as usize;
            match db.get_stats(days) {
                Ok(stats) => serde_json::to_value(stats).unwrap_or(json!({})),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "get_lyrics" => {
            let track_id = match args.get("track_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return json!({"error": "missing required parameter: track_id"}),
            };
            if let Ok(Some(cached)) = db.get_track_info(track_id) {
                if let Some(lyrics) = cached.lyrics {
                    return json!({"lyrics": lyrics, "source": "cache"});
                }
                let lc = LyricsClient::new();
                match lc.get_lyrics(&cached.track_name, &cached.artist_name).await {
                    Ok(lyrics) => json!({"lyrics": lyrics, "source": "live"}),
                    Err(e) => json!({"error": e.to_string()}),
                }
            } else {
                json!({"error": "track not found in local library"})
            }
        }

        _ => json!({"error": format!("unknown tool: {}", name)}),
    }
}

/// Run the MCP stdio server loop. Exits when stdin closes.
pub async fn serve(db: Database) -> Result<()> {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let resp = err(json!(null), -32700, &format!("Parse error: {}", e));
                let out = serde_json::to_string(&resp)? + "\n";
                stdout.write_all(out.as_bytes()).await?;
                stdout.flush().await?;
                continue;
            }
        };

        let response = match req.method.as_str() {
            "initialize" => ok(
                req.id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "playbot", "version": env!("CARGO_PKG_VERSION")}
                }),
            ),

            "tools/list" => ok(req.id, tool_list()),

            "tools/call" => {
                let name = req
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let args = req.params.get("arguments").cloned().unwrap_or(json!({}));

                let result = dispatch_tool(name, &args, &db).await;
                ok(
                    req.id,
                    json!({
                        "content": [{"type": "text", "text": serde_json::to_string_pretty(&result)?}]
                    }),
                )
            }

            _ => err(req.id, -32601, "Method not found"),
        };

        let out = serde_json::to_string(&response)? + "\n";
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}
