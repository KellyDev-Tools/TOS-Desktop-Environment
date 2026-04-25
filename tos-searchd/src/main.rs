//! TOS Search Daemon (`tos-searchd`)
//!
//! High-performance hybrid search runner.

use notify::{RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tos_searchd::SearchState;

/// Entry point for the TOS Search Daemon.
///
/// Initializes the search state, starts the indexing worker, and listens
/// for search requests on a Unix Domain Socket.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = Arc::new(SearchState::new()?);

    // Unix Domain Socket setup
    let socket_path = "/tmp/tos-search.sock";
    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;
    tracing::info!(
        "TOS-SEARCHD: Hybrid Engine ONLINE on {} (Candle Lib)",
        socket_path
    );

    // Channel for indexing requests to avoid spawning from non-tokio threads
    let (tx, mut rx) = mpsc::channel::<PathBuf>(100);
    let indexer_state = state.clone();

    tokio::spawn(async move {
        while let Some(path) = rx.recv().await {
            let _ = indexer_state.index_file(&path).await;
        }
    });

    // Initial walk
    let initial_tx = tx.clone();
    let initial_state = state.clone();
    tokio::task::spawn(async move {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        tracing::info!("TOS-SEARCHD: Starting initial indexing of {:?}", root);

        let walker = walkdir::WalkDir::new(root)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let _ = initial_tx.send(entry.path().to_path_buf()).await;
            }
        }
        tracing::info!("TOS-SEARCHD: Initial Index Queueing Complete");

        // Wait a bit for indexing to catch up and then commit
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let _ = initial_state.commit();
        tracing::info!("TOS-SEARCHD: Initial Index Commit Complete");
    });

    // Notify Watcher
    let watcher_tx = tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if event.kind.is_modify() || event.kind.is_create() {
                for path in event.paths {
                    // Filter out excluded dirs in watcher too
                    let path_str = path.to_string_lossy();
                    if !path_str.contains("/target/")
                        && !path_str.contains("/node_modules/")
                        && !path_str.contains("/.git/")
                    {
                        let _ = watcher_tx.blocking_send(path);
                    }
                }
            }
        }
    })?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    // Periodic commit task
    let commit_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let _ = commit_state.commit();
        }
    });

    // Register with Brain
    let _ = tos_common::register_with_brain("tos-searchd", 0).await;

    loop {
        let (socket, _) = listener.accept().await?;
        let s = state.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                    break;
                }
                let req = line.trim();
                let parts: Vec<&str> = req.splitn(2, ':').collect();

                let response = match parts[0] {
                    "search" => {
                        let hits = s.search(parts.get(1).unwrap_or(&""));
                        serde_json::to_string(&hits).unwrap_or_default()
                    }
                    "semantic_search" | "semantic" => {
                        let hits = s.semantic_search(parts.get(1).unwrap_or(&""));
                        serde_json::to_string(&hits).unwrap_or_default()
                    }
                    "rebuild" => "OK".to_string(),
                    _ => "ERROR: Unknown command".to_string(),
                };

                let _ = writer
                    .write_all(format!("{}\n", response).as_bytes())
                    .await;
            }
        });
    }
}
