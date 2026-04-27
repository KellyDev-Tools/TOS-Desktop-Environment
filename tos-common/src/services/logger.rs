// use crate::TerminalLine; // Replaced by IPC state append
use std::sync::{Arc, Mutex};
// use chrono::Local; // Replaced by IPC state append
use crate::ipc::IpcDispatcher;
use crate::services::audio::AudioService;

pub struct LoggerService {
    ipc: Arc<Mutex<Option<Arc<dyn IpcDispatcher>>>>,
    audio: Arc<Mutex<Option<Arc<AudioService>>>>,
    registry: Option<Arc<Mutex<crate::services::registry::ServiceRegistry>>>,
}

impl Default for LoggerService {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerService {
    pub fn new() -> Self {
        Self {
            ipc: Arc::new(Mutex::new(None)),
            audio: Arc::new(Mutex::new(None)),
            registry: None,
        }
    }

    pub fn with_registry(registry: Arc<Mutex<crate::services::registry::ServiceRegistry>>) -> Self {
        Self {
            ipc: Arc::new(Mutex::new(None)),
            audio: Arc::new(Mutex::new(None)),
            registry: Some(registry),
        }
    }

    pub fn set_ipc(&self, ipc: Arc<dyn IpcDispatcher>) {
        let mut lock = self.ipc.lock().unwrap();
        *lock = Some(ipc);
    }

    pub fn set_audio_service(&self, audio: Arc<AudioService>) {
        let mut lock = self.audio.lock().unwrap();
        *lock = Some(audio);
    }

    /// Log an event to the unified system storage.
    pub fn log(&self, text: &str, priority: u8) {
        self.log_event(text, priority, "system");
    }

    /// Log a structured event with a specific source.
    pub fn log_event(&self, text: &str, priority: u8, source: &str) {
        // §19.1: Remote Log Submission (tos-loggerd)
        let port = self
            .registry
            .as_ref()
            .and_then(
                |r: &Arc<Mutex<crate::services::registry::ServiceRegistry>>| {
                    r.lock().unwrap().port_of("tos-loggerd")
                },
            )
            .unwrap_or(7003); // Fallback to hardcoded for Alpha-2.1/Beta-0 transition

        let addr = format!("127.0.0.1:{}", port);
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            use std::io::Write;
            let _ = stream.write_all(format!("log:{};{};{}\n", text, priority, source).as_bytes());
        } else {
            // Fallback: Local IPC notification for state append
            if let Some(ipc) = &*self.ipc.lock().unwrap() {
                let _ = ipc.dispatch(&format!("system_log_append:{};{}", priority, text));
            }
        }

        // Multi-sensory feedback based on priority level
        if let Some(audio) = &*self.audio.lock().unwrap() {
            if priority >= 3 {
                audio.play_earcon("priority_high_alert");
            } else if priority == 2 {
                audio.play_earcon("priority_mid_alert");
            }
        }

        tracing::info!("[LOG P{}] [{}] {}", priority, source, text);
    }

    /// Query system logs via the Log Service (§3.3.4).
    pub fn query(&self, surface: Option<&str>, limit: Option<usize>) -> anyhow::Result<String> {
        let port = self
            .registry
            .as_ref()
            .and_then(
                |r: &Arc<Mutex<crate::services::registry::ServiceRegistry>>| {
                    r.lock().unwrap().port_of("tos-loggerd")
                },
            )
            .unwrap_or(7003);

        let addr = format!("127.0.0.1:{}", port);
        let mut stream = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(100),
        )?;
        use std::io::{BufRead, BufReader, Write};

        let query = serde_json::json!({
            "surface": surface,
            "limit": limit
        });

        let _ = stream.write_all(format!("query:{}\n", query).as_bytes());
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        Ok(response.trim().to_string())
    }

    /// Deep Inspection Audit Log for security auditing.
    pub fn audit_log(&self, actor: &str, action: &str, result: &str) {
        let msg = format!("AUDIT [{}]: {} -> {}", actor, action, result);
        self.log_event(&msg, 3, "security"); // Forced high priority

        // Final implementation would sign this entry cryptographically
        tracing::warn!("SECURITY AUDIT ENTRY: {}", msg);
    }

    /// Archive an AI interaction pair (§7.4).
    pub fn archive_ai(&self, behavior_id: &str, prompt: &str, response: &str) {
        let port = self
            .registry
            .as_ref()
            .and_then(|r| r.lock().unwrap().port_of("tos-loggerd"))
            .unwrap_or(7003);

        let addr = format!("127.0.0.1:{}", port);
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            use std::io::Write;
            let payload = serde_json::json!({
                "behavior_id": behavior_id,
                "prompt": prompt,
                "response": response
            });
            let _ =
                stream.write_all(format!("archive_ai:{}\n", payload).as_bytes());
        }
    }

    /// Automated crash dump collection (§6.10).
    pub fn crash_report(&self, payload: &str) {
        let port = self
            .registry
            .as_ref()
            .and_then(|r| r.lock().unwrap().port_of("tos-loggerd"))
            .unwrap_or(7003);

        let addr = format!("127.0.0.1:{}", port);
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            std::time::Duration::from_millis(50),
        ) {
            use std::io::Write;
            let _ = stream.write_all(format!("crash:{}\n", payload).as_bytes());
        }
    }
}
