//! Search Service — interface for global and semantic retrieval.
//!
//! This service communicates with the `tos-searchd` daemon to provide
//! indexed file searching and semantic "vector" retrieval.

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::services::registry::ServiceRegistry;

#[derive(Clone, serde::Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub is_dir: bool,
    pub score: f32,
}

pub struct SearchService {
    registry: Arc<Mutex<ServiceRegistry>>,
}

impl SearchService {
    pub fn new(registry: Arc<Mutex<ServiceRegistry>>) -> Self {
        Self { registry }
    }

    /// Perform a literal regex/substring search.
    pub fn query(&self, pattern: &str) -> Vec<SearchHit> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            self.remote_call("search", pattern).await.unwrap_or_default()
        })
    }

    /// Perform a semantic "vector" search.
    pub fn semantic_query(&self, prompt: &str) -> Vec<SearchHit> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            self.remote_call("semantic_search", prompt).await.unwrap_or_default()
        })
    }

    async fn remote_call(&self, cmd: &str, payload: &str) -> anyhow::Result<Vec<SearchHit>> {
        let port = {
            let reg = self.registry.lock().unwrap();
            reg.port_of("tos-searchd")
        }.ok_or_else(|| anyhow::anyhow!("Search daemon not found"))?;

        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
        stream.write_all(format!("{}:{}\n", cmd, payload).as_bytes()).await?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        let hits: Vec<SearchHit> = serde_json::from_str(response.trim())?;
        Ok(hits)
    }
}
