//! Heuristic Service — interface for predictive intelligence.
//!
//! This service communicates with the `tos-heuristicd` daemon to provide
//! real-time suggestions, typo corrections, and other smart features.

use crate::services::registry::ServiceRegistry;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct HeuristicService {
    registry: Arc<Mutex<ServiceRegistry>>,
}

impl HeuristicService {
    pub fn new(registry: Arc<Mutex<ServiceRegistry>>) -> Self {
        Self { registry }
    }

    /// Query the heuristic daemon for suggestions based on a keyword and CWD.
    pub async fn query(&self, keyword: &str, cwd: &str) -> anyhow::Result<String> {
        let port = {
            let reg = self.registry.lock().unwrap();
            reg.port_of("tos-heuristicd")
        }
        .ok_or_else(|| anyhow::anyhow!("Heuristic service not registered"))?;

        let addr = format!("127.0.0.1:{}", port);
        let mut stream = tokio::net::TcpStream::connect(addr).await?;

        let request = format!("heuristic_query:{};{}\n", keyword, cwd);
        stream.write_all(request.as_bytes()).await?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        Ok(response.trim().to_string())
    }

    /// Record a command to the history for future suggestions.
    pub async fn record_history(&self, command: &str) -> anyhow::Result<()> {
        let port = {
            let reg = self.registry.lock().unwrap();
            reg.port_of("tos-heuristicd")
        }
        .ok_or_else(|| anyhow::anyhow!("Heuristic service not registered"))?;

        let addr = format!("127.0.0.1:{}", port);
        let mut stream = tokio::net::TcpStream::connect(addr).await?;

        let request = format!("history_append:{}\n", command);
        stream.write_all(request.as_bytes()).await?;
        
        Ok(())
    }
}
