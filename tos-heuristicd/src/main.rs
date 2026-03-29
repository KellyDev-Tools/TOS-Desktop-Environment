//! TOS Heuristic Service (`tos-heuristicd`) — predictive intelligence and smart completion.
//!
//! This daemon provides real-time predictive fillers, autocomplete-to-chip
//! suggestions, typo corrections, and heuristic sector labeling. It registers
//! with the Brain via Unix domain socket.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// In-memory state for the heuristic service.
struct HeuristicState {
    /// Cached common commands for typo matching.
    command_history: Vec<String>,
}

impl HeuristicState {
    fn new() -> Self {
        Self {
            command_history: vec![
                "ls".to_string(),
                "cd".to_string(),
                "cp".to_string(),
                "mv".to_string(),
                "rm".to_string(),
                "mkdir".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "git".to_string(),
                "make".to_string(),
                "cargo".to_string(),
                "sudo".to_string(),
                "apt".to_string(),
                "systemctl".to_string(),
                "docker".to_string(),
            ],
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Bind ephemeral port
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    tracing::info!("TOS-HEURISTICD: Operational on port {}", port);

    let state = Arc::new(Mutex::new(HeuristicState::new()));

    // §4.1: Dynamic Port Registration Gate
    tos_common::daemon::register_with_brain("tos-heuristicd", port).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let state_clone = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, state_clone).await {
                tracing::error!("[HEURISTICD] Client error: {}", e);
            }
        });
    }
}

async fn handle_client(socket: TcpStream, state: Arc<Mutex<HeuristicState>>) -> anyhow::Result<()> {
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
            "heuristic_query" => {
                // Format: keyword;cwd
                let args: Vec<&str> = payload.split(';').collect();
                let keyword = args.get(0).copied().unwrap_or("");
                let cwd = args.get(1).copied().unwrap_or(".");

                if keyword.is_empty() {
                    "[]".to_string()
                } else {
                    let results = generate_suggestions(keyword, cwd, &state);
                    serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
                }
            }
            "ping" => "pong".to_string(),
            _ => "ERROR: Unknown command".to_string(),
        };

        writer
            .write_all(format!("{}\n", response).as_bytes())
            .await?;
        writer.flush().await?;
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct Suggestion {
    text: String,
    score: f32,
    source: String,
}

fn generate_suggestions(
    keyword: &str,
    cwd_str: &str,
    state: &Arc<Mutex<HeuristicState>>,
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // 1. Path Completion
    let cwd = PathBuf::from(cwd_str);
    if let Ok(entries) = std::fs::read_dir(&cwd) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(keyword) {
                    suggestions.push(Suggestion {
                        text: name.to_string(),
                        score: 0.9,
                        source: "Path".to_string(),
                    });
                }
            }
        }
    }

    // 2. Command Typo Correction
    let lock = state.lock().unwrap();
    for cmd in &lock.command_history {
        let distance = levenshtein_distance(keyword, cmd);
        if distance > 0 && distance <= 2 {
            suggestions.push(Suggestion {
                text: cmd.clone(),
                score: 1.0 - (distance as f32 * 0.2),
                source: "Typo".to_string(),
            });
        }
    }

    // Sort by score descending
    suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    suggestions.truncate(5);
    suggestions
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let n = v1.len();
    let m = v2.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }

    for i in 1..=n {
        for j in 1..=m {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
            dp[i][j] = std::cmp::min(
                dp[i - 1][j] + 1,
                std::cmp::min(dp[i][j - 1] + 1, dp[i - 1][j - 1] + cost),
            );
        }
    }
    dp[n][m]
}
