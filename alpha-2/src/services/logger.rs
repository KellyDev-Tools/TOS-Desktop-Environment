use crate::common::{TosState, TerminalLine};
use std::sync::{Arc, Mutex};
use chrono::Local;
use crate::services::audio::AudioService;

pub struct LoggerService {
    state: Arc<Mutex<TosState>>,
    audio: Arc<Mutex<Option<Arc<AudioService>>>>,
}

impl LoggerService {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        Self { 
            state,
            audio: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_audio_service(&self, audio: Arc<AudioService>) {
        let mut lock = self.audio.lock().unwrap();
        *lock = Some(audio);
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
        let limit = 1000;
        if state.system_log.len() > limit {
            let to_drain = state.system_log.len() - limit;
            state.system_log.drain(0..to_drain);
        }

        // §21.2: Multi-sensory feedback based on priority
        if let Some(audio) = &*self.audio.lock().unwrap() {
            if priority >= 3 {
                audio.play_earcon("priority_high_alert");
            } else if priority == 2 {
                audio.play_earcon("priority_mid_alert");
            }
        }

        println!("[LOG P{}] {}", priority, text);
    }
}
