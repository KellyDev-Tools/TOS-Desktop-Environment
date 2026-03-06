use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::services::registry::ServiceRegistry;
use crate::common::TosState;

#[derive(Clone)]
pub struct SessionService {
    registry: Arc<Mutex<ServiceRegistry>>,
    last_trigger: Arc<Mutex<std::time::Instant>>,
}

impl SessionService {
    pub fn new(registry: Arc<Mutex<ServiceRegistry>>) -> Self {
        Self { 
            registry,
            last_trigger: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    fn get_address(&self) -> anyhow::Result<String> {
        let reg = self.registry.lock().unwrap();
        if let Some(port) = reg.port_of("tos-sessiond") {
            Ok(format!("127.0.0.1:{}", port))
        } else {
            Err(anyhow::anyhow!("tos-sessiond not found in registry"))
        }
    }

    /// Triggers a live write to the _live.tos-session file.
    pub fn save_live(&self, state: &TosState) -> anyhow::Result<()> {
        let addr = self.get_address()?;
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_millis(50)) {
            let json = serde_json::to_string(state)?;
            let _ = stream.write_all(format!("session_live_write:{}\n", json).as_bytes());
            return Ok(());
        }
        Err(anyhow::anyhow!("connection failed to session service"))
    }

    /// Triggers a debounced live write. Repeated calls within 2 seconds will only result in a single write.
    pub fn debounced_save_live(&self, state: Arc<Mutex<TosState>>) {
        let now = std::time::Instant::now();
        *self.last_trigger.lock().unwrap() = now;
        
        // Only one task needs to sleep and check if it's the final debounce winner
        let last_trigger_ref = self.last_trigger.clone();
        let service_ref = self.clone(); // Requires Clone implementation
        
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let current_last = *last_trigger_ref.lock().unwrap();
            
            // If the timestamp hasn't changed since we started sleeping, we are the winner
            if current_last == now {
                let state_lock = state.lock().unwrap();
                if let Err(e) = service_ref.save_live(&state_lock) {
                    tracing::warn!("Failed debounced live save: {}", e);
                }
            }
        });
    }

    /// Request sessiond to save a named session.
    pub fn save(&self, sector_id: &str, name: &str, state: &TosState) -> anyhow::Result<()> {
        let addr = self.get_address()?;
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_millis(50)) {
            let json = serde_json::to_string(state)?;
            let _ = stream.write_all(format!("session_save:{};{};{}\n", sector_id, name, json).as_bytes());
            return Ok(());
        }
        Err(anyhow::anyhow!("connection failed to session service"))
    }

    /// Load a named session from sessiond.
    pub fn load(&self, sector_id: &str, name: &str) -> anyhow::Result<String> {
        let addr = self.get_address()?;
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_millis(50)) {
            let _ = stream.write_all(format!("session_load:{};{}\n", sector_id, name).as_bytes());
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() {
                let trimmed = response.trim();
                if trimmed.starts_with("ERROR") {
                    return Err(anyhow::anyhow!("{}", trimmed));
                }
                return Ok(trimmed.to_string());
            }
        }
        Err(anyhow::anyhow!("connection failed to session service"))
    }

    /// Delete a named session.
    pub fn delete(&self, sector_id: &str, name: &str) -> anyhow::Result<()> {
        let addr = self.get_address()?;
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_millis(50)) {
            let _ = stream.write_all(format!("session_delete:{};{}\n", sector_id, name).as_bytes());
            return Ok(());
        }
        Err(anyhow::anyhow!("connection failed to session service"))
    }

    /// Get a JSON string array of session names for a sector.
    pub fn list(&self, _sector_id: &str) -> anyhow::Result<String> {
        let addr = self.get_address()?;
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_millis(50)) {
            let _ = stream.write_all(b"session_list:\n");
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() {
                return Ok(response.trim().to_string());
            }
        }
        Err(anyhow::anyhow!("connection failed to session service"))
    }
}
