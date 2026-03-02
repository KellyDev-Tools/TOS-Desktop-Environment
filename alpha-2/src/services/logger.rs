// use crate::common::TerminalLine; // Replaced by IPC state append
use std::sync::{Arc, Mutex};
// use chrono::Local; // Replaced by IPC state append
use crate::services::audio::AudioService;
use crate::common::ipc_dispatcher::IpcDispatcher;

pub struct LoggerService {
    ipc: Arc<Mutex<Option<Arc<dyn IpcDispatcher>>>>,
    audio: Arc<Mutex<Option<Arc<AudioService>>>>,
}

impl LoggerService {
    pub fn new() -> Self {
        Self { 
            ipc: Arc::new(Mutex::new(None)),
            audio: Arc::new(Mutex::new(None)),
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
        // §19.1: Remote Log Submission (tos-loggerd)
        // If the daemon is active, it handles the state broadcast to the Brain via Port 7000.
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(&"127.0.0.1:7003".parse().unwrap(), std::time::Duration::from_millis(50)) {
            use std::io::Write;
            let _ = stream.write_all(format!("log:{};{}\n", text, priority).as_bytes());
        } else {
            // Fallback: Local IPC notification for state append (Face visualization)
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

        println!("[LOG P{}] {}", priority, text);
    }

    /// Deep Inspection Audit Log for security auditing.
    pub fn audit_log(&self, actor: &str, action: &str, result: &str) {
        let msg = format!("AUDIT [{}]: {} -> {}", actor, action, result);
        self.log(&msg, 3); // Forced high priority
        
        // Final implementation would sign this entry cryptographically
        tracing::warn!("SECURITY AUDIT ENTRY: {}", msg);
    }
}
