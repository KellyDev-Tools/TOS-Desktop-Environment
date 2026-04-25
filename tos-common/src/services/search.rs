//! Search Service — interface for global and semantic retrieval.
//!
//! This service communicates with the `tos-searchd` daemon to provide
//! indexed file searching and semantic "vector" retrieval.

use crate::services::registry::ServiceRegistry;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// The type of a search result hit.
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub enum SearchHitType {
    /// The hit is a file.
    File,
    /// The hit is a directory.
    Directory,
}

/// A single search result from either exact or semantic search.
#[derive(Clone, serde::Deserialize, Debug)]
pub struct SearchHit {
    /// The absolute path to the hit.
    pub path: String,
    /// Whether the hit is a file or directory.
    pub hit_type: SearchHitType,
    /// The relevance score (higher is better).
    pub score: f32,
}

/// Service for interacting with the TOS search daemon.
pub struct SearchService {
    registry: Arc<Mutex<ServiceRegistry>>,
}

impl SearchService {
    /// Create a new SearchService with the given service registry.
    pub fn new(registry: Arc<Mutex<ServiceRegistry>>) -> Self {
        Self { registry }
    }

    /// Perform a literal regex/substring search.
    pub fn query(&self, pattern: &str) -> Vec<SearchHit> {
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(h) => h,
            Err(_) => return vec![], // Not in a tokio context
        };
        rt.block_on(async move {
            self.remote_call("search", pattern)
                .await
                .unwrap_or_default()
        })
    }

    /// Perform a semantic "vector" search.
    pub fn semantic_query(&self, prompt: &str) -> Vec<SearchHit> {
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(h) => h,
            Err(_) => return vec![], // Not in a tokio context
        };
        rt.block_on(async move {
            self.remote_call("semantic_search", prompt)
                .await
                .unwrap_or_default()
        })
    }

    async fn remote_call(&self, cmd: &str, payload: &str) -> anyhow::Result<Vec<SearchHit>> {
        let port = {
            let reg = self
                .registry
                .lock()
                .map_err(|_| anyhow::anyhow!("Service registry mutex poisoned"))?;
            reg.port_of("tos-searchd")
        }
        .ok_or_else(|| anyhow::anyhow!("Search daemon not found"))?;

        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
        stream
            .write_all(format!("{}:{}\n", cmd, payload).as_bytes())
            .await?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        let hits: Vec<SearchHit> = serde_json::from_str(response.trim())?;
        Ok(hits)
    }
}
