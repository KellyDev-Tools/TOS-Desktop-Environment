//! TOS Search Service (`tos-searchd`) — semantic indexing and vector search.
//!
//! This daemon manages the global file index and provides "semantic" retrieval
//! by utilizing vector embeddings and cosine similarity via `fastembed`. It registers with
//! the Brain via Unix socket.

use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use walkdir::WalkDir;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct VectorHit {
    path: String,
    score: f32,
    is_dir: bool,
}

#[derive(Clone)]
struct DocumentEntry {
    path: String,
    mtime: u64,
    embedding: Vec<f32>,
    is_dir: bool,
}

struct SearchState {
    index: Vec<DocumentEntry>,
    model: TextEmbedding,
}

impl SearchState {
    fn new() -> Self {
        // Initialize the embedding model
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2))
            .expect("Failed to initialize fastembed model");

        Self { 
            index: Vec::new(),
            model,
        }
    }

    fn rebuild_index(&mut self) {
        let mut new_index = Vec::new();
        // In-memory cache of old embeddings
        let old_index = std::mem::take(&mut self.index);
        let mut old_map = std::collections::HashMap::new();
        for entry in old_index {
            old_map.insert(entry.path.clone(), entry);
        }

        let roots = vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))];
        let mut to_embed_paths = Vec::new();
        let mut to_embed_texts = Vec::new();

        for root in roots {
            for entry in WalkDir::new(root.clone()).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path().to_string_lossy().to_string();
                let metadata = entry.metadata().ok();
                let mtime = metadata.and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                    
                let is_dir = entry.file_type().is_dir();
                
                if let Some(old) = old_map.remove(&path) {
                    if old.mtime == mtime {
                        new_index.push(old);
                        continue;
                    }
                }
                
                to_embed_paths.push((path.clone(), mtime, is_dir));
                let file_name = entry.file_name().to_string_lossy().to_string();
                to_embed_texts.push(format!("{} {}", file_name, path));
            }
        }
        
        if !to_embed_texts.is_empty() {
            // Batch embed
            if let Ok(embeddings) = self.model.embed(to_embed_texts, None) {
                for ((path, mtime, is_dir), embedding) in to_embed_paths.into_iter().zip(embeddings) {
                    new_index.push(DocumentEntry { path, mtime, embedding, is_dir });
                }
            }
        }
        
        self.index = new_index;
    }

    fn query(&self, pattern: &str) -> Vec<VectorHit> {
        let pat = pattern.to_lowercase();
        self.index.iter()
            .filter(|p| p.path.to_lowercase().contains(&pat))
            .take(20)
            .map(|p| VectorHit {
                path: p.path.clone(),
                score: 1.0,
                is_dir: p.is_dir,
            })
            .collect()
    }

    fn semantic_query(&mut self, prompt: &str) -> Vec<VectorHit> {
        if let Ok(emb_vecs) = self.model.embed(vec![prompt], None) {
            if let Some(query_emb) = emb_vecs.into_iter().next() {
                let mut results = Vec::new();
                for entry in &self.index {
                    let score = cosine_similarity(&query_emb, &entry.embedding);
                    if score > 0.1 { // Minimal threshold
                        results.push(VectorHit {
                            path: entry.path.clone(),
                            score,
                            is_dir: entry.is_dir,
                        });
                    }
                }
                results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                return results.into_iter().take(10).collect();
            }
        }
        vec![]
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot_product / (norm_a * norm_b)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    tracing::info!("TOS-SEARCHD: Operational on port {}", port);

    let state = Arc::new(Mutex::new(SearchState::new()));
    
    // Background indexer
    let indexer_state = state.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            {
                let mut s = indexer_state.lock().unwrap();
                s.rebuild_index();
            }
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });

    // Register with Brain (§4.1)
    tos_lib::daemon::register_with_brain("tos-searchd", port).await?;

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
