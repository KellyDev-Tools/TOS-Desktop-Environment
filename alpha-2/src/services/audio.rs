use std::sync::{Arc, Mutex};
use crate::common::TosState;

pub struct AudioService {
    _state: Arc<Mutex<TosState>>,
}

impl AudioService {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        Self { _state: state }
    }

    /// §21.2: Trigger specific earcon
    pub fn play_earcon(&self, earcon_id: &str) {
        // Alpha-2 logic: just log for verification
        println!("[EARCON TRIGGER] Playing cue: {}", earcon_id);
    }
}
