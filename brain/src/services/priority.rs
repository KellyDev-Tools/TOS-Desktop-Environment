use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityScore {
    pub total_score: f32, // 0.0 to 1.0
    pub rank: u8,         // 1 to 5
    pub factors: HashMap<String, f32>,
}

pub struct PriorityService {
    registry: Arc<Mutex<crate::services::registry::ServiceRegistry>>,
}

impl PriorityService {
    pub fn new(registry: Arc<Mutex<crate::services::registry::ServiceRegistry>>) -> Self {
        Self { registry }
    }

    /// Calculate priority score for a sector based on current system heuristics.
    /// Prioritizes the Priority Daemon if active.
    pub fn calculate_priority(&self, sector_id: Uuid) -> anyhow::Result<PriorityScore> {
        let port = self
            .registry
            .lock()
            .unwrap()
            .port_of("tos-priorityd")
            .unwrap_or(7005);
        let addr = format!("127.0.0.1:{}", port);
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            use std::io::{BufRead, BufReader, Write};
            let _ = stream.write_all(format!("get_priority:{}\n", sector_id).as_bytes());
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if let Ok(_) = reader.read_line(&mut response) {
                if let Ok(score) = serde_json::from_str(&response) {
                    return Ok(score);
                }
            }
        }

        // Fallback: Local basic scoring
        Ok(self.calculate_local_fallback(sector_id))
    }

    fn calculate_local_fallback(&self, _sector_id: Uuid) -> PriorityScore {
        let mut factors = HashMap::new();
        factors.insert("recency".to_string(), 0.8);
        factors.insert("activity".to_string(), 0.2);

        PriorityScore {
            total_score: 0.5,
            rank: 3,
            factors,
        }
    }
}
