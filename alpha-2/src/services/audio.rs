use std::sync::{Arc, Mutex};
use crate::common::TosState;

pub struct AudioService {
    _state: Arc<Mutex<TosState>>,
}

impl AudioService {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        Self { _state: state }
    }

    /// Trigger a specific system earcon (audio notification).
    pub fn play_earcon(&self, name: &str) {
        // Alpha-2 logic: just log for verification
        println!("[EARCON TRIGGER] Playing cue: {}", name);
    }
}
