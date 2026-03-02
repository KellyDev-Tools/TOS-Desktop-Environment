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
        if let Some(ipc) = &*self.ipc.lock().unwrap() {
            // Forward log to the IPC handler for state append
            let _ = ipc.dispatch(&format!("system_log_append:{};{}", priority, text));
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
