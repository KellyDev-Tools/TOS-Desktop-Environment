//! TOS Session Service (`tos-sessiond`) — session persistence and workspace memory.
//!
//! This daemon handles all session file I/O: auto-saving the live state,
//! managing named sessions, and providing crash recovery via atomic
//! temp-file writes. It registers with the Brain's service registry on
//! startup using an ephemeral TCP port.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// In-memory state tracking for the session service.
struct SessionState {
    /// Path to the sessions directory (`~/.local/share/tos/sessions/`).
    sessions_dir: PathBuf,
    /// Whether a live state write is currently debounced.
    #[allow(dead_code)]
    write_pending: bool,
}

impl SessionState {
    fn new() -> Self {
        let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        dir.push("tos/sessions");

        Self {
            sessions_dir: dir,
            write_pending: false,
        }
    }

    /// Ensure the sessions directory exists on disk.
    fn ensure_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Bind ephemeral port — the OS assigns a free port.
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    tracing::info!("TOS-SESSIOND: Operational on ephemeral port {}", port);

    let session_state = Arc::new(Mutex::new(SessionState::new()));

    // Ensure the sessions directory exists.
    {
        let lock = session_state.lock().unwrap();
        if let Err(e) = lock.ensure_dir() {
            tracing::warn!("TOS-SESSIOND WARNING: Could not create sessions dir: {}", e);
        }
    }

    // §4.1: Dynamic Port Registration Gate
    tos_common::register_with_brain("tos-sessiond", port).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let state_clone = session_state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, state_clone).await {
                tracing::error!("[SESSIOND] Client error: {}", e);
            }
        });
    }
}

/// Handle a single client connection with line-delimited IPC messages.
async fn handle_client(
    socket: TcpStream,
    session_state: Arc<Mutex<SessionState>>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let request = line.trim();
        if request.is_empty() {
            continue;
        }

        let parts: Vec<&str> = request.splitn(2, ':').collect();
        let prefix = parts[0];
        let payload = if parts.len() > 1 { parts[1] } else { "" };

        let response = match prefix {
            "session_list" => {
                // List named sessions for a sector. Payload: sector_id
                let target_sector = payload.trim();
                let lock = session_state.lock().unwrap();
                let dir = &lock.sessions_dir;
                match std::fs::read_dir(dir) {
                    Ok(entries) => {
                        let sessions: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .filter_map(|e| {
                                let path = e.path();
                                if path.extension().unwrap_or_default() == "tos-session" {
                                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                        if stem != "_live" {
                                            // Format is sector_id_name
                                            if target_sector.is_empty()
                                                || target_sector == "global"
                                                || stem.starts_with(&format!("{}_", target_sector))
                                            {
                                                // Extract just the name portion
                                                let parts: Vec<&str> =
                                                    stem.splitn(2, '_').collect();
                                                if parts.len() == 2 {
                                                    return Some(parts[1].to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                None
                            })
                            .collect();
                        serde_json::to_string(&sessions).unwrap_or_else(|_| "[]".to_string())
                    }
                    Err(_) => "[]".to_string(),
                }
            }
            "session_live_write" => {
                // Force an immediate synchronous live state write.
                // The payload is the full state JSON.
                let lock = session_state.lock().unwrap();
                let live_path = lock.sessions_dir.join("_live.tos-session");
                let tmp_path = lock.sessions_dir.join("_live.tos-session.tmp");
                drop(lock);

                match std::fs::write(&tmp_path, payload) {
                    Ok(_) => match std::fs::rename(&tmp_path, &live_path) {
                        Ok(_) => "OK".to_string(),
                        Err(e) => format!("ERROR: Atomic rename failed: {}", e),
                    },
                    Err(e) => format!("ERROR: Write failed: {}", e),
                }
            }
            "session_save" => {
                // Save a named session. Payload format: sector_id;name;json_data
                let args: Vec<&str> = payload.splitn(3, ';').collect();
                if args.len() < 3 {
                    "ERROR: Expected sector_id;name;json_data".to_string()
                } else {
                    let sector_id = args[0];
                    let name = args[1];
                    let data = args[2];
                    let lock = session_state.lock().unwrap();
                    let path = lock
                        .sessions_dir
                        .join(format!("{}_{}.tos-session", sector_id, name));
                    drop(lock);
                    match std::fs::write(&path, data) {
                        Ok(_) => "OK".to_string(),
                        Err(e) => format!("ERROR: {}", e),
                    }
                }
            }
            "session_load" => {
                // Load a named session. Payload: sector_id;name
                let args: Vec<&str> = payload.splitn(2, ';').collect();
                if args.len() < 2 {
                    "ERROR: Expected sector_id;name".to_string()
                } else {
                    let sector_id = args[0];
                    let name = args[1];
                    let lock = session_state.lock().unwrap();
                    let path = lock
                        .sessions_dir
                        .join(format!("{}_{}.tos-session", sector_id, name));
                    drop(lock);
                    match std::fs::read_to_string(&path) {
                        Ok(content) => content,
                        Err(e) => format!("ERROR: {}", e),
                    }
                }
            }
            "session_delete" => {
                // Delete a named session. Payload: sector_id;name
                let args: Vec<&str> = payload.splitn(2, ';').collect();
                if args.len() < 2 {
                    "ERROR: Expected sector_id;name".to_string()
                } else {
                    let sector_id = args[0];
                    let name = args[1];
                    let lock = session_state.lock().unwrap();
                    let path = lock
                        .sessions_dir
                        .join(format!("{}_{}.tos-session", sector_id, name));
                    drop(lock);
                    match std::fs::remove_file(&path) {
                        Ok(_) => "OK".to_string(),
                        Err(e) => format!("ERROR: {}", e),
                    }
                }
            }
            _ => "ERROR: Unknown command".to_string(),
        };

        writer
            .write_all(format!("{}\n", response).as_bytes())
            .await?;
        writer.flush().await?;
    }

    Ok(())
}
