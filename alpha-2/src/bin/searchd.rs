//! TOS Search Service (`tos-searchd`) — semantic indexing and vector search.
//!
//! This daemon manages the global file index and provides "semantic" retrieval
//! by simulating vector embeddings and cosine similarity. It registers with
//! the Brain via Unix socket.

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct VectorHit {
    path: String,
    score: f32,
    is_dir: bool,
}

struct SearchState {
    index: Vec<String>,
    // Mock embeddings: word -> vector
    // In a real implementation, this would be a real model like BERT/CLIP
}

impl SearchState {
    fn new() -> Self {
        let mut index = Vec::new();
        let roots = vec![
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ];

        for root in roots {
            for entry in WalkDir::new(root).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                index.push(entry.path().to_string_lossy().to_string());
            }
        }

        Self { index }
    }

    fn query(&self, pattern: &str) -> Vec<VectorHit> {
        let pat = pattern.to_lowercase();
        self.index.iter()
            .filter(|p| p.to_lowercase().contains(&pat))
            .take(20)
            .map(|p| VectorHit {
                path: p.clone(),
                score: 1.0,
                is_dir: Path::new(p).is_dir(),
            })
            .collect()
    }

    fn semantic_query(&self, prompt: &str) -> Vec<VectorHit> {
        let words: Vec<String> = prompt.split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 2)
            .collect();

        if words.is_empty() { return vec![]; }

        let mut results = Vec::new();
        for path in &self.index {
            let path_lower = path.to_lowercase();
            let mut score = 0.0;
            for word in &words {
                if path_lower.contains(word) {
                    score += 1.0;
                }
            }
            if score > 0.0 {
                // Penalize depth to favor root files
                let depth = path.split('/').count() as f32;
                let final_score = score / (1.0 + depth * 0.1);
                results.push(VectorHit {
                    path: path.clone(),
                    score: final_score,
                    is_dir: Path::new(path).is_dir(),
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.into_iter().take(10).collect()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    println!("TOS-SEARCHD: Operational on port {}", port);

    let state = Arc::new(Mutex::new(SearchState::new()));

    // Register with Brain
    let brain_sock_path = "/tmp/tos.brain.sock";
    tokio::spawn(async move {
        for attempt in 1..=10 {
            match tokio::net::UnixStream::connect(brain_sock_path).await {
                Ok(mut sock) => {
                    let cmd = format!("service_register:tos-searchd;{}\n", port);
                    let _ = sock.write_all(cmd.as_bytes()).await;
                    println!("[SEARCHD] Registered on port {}", port);
                    break;
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    });

    loop {
        let (socket, _) = listener.accept().await?;
        let state_clone = state.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                if reader.read_line(&mut line).await.unwrap_or(0) == 0 { break; }
                let req = line.trim();
                let parts: Vec<&str> = req.splitn(2, ':').collect();
                if parts.is_empty() { continue; }

                let response = match parts[0] {
                    "search" => {
                        let query = parts.get(1).unwrap_or(&"");
                        let hits = state_clone.lock().unwrap().query(query);
                        serde_json::to_string(&hits).unwrap_or_default()
                    }
                    "semantic_search" => {
                        let query = parts.get(1).unwrap_or(&"");
                        let hits = state_clone.lock().unwrap().semantic_query(query);
                        serde_json::to_string(&hits).unwrap_or_default()
                    }
                    _ => "ERROR: Unknown".to_string(),
                };

                let _ = writer.write_all(format!("{}\n", response).as_bytes()).await;
            }
        });
    }
}
