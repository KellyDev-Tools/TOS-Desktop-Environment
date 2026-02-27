use crate::common::{TosState, TerminalLine};
use std::sync::{Arc, Mutex};
use chrono::Local;

pub struct LoggerService {
    state: Arc<Mutex<TosState>>,
}

impl LoggerService {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        Self { state }
    }

    /// §19.1: Log an event to the unified storage
    pub fn log(&self, text: &str, priority: u8) {
        let mut state = self.state.lock().unwrap();
        
        let line = TerminalLine {
            text: text.to_string(),
            priority,
            timestamp: Local::now(),
        };

        state.system_log.push(line);
        
        // Enforce FIFO limit for system log (§29.2)
        let limit = 1000; // System log has larger limit than standard hubs
        if state.system_log.len() > limit {
            let to_drain = state.system_log.len() - limit;
            state.system_log.drain(0..to_drain);
        }

        // Also print to standard output for the Brain process log
        println!("[LOG P{}] {}", priority, text);
    }
}
